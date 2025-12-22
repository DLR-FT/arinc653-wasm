use std::sync::{Arc, Condvar, Mutex, RwLock};

use a653::{PartitionContext, init_process};
use a653rs::prelude::OperatingMode;
use config::WasmConfig;
use log::debug;
use process::ProcessTable;
use wasmtime::*;

mod a653;
mod api;
pub mod channel;
pub mod config;
pub mod process;

fn infer_shared_mem_type(
    module: &Module,
    host_module_name: &str,
    shared_memory_name: &str,
) -> Option<(u32, Option<u32>)> {
    // https://webassembly.github.io/threads/core/valid/types.html#import-subtyping

    let mem_type = module.imports().find_map(|import| {
        if import.module() != host_module_name || import.name() != shared_memory_name {
            return None;
        }
        match import.ty() {
            ExternType::Memory(memory_type) => {
                if memory_type.is_shared() {
                    Some(memory_type)
                } else {
                    None
                }
            }
            _ => None,
        }
    })?;

    // always fits in u32 according to wasmtime docs
    let n1 = mem_type.minimum().try_into().unwrap();
    let m1 = mem_type.maximum().map(|m2| m2.try_into().unwrap());

    Some((n1, m1))
}

pub fn run(config: &config::Config) -> Result<()> {
    let WasmConfig {
        host_module_name,
        shared_memory_name,
        main_function_name,
        main_argc_value,
        main_argv_value,
        wasm_module_path,
        ..
    } = &config.wasm_config;
    let engine = Engine::default();
    let module = Module::from_file(&engine, wasm_module_path).unwrap();

    let (shared_memory_min_size, Some(shared_memory_max_size)) =
        infer_shared_mem_type(&module, host_module_name, shared_memory_name)
            .expect("the modules does not define a shared memory type")
    else {
        panic!("wasmtime cannot handle unspecified maximum shared memory size");
    };

    let shared_memory = SharedMemory::new(
        &engine,
        MemoryType::shared(shared_memory_min_size, shared_memory_max_size),
    )
    .expect("shared memory could not be instantiated");

    let main_function_name = main_function_name.to_owned();

    let ctx = Arc::new(PartitionContext {
        config: config.clone(),
        module,
        shared_memory,
        processes: RwLock::new(ProcessTable::default()),
        mode: RwLock::new(OperatingMode::ColdStart),
        shutdown_signal: Condvar::new(),
        shutdown_lock: Mutex::new(false),
        sampling_ports: RwLock::default(),
    });

    let (instance, mut store) = init_process(&ctx).unwrap();

    let main = instance
        .get_typed_func::<(i32, i32), (i32,)>(&mut store, &main_function_name)
        .expect("module::main could not be found");
    main.call(&mut store, (*main_argc_value, *main_argv_value))?;

    {
        let mut shutdown = ctx.shutdown_lock.lock().unwrap();
        *shutdown = true;
        ctx.shutdown_signal.notify_all();
    }

    let procs = ctx.processes.write().unwrap().clear();
    for (pid, proc) in procs.into_iter().enumerate() {
        if let Some(handle) = proc.into_handle() {
            debug!("Joining PID({pid})");
            handle.into_inner().join().ok();
        }
    }

    Ok(())
}
