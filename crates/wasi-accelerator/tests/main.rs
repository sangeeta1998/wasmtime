use anyhow::{Result, anyhow};
use test_programs_artifacts::{ACCELERATOR_MAIN_COMPONENT, foreach_accelerator};
use wasmtime::{
    Store,
    component::{Component, Linker, ResourceTable},
};

use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView, p2::bindings::Command};
use wasmtime_wasi_accelerator::{WasiAccelerator, WasiAcceleratorCtx, WasiAcceleratorCtxBuilder};

struct Ctx {
    table: ResourceTable,
    wasi_ctx: WasiCtx,
    wasi_accelerator_ctx: WasiAcceleratorCtx,
}

impl WasiView for Ctx {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.table,
        }
    }
}


async fn run_wasi(path: &str, ctx: Ctx) -> Result<()> {
    let engine = test_programs_artifacts::engine(|config| {
        config.async_support(true);
    });
    let mut store = Store::new(&engine, ctx);
    let component = Component::from_file(&engine, path)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_accelerator::add_to_linker(&mut linker, |h: &mut Ctx| {
        WasiAccelerator::new(& mut h.wasi_accelerator_ctx)
    })?;

    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    command
        .wasi_cli_run()
        .call_run(&mut store)
        .await?
        .map_err(|()| anyhow!("command returned with failing exit status"))
}

macro_rules! assert_test_exists {
    ($name:ident) => {
        #[expect(unused_imports, reason = "just here to assert it exists")]
        use self::$name as _;
    };
}

foreach_accelerator!(assert_test_exists);

#[tokio::test(flavor = "multi_thread")]
async fn accelerator_main() -> Result<()> {
    run_wasi(
        ACCELERATOR_MAIN_COMPONENT,
        Ctx {         
            table: ResourceTable::new(),   
            wasi_ctx: WasiCtx::builder().inherit_stderr().build(),
            wasi_accelerator_ctx: WasiAcceleratorCtxBuilder::new()
                .build(),
        },
    )
    .await
}
