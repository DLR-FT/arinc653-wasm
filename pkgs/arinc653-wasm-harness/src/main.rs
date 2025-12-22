use arinc653_wasm_harness::run;

use anyhow::Result;
use arinc653_wasm_harness::config::Cli;
use clap::Parser;

fn main() -> Result<()> {
    colog::default_builder()
        .filter_module("wasmtime", log::LevelFilter::Warn)
        .filter_module("cranelift_codegen", log::LevelFilter::Warn)
        .filter_module("cranelift_frontend", log::LevelFilter::Warn)
        .init();

    let cli = Cli::parse();
    log::debug!("{cli:#?}");
    let result = run(&cli.config_delegate);

    let wasm_module_path = cli.config_delegate.wasm_config.wasm_module_path;
    println!("{wasm_module_path} returned: {result:?}");
    Ok(())
}
