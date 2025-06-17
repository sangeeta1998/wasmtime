//! # Wasmtime's [wasi-acc] Implementation
//!
//! This crate provides a Wasmtime host implementation of the [wasi-acc]
//! API. With this crate, the runtime can run components that call APIs in
//! [wasi-acc] and provide components with access to accelerated computing.
//!
//! Currently supported compute backends:
//! * CPUs
//! * Cuda enabled GPUs (to come)
//!

#![deny(missing_docs)]

mod generated {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "wasi:acc/imports",
    });
}

use self::generated::wasi::acc;
use anyhow::Result;
use std::collections::HashMap;
use wasmtime::component::{HasData, Resource, ResourceTable, ResourceTableError};


// Simulated host state
struct HostState {
    buffers: HashMap<acc::host_allocator::Handle, Vec<u8>>,
    matrix_dims: HashMap<acc::host_allocator::Handle, (u32, u32)>, // rows, cols
    next_handle: acc::host_allocator::Handle,
}

impl HostState {
    fn new() -> Self {
        HostState {
            buffers: HashMap::new(),
            matrix_dims: HashMap::new(),
            next_handle: 1, // Start handles from 1
        }
    }

    fn new_handle(&mut self) -> Handle {
        let handle = self.next_handle;
        self.next_handle += 1;
        if self.next_handle == 0 { panic!("Handle overflow!"); } 
        handle
    }
}

static HOST_STATE: Lazy<Mutex<HostState>> = Lazy::new(|| Mutex::new(HostState::new()));


/// Capture the state necessary for use in the `wasi-acc` API implementation.
pub struct WasiAccCtx {
    host_state: HostState,
    ctx: &'a WasiAccCtx,
    table: &'a mut ResourceTable,
}

impl WasiAccCtx {
    /// Convenience function for calling [`WasiAccCtxBuilder::new`].
    pub fn builder() -> WasiAcctxBuilder {
        WasiAccCtxBuilder::new()
    }
}

/// A wrapper capturing the needed internal `wasi-acc` state.
pub struct WasiAcc<'a> {
    ctx: &'a WasiAccCtx,
    table: &'a mut ResourceTable,
}

impl<'a> WasiAcc<'a> {
    /// Create a new view into the `wasi-acc` state.
    pub fn new(ctx: &'a WasiAccCtx, table: &'a mut ResourceTable) -> Self {
        Self { ctx, table }
    }
}


impl acc::host_allocator::Host for WasiAcc<'_> {
    fn allocate_buffer(&mut self, size: u64) -> Result<Handle, HostError> {
        println!("[Provider Wasm] Allocating buffer of size {}", size);
        if size == 0 {
            return Err(HostError::Other("Cannot allocate zero-size buffer".to_string()));
        }
        let mut state = HOST_STATE.lock().unwrap();
        let handle = state.new_handle();
        state.buffers.insert(handle, vec![0u8; size as usize]);
        Ok(handle)
    }

    fn free_buffer(h: Handle) -> Result<(), HostError> {
        println!("[Provider Wasm] Freeing buffer {}", h);
        let mut state = HOST_STATE.lock().unwrap();
        if state.buffers.remove(&h).is_some() {
            state.matrix_dims.remove(&h);
            Ok(())
        } else {
            Err(HostError::InvalidHandle)
        }
    }

    fn write_to_host(
        guest_bytes: Vec<u8>,
        target_handle: Handle,
        target_offset: u64,
    ) -> Result<(), HostError> {
        println!("[Provider Wasm] Writing {} bytes to handle {} at offset {}", guest_bytes.len(), target_handle, target_offset);
        let mut state = HOST_STATE.lock().unwrap();
        match state.buffers.get_mut(&target_handle) {
            Some(buffer) => {
                let offset = target_offset as usize;
                let end = offset + guest_bytes.len();
                if end > buffer.len() {
                    return Err(HostError::CopyOutOfBounds);
                }
                buffer[offset..end].copy_from_slice(&guest_bytes);
                Ok(())
            }
            None => Err(HostError::InvalidHandle),
        }
    }

    fn read_from_host(
        source_handle: Handle,
        source_offset: u64,
        len: u64,
    ) -> Result<Vec<u8>, HostError> {
        println!("[Provider Wasm] Reading {} bytes from handle {} at offset {}", len, source_handle, source_offset);
        let state = HOST_STATE.lock().unwrap();
        match state.buffers.get(&source_handle) {
            Some(buffer) => {
                let offset = source_offset as usize;
                let read_len = len as usize;
                if offset + read_len > buffer.len() {
                    return Err(HostError::CopyOutOfBounds);
                }
                Ok(buffer[offset..offset + read_len].to_vec())
            }
            None => Err(HostError::InvalidHandle),
        }
    }

    fn register_matrix_dimensions(h: Handle, dims: MatrixDimensions) -> Result<(), HostError> {
        println!("[Provider Wasm] Registering dimensions {}x{} for handle {}", dims.rows, dims.cols, h);
        let mut state = HOST_STATE.lock().unwrap();
        if !state.buffers.contains_key(&h) {
            return Err(HostError::InvalidHandle);
        }
        state.matrix_dims.insert(h, (dims.rows, dims.cols));
        Ok(())
    }


    fn matrix_multiply_f32(
        handle_a: Handle,
        handle_b: Handle,
    ) -> Result<Handle, HostError> {
        println!("[Provider Wasm] Matrix multiply f32 for A:{} and B:{}", handle_a, handle_b);
        let mut state = HOST_STATE.lock().unwrap();

        let (rows_a, cols_a) = *state.matrix_dims.get(&handle_a).ok_or(HostError::InvalidHandle)?;
        let buffer_a_bytes = state.buffers.get(&handle_a).ok_or(HostError::InvalidHandle)?;
        let matrix_a_data = bytes_to_f32_slice(buffer_a_bytes)
            .ok_or_else(|| HostError::Other("Failed to cast buffer A to f32".to_string()))?;
        if matrix_a_data.len() != (rows_a * cols_a) as usize { return Err(HostError::Other("Buffer A size mismatch with dims".to_string())); }
        let matrix_a = nalgebra::DMatrix::<f32>::from_row_slice(rows_a as usize, cols_a as usize, matrix_a_data);

        let (rows_b, cols_b) = *state.matrix_dims.get(&handle_b).ok_or(HostError::InvalidHandle)?;
        let buffer_b_bytes = state.buffers.get(&handle_b).ok_or(HostError::InvalidHandle)?;
        let matrix_b_data = bytes_to_f32_slice(buffer_b_bytes)
            .ok_or_else(|| HostError::Other("Failed to cast buffer B to f32".to_string()))?;
        if matrix_b_data.len() != (rows_b * cols_b) as usize { return Err(HostError::Other("Buffer B size mismatch with dims".to_string())); }
        let matrix_b = nalgebra::DMatrix::<f32>::from_row_slice(rows_b as usize, cols_b as usize, matrix_b_data);

        if cols_a != rows_b {
            return Err(HostError::DimensionMismatch);
        }

        let matrix_c = matrix_a * matrix_b;
        let handle_c = state.new_handle();
        let c_bytes = f32_slice_to_bytes(matrix_c.as_slice());
        state.buffers.insert(handle_c, c_bytes);
        state.matrix_dims.insert(handle_c, (matrix_c.nrows() as u32, matrix_c.ncols() as u32));
        println!("[Provider Wasm] Stored result C ({},{}) with handle {}", matrix_c.nrows(), matrix_c.ncols(), handle_c);
        Ok(handle_c)
    }

    fn get_matrix_dimensions(h: Handle) -> Result<MatrixDimensions, HostError> {
        println!("[Provider Wasm] Getting dimensions for handle {}", h);
        let state = HOST_STATE.lock().unwrap();
        match state.matrix_dims.get(&h) {
            Some(&(rows, cols)) => Ok(MatrixDimensions { rows, cols }),
            None => Err(HostError::InvalidHandle),
        }
    }
}


fn bytes_to_f32_slice(bytes: &[u8]) -> Option<&[f32]> {
    if bytes.as_ptr() as usize % std::mem::align_of::<f32>() != 0 { return None; } // Alignment check
    if bytes.len() % std::mem::size_of::<f32>() != 0 { return None; }
    unsafe {
        Some(std::slice::from_raw_parts(
            bytes.as_ptr() as *const f32,
            bytes.len() / std::mem::size_of::<f32>(),
        ))
    }
}

fn f32_slice_to_bytes(floats: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * std::mem::size_of::<f32>());
    for float_val in floats {
        bytes.extend_from_slice(&float_val.to_ne_bytes());
    }
    bytes
}




















impl keyvalue::store::Host for WasiKeyValue<'_> {
    fn open(&mut self, identifier: String) -> Result<Resource<Bucket>, Error> {
        match identifier.as_str() {
            "" => Ok(self.table.push(Bucket {
                in_memory_data: self.ctx.in_memory_data.clone(),
            })?),
            _ => Err(Error::NoSuchStore),
        }
    }

    fn convert_error(&mut self, err: Error) -> Result<keyvalue::store::Error> {
        match err {
            Error::NoSuchStore => Ok(keyvalue::store::Error::NoSuchStore),
            Error::AccessDenied => Ok(keyvalue::store::Error::AccessDenied),
            Error::Other(e) => Ok(keyvalue::store::Error::Other(e)),
        }
    }
}

impl keyvalue::store::HostBucket for WasiKeyValue<'_> {
    fn get(&mut self, bucket: Resource<Bucket>, key: String) -> Result<Option<Vec<u8>>, Error> {
        let bucket = self.table.get_mut(&bucket)?;
        Ok(bucket.in_memory_data.get(&key).cloned())
    }

    fn set(&mut self, bucket: Resource<Bucket>, key: String, value: Vec<u8>) -> Result<(), Error> {
        let bucket = self.table.get_mut(&bucket)?;
        bucket.in_memory_data.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, bucket: Resource<Bucket>, key: String) -> Result<(), Error> {
        let bucket = self.table.get_mut(&bucket)?;
        bucket.in_memory_data.remove(&key);
        Ok(())
    }

    fn exists(&mut self, bucket: Resource<Bucket>, key: String) -> Result<bool, Error> {
        let bucket = self.table.get_mut(&bucket)?;
        Ok(bucket.in_memory_data.contains_key(&key))
    }

    fn list_keys(
        &mut self,
        bucket: Resource<Bucket>,
        cursor: Option<u64>,
    ) -> Result<keyvalue::store::KeyResponse, Error> {
        let bucket = self.table.get_mut(&bucket)?;
        let keys: Vec<String> = bucket.in_memory_data.keys().cloned().collect();
        let cursor = cursor.unwrap_or(0) as usize;
        let keys_slice = &keys[cursor..];
        Ok(keyvalue::store::KeyResponse {
            keys: keys_slice.to_vec(),
            cursor: None,
        })
    }

    fn drop(&mut self, bucket: Resource<Bucket>) -> Result<()> {
        self.table.delete(bucket)?;
        Ok(())
    }
}

impl keyvalue::atomics::Host for WasiKeyValue<'_> {
    fn increment(
        &mut self,
        bucket: Resource<Bucket>,
        key: String,
        delta: u64,
    ) -> Result<u64, Error> {
        let bucket = self.table.get_mut(&bucket)?;
        let value = bucket
            .in_memory_data
            .entry(key.clone())
            .or_insert("0".to_string().into_bytes());
        let current_value = String::from_utf8(value.clone())
            .map_err(|e| Error::Other(e.to_string()))?
            .parse::<u64>()
            .map_err(|e| Error::Other(e.to_string()))?;
        let new_value = current_value + delta;
        *value = new_value.to_string().into_bytes();
        Ok(new_value)
    }
}

impl keyvalue::batch::Host for WasiKeyValue<'_> {
    fn get_many(
        &mut self,
        bucket: Resource<Bucket>,
        keys: Vec<String>,
    ) -> Result<Vec<Option<(String, Vec<u8>)>>, Error> {
        let bucket = self.table.get_mut(&bucket)?;
        Ok(keys
            .into_iter()
            .map(|key| {
                bucket
                    .in_memory_data
                    .get(&key)
                    .map(|value| (key.clone(), value.clone()))
            })
            .collect())
    }

    fn set_many(
        &mut self,
        bucket: Resource<Bucket>,
        key_values: Vec<(String, Vec<u8>)>,
    ) -> Result<(), Error> {
        let bucket = self.table.get_mut(&bucket)?;
        for (key, value) in key_values {
            bucket.in_memory_data.insert(key, value);
        }
        Ok(())
    }

    fn delete_many(&mut self, bucket: Resource<Bucket>, keys: Vec<String>) -> Result<(), Error> {
        let bucket = self.table.get_mut(&bucket)?;
        for key in keys {
            bucket.in_memory_data.remove(&key);
        }
        Ok(())
    }
}

/// Add all the `wasi-keyvalue` world's interfaces to a [`wasmtime::component::Linker`].
pub fn add_to_linker<T: Send + 'static>(
    l: &mut wasmtime::component::Linker<T>,
    f: fn(&mut T) -> WasiKeyValue<'_>,
) -> Result<()> {
    keyvalue::store::add_to_linker::<_, HasWasiKeyValue>(l, f)?;
    keyvalue::atomics::add_to_linker::<_, HasWasiKeyValue>(l, f)?;
    keyvalue::batch::add_to_linker::<_, HasWasiKeyValue>(l, f)?;
    Ok(())
}

struct HasWasiKeyValue;

impl HasData for HasWasiKeyValue {
    type Data<'a> = WasiKeyValue<'a>;
}
