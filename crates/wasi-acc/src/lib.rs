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
        world: "wasi:acc/wasiaccimports",
    });
}

use crate::generated::wasi::acc;

use self::generated::wasi::acc::host_allocator::{Handle, Host, HostError, MatrixDimensions};

use anyhow::Result;
use std::collections::HashMap;

use wasmtime::component::HasData;
// use wasmtime::component::{HasData, Resource, ResourceTable, ResourceTableError};

/// The host-side state for the `wasi-acc` implementation.
#[derive(Default)]
pub struct WasiAccCtx {
    buffers: HashMap<Handle, Vec<u8>>,
    matrix_dims: HashMap<Handle, (u32, u32)>,
    next_handle: Handle,
}

impl WasiAccCtx {
    /// Creates a new context with default parameters.
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            ..Default::default()
        }
    }

    fn new_handle(&mut self) -> Handle {
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }
}

/// A wrapper capturing the needed internal state for `wasi-acc`.
pub struct WasiAcc<'a> {
    /// The user-provided context.
    ctx: &'a mut WasiAccCtx,
}

impl<'a> WasiAcc<'a> {
    /// Create a new view into the `wasi-acc` state.
    pub fn new(ctx: &'a mut WasiAccCtx) -> Self {
        Self { ctx }
    }
}

// Implement the `Host` trait we just imported.
impl Host for WasiAcc<'_> {
    fn allocate_buffer(&mut self, size: u64) -> Result<Handle, HostError> {
        println!("[Host Impl] Allocating buffer of size {}", size);
        if size == 0 {
            return Err(HostError::Other("Cannot allocate zero-size buffer".to_string()));
        }
        let handle = self.ctx.new_handle();
        self.ctx.buffers.insert(handle, vec![0u8; size as usize]);
        Ok(handle)
    }

    fn free_buffer(&mut self, h: Handle) -> Result<(), HostError> {
        println!("[Host Impl] Freeing buffer {}", h);
        if self.ctx.buffers.remove(&h).is_some() {
            self.ctx.matrix_dims.remove(&h);
            Ok(())
        } else {
            Err(HostError::InvalidHandle)
        }
    }

    fn write_to_host(
        &mut self,
        guest_bytes: Vec<u8>,
        target_handle: Handle,
        target_offset: u64,
    ) -> Result<(), HostError> {
        println!(
            "[Host Impl] Writing {} bytes to handle {} at offset {}",
            guest_bytes.len(),
            target_handle,
            target_offset
        );
        match self.ctx.buffers.get_mut(&target_handle) {
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
        &mut self,
        source_handle: u32,
        source_offset: u64,
        len: u64,
    ) -> Result<Vec<u8>, HostError> {
        println!(
            "[Host Impl] Reading {} bytes from handle {} at offset {}",
            len, source_handle, source_offset
        );
        match self.ctx.buffers.get(&source_handle) {
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

    fn register_matrix_dimensions(
        &mut self,
        h: u32,
        dims: MatrixDimensions,
    ) -> Result<(), HostError> {
        println!(
            "[Host Impl] Registering dimensions {}x{} for handle {}",
            dims.rows, dims.cols, h
        );
        if !self.ctx.buffers.contains_key(&h) {
            return Err(HostError::InvalidHandle);
        }
        self.ctx.matrix_dims.insert(h, (dims.rows, dims.cols));
        Ok(())
    }

    fn get_matrix_dimensions(&mut self, h: u32) -> Result<MatrixDimensions, HostError> {
        println!("[Host Impl] Getting dimensions for handle {}", h);
        match self.ctx.matrix_dims.get(&h) {
            Some(&(rows, cols)) => Ok(MatrixDimensions { rows, cols }),
            None => Err(HostError::InvalidHandle),
        }
    }


    fn matrix_multiply_f32(
        &mut self,
        handle_a: u32,
        handle_b: u32,
    ) -> Result<u32, HostError> {
        println!("[Host Impl] Matrix multiply f32 for A:{} and B:{}", handle_a, handle_b);

        let result = (|| {
            let (rows_a, cols_a) = *self.ctx.matrix_dims.get(&handle_a).ok_or(HostError::InvalidHandle)?;
            let buffer_a_bytes = self.ctx.buffers.get(&handle_a).ok_or(HostError::InvalidHandle)?;
            let matrix_a_data = bytes_to_f32_slice(buffer_a_bytes)
                .ok_or_else(|| HostError::Other("Failed to cast buffer A to f32".to_string()))?;
            if matrix_a_data.len() != (rows_a * cols_a) as usize { return Err(HostError::Other("Buffer A size mismatch with dims".to_string())); }
            let matrix_a = nalgebra::DMatrix::<f32>::from_row_slice(rows_a as usize, cols_a as usize, matrix_a_data);
    
            let (rows_b, cols_b) = *self.ctx.matrix_dims.get(&handle_b).ok_or(HostError::InvalidHandle)?;
            let buffer_b_bytes = self.ctx.buffers.get(&handle_b).ok_or(HostError::InvalidHandle)?;
            let matrix_b_data = bytes_to_f32_slice(buffer_b_bytes)
                .ok_or_else(|| HostError::Other("Failed to cast buffer B to f32".to_string()))?;
            if matrix_b_data.len() != (rows_b * cols_b) as usize { return Err(HostError::Other("Buffer B size mismatch with dims".to_string())); }
            let matrix_b = nalgebra::DMatrix::<f32>::from_row_slice(rows_b as usize, cols_b as usize, matrix_b_data);
    
            if cols_a != rows_b {
                return Err(HostError::DimensionMismatch);
            }
    
            let matrix_c = matrix_a * matrix_b;
            let handle_c = self.ctx.new_handle();
            
            let mut row_major_data = Vec::with_capacity(matrix_c.len());
            for r in 0..matrix_c.nrows() {
                for c in 0..matrix_c.ncols() {
                    row_major_data.push(matrix_c[(r, c)]);
                }
            }
            let c_bytes = f32_slice_to_bytes(&row_major_data);

            self.ctx.buffers.insert(handle_c, c_bytes);
            self.ctx.matrix_dims.insert(handle_c, (matrix_c.nrows() as u32, matrix_c.ncols() as u32));
            println!("[Host Impl] Stored result C ({},{}) with handle {}", matrix_c.nrows(), matrix_c.ncols(), handle_c);
            Ok(handle_c)
        })();

        result
    }
}

struct HasWasiAcc;

impl HasData for HasWasiAcc {
    type Data<'a> = WasiAcc<'a>;
}

/// Add all the `wasi-acc` world's interfaces to a [`wasmtime::component::Linker`].
pub fn add_to_linker<T: Send + 'static>(
    l: &mut wasmtime::component::Linker<T>,
    f: fn(&mut T) -> WasiAcc<'_>,
) -> Result<()> {
    acc::host_allocator::add_to_linker::<_, HasWasiAcc>(l, f)
}


fn bytes_to_f32_slice(bytes: &[u8]) -> Option<&[f32]> {
    if bytes.as_ptr() as usize % std::mem::align_of::<f32>() != 0 {
        return None;
    }
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        return None;
    }
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

