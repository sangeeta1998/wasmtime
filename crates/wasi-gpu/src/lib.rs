mod wit;

use wasmtime::component::*;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Default)]
pub struct GpuCtx {
    pub devices: Vec<GpuDevice>,
    pub programs: HashMap<u32, String>,
}

pub struct GpuDevice {
    pub id: u32,
    pub name: String,
}

pub struct GpuProgram {
    pub source: String,
}

pub fn add_to_linker<T: Send + Sync + 'static>(
    linker: &mut wasmtime::component::Linker<T>,
    get_ctx: impl Fn(&mut T) -> &mut GpuCtx + Send + Sync + Copy + 'static,
) -> Result<()> {
    wit::device::add_to_linker(linker, get_ctx)?;
    wit::program::add_to_linker(linker, get_ctx)?;
    wit::context::add_to_linker(linker, get_ctx)?;
    Ok(())
}
