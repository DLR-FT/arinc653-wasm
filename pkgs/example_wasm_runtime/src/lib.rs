use wasmtime::*;
pub fn run(
    host_module_name: &str,
    shared_memory_name: &str,
    shared_memory_min_size: u32,
    shared_memory_max_size: u32,
    main_function_name: &str,
    main_argc_value: i32,
    main_argv_value: i32,
    wasm_module_binaries: Vec<String>,
) -> Result<Vec<i32>> {
    let engine = Engine::default();
    let linker = Linker::new(&engine);
    let shared_mem = SharedMemory::new(
        &engine,
        MemoryType::shared(shared_memory_min_size, shared_memory_max_size),
    )?;

    let threads = wasm_module_binaries.iter().map(|binary| {
        let module = Module::new(&engine, binary).expect(&format!("module is invalid"));
        let engine = engine.clone();
        let mut linker = linker.clone();
        let shared_mem = shared_mem.clone();
        let host_module_name = host_module_name.to_owned();
        let shared_memory_name = shared_memory_name.to_owned();
        let main_function_name = main_function_name.to_owned();
        std::thread::spawn(move || {
            let mut store = Store::new(&engine, ());
            linker.define(&store, &host_module_name, &shared_memory_name, shared_mem)?;
            let instance = linker.instantiate(&mut store, &module)?;
            let run =
                instance.get_typed_func::<(i32, i32), (i32,)>(&mut store, &main_function_name)?;
            run.call(&mut store, (main_argc_value, main_argv_value))
        })
    });

    let results: Vec<i32> = threads
        .map(|thread| thread.join().unwrap().unwrap().0)
        .collect();
    Ok(results)
}
