use wasmtime::*;

fn infer_shared_mem_type(
    modules: Vec<Module>,
    host_module_name: &str,
    shared_memory_name: &str,
) -> Option<(u32, Option<u32>)> {
    // https://webassembly.github.io/threads/core/valid/types.html#import-subtyping

    let mut n1: u32 = 0; // smallest possible minimum
    let mut m1: Option<u32> = None; // largest possible maximum

    for module in modules {
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
        });
        let Some(mem_type) = mem_type else {
            return None;
        };
        n1 = n1.max(mem_type.minimum().try_into().unwrap());
        m1 = match m1 {
            Some(m1) => match mem_type.maximum() {
                Some(m2) => Some(m1.min(m2.try_into().unwrap())),
                None => Some(m1),
            },
            None => mem_type.maximum().map(|m2| m2.try_into().unwrap()),
        };
    }

    if let Some(m1) = m1 {
        if n1 > m1 {
            return None;
        }
    }

    Some((n1, m1))
}

pub fn run(
    host_module_name: &str,
    shared_memory_name: &str,
    main_function_name: &str,
    main_argc_value: i32,
    main_argv_value: i32,
    wasm_module_binaries: Vec<String>,
) -> Result<Vec<i32>> {
    let engine = Engine::default();
    let linker = Linker::new(&engine);

    let modules: Vec<Module> = wasm_module_binaries
        .iter()
        .map(|binary| Module::new(&engine, binary).expect(&format!("module is invalid")))
        .collect();

    let (shared_memory_min_size, Some(shared_memory_max_size)) =
        infer_shared_mem_type(modules, host_module_name, shared_memory_name).unwrap()
    else {
        panic!("cannot handle unspecified maximum memory size");
    };

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
