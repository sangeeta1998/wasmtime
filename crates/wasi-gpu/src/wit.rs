wasmtime::component::bindgen!({
    world: "compute",
    path: "wit/wasi-gpu.wit",
    with: {
        "wasi:gpu/device/gpu-device": crate::GpuDevice,
        "wasi:gpu/program/gpu-program": crate::GpuProgram,
    }
});

pub use bindings::wasi::gpu::*;
impl wit::device::Host for GpuCtx {
    fn list_devices(&mut self) -> Result<Vec<Resource<GpuDevice>>> {
        println!(">>>> list_devices called");
        Ok(vec![self.resource_table.push(GpuDevice { id: 0, name: "FakeGPU".into() })?])
    }
}
