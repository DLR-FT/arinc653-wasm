use arinc653_wasm_harness::run;

use anyhow::Result;
use arinc653_wasm_harness::config::Config;
use clap::Parser;

fn main() -> Result<()> {
    colog::default_builder()
        .filter_module("wasmtime", log::LevelFilter::Warn)
        .filter_module("cranelift_codegen", log::LevelFilter::Warn)
        .filter_module("cranelift_frontend", log::LevelFilter::Warn)
        .init();

    let config = Config::parse();
    log::debug!("{config:#?}");
    let result = run(&config);

    let wasm_module_path = config.wasm_config.wasm_module_path;
    println!("{wasm_module_path} returned: {result:?}");
    Ok(())
}
