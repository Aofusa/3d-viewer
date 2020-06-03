use lucet_runtime::{DlModule, Limits, MmapRegion, Region};
use lucet_wasi::WasiCtxBuilder;

fn main() {
    // ensure the WASI symbols are exported from the final executable
    lucet_wasi::export_wasi_funcs();
    // load the compiled Lucet module
    let dl_module = DlModule::load("assets/plugin/hello/hello.so").unwrap();
    // create a new memory region with default limits on heap and stack size
    let region = MmapRegion::create(1, &Limits::default()).unwrap();
    // instantiate the module in the memory region
    let mut instance = region.new_instance(dl_module).unwrap();
    // prepare the WASI context, inheriting stdio handles from the host executable
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build().unwrap();
    instance.insert_embed_ctx(wasi_ctx);

    let a = 1;
    let b = 2;

    // run the WASI main function
    let res = instance.run("calc", &[a.into(), b.into()]).unwrap().unwrap_returned().as_u32();

    println!("calc: {}", res);
}

