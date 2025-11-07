use example_wasm_runtime::run;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, default_value = "env")]
    pub host_module_name: String,
    #[arg(long, default_value = "memory")]
    pub shared_memory_name: String,
    #[arg(long, default_value = "__apex_wasm_proc_alloc")]
    pub proc_alloc_name: String,
    #[arg(long, default_value = "main")]
    pub main_function_name: String,
    #[arg(long, default_value = "0")]
    pub main_argc_value: i32,
    #[arg(long, default_value = "0")]
    pub main_argv_value: i32,
    #[arg(required = true)]
    pub wasm_module_paths: Vec<String>,
}

fn main() {
    let Cli {
        host_module_name,
        shared_memory_name,
        proc_alloc_name,
        main_function_name,
        main_argc_value,
        main_argv_value,
        wasm_module_paths,
    } = Cli::parse();
    let results = run(
        &host_module_name,
        &shared_memory_name,
        &proc_alloc_name,
        &main_function_name,
        main_argc_value,
        main_argv_value,
        &wasm_module_paths,
    );

    for (result, wasm_module_path) in results.iter().zip(wasm_module_paths) {
        println!("{wasm_module_path} returned: {result}")
    }
}
