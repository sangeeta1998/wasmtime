use anyhow::{Result, anyhow};
use test_programs_artifacts::{ACC_MAIN_COMPONENT, foreach_acc};
use wasmtime::{
    Store,
    component::{Component, Linker},
};
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView, bindings::Command};
use wasmtime_wasi_acc::{WasiAcc, WasiAccCtx, WasiAccCtxBuilder};

struct Ctx {
    wasi_ctx: WasiCtx,
    wasi_acc_ctx: WasiAccCtx,
}

impl WasiView for Ctx {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
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
    wasmtime_wasi_acc::add_to_linker(&mut linker, |h: &mut Ctx| {
        WasiAcc::new(&h.wasi_acc_ctx)
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

foreach_keyvalue!(assert_test_exists);

#[tokio::test(flavor = "multi_thread")]
async fn keyvalue_main() -> Result<()> {
    run_wasi(
        KEYVALUE_MAIN_COMPONENT,
        Ctx {            
            wasi_ctx: WasiCtxBuilder::new().inherit_stderr().build(),
            wasi_acc_ctx: WasiAccCtxBuilder::new()
                .in_memory_data([("atomics_key", "5")])
                .build(),
        },
    )
    .await
}
