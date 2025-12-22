use std::ffi::CStr;

use binrw::{BinRead, BinReaderExt, NullString};
use wasmtime::*;

fn infer_shared_mem_type(
    modules: &Vec<Module>,
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
        })?;
        // always fits in u32 according to wasmtime docs
        n1 = n1.max(mem_type.minimum().try_into().unwrap());
        m1 = match m1 {
            Some(m1) => match mem_type.maximum() {
                // always fits in u32 according to wasmtime docs
                Some(m2) => Some(m1.min(m2.try_into().unwrap())),
                None => Some(m1),
            },
            // always fits in u32 according to wasmtime docs
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

#[derive(BinRead, Debug)]
#[br(little)]
struct ProcessAttribute {
    period: i64,
    time_capacity: i64,
    entry_point: i32,
    stack_size: u32,
    base_priority: i32,
    deadline: i32,
    name: NullString,
}

// size of processattribute + MAX_NAME_LENGTH(32)
const BUFFER_SIZE: usize = std::mem::size_of::<ProcessAttribute>() + 32;

type Buffer = [u8; BUFFER_SIZE];

pub fn create_process(mut caller: Caller<'_, ()>, x: i32, y: i32, z: i32) {
    let mem = caller.get_export("memory").unwrap();
    let mem = mem.into_shared_memory().unwrap();
    let attr = unsafe { mem.data().as_ptr().byte_add(x as usize) };
    let buffer: Buffer = unsafe { std::ptr::read_volatile(attr as _) };
    let mut reader = binrw::io::Cursor::new(buffer);
    let attr: ProcessAttribute = reader.read_ne().unwrap();

    println!("attr: {attr:?}, x: {x}, y: {y}, z: {z}");
    todo!()
}

pub fn run(
    host_module_name: &str,
    shared_memory_name: &str,
    proc_alloc_name: &str,
    main_function_name: &str,
    main_argc_value: i32,
    main_argv_value: i32,
    wasm_module_paths: &[String],
) -> Vec<i32> {
    let engine = Engine::default();
    let linker = Linker::new(&engine);

    let modules: Vec<Module> = wasm_module_paths
        .iter()
        .map(|path| Module::from_file(&engine, path).unwrap())
        .collect();

    let (shared_memory_min_size, Some(shared_memory_max_size)) =
        infer_shared_mem_type(&modules, host_module_name, shared_memory_name)
            .expect("the modules do not agree on a common shared memory type")
    else {
        panic!("wasmtime cannot handle unspecified maximum shared memory size");
    };

    let shared_mem = SharedMemory::new(
        &engine,
        MemoryType::shared(shared_memory_min_size, shared_memory_max_size),
    )
    .expect("shared memory could not be instantiated");

    let threads = modules.into_iter().map(|module| {
        let engine = engine.clone();
        let mut linker = linker.clone();
        let shared_mem = shared_mem.clone();
        let host_module_name = host_module_name.to_owned();
        let shared_memory_name = shared_memory_name.to_owned();
        let main_function_name = main_function_name.to_owned();
        let proc_alloc_name = proc_alloc_name.to_owned();
        std::thread::spawn(move || {
            let mut store = Store::new(&engine, ());
            linker
                .define(&store, &host_module_name, &shared_memory_name, shared_mem)
                .expect("shared memory could not be linked in");
            linker
                .func_wrap("arinc653:p1@0.1.0", "CREATE_PROCESS", create_process)
                .expect("could not add create process host function");
            let instance = linker
                .instantiate(&mut store, &module)
                .expect("module could not be instantiated");
            let proc_alloc = instance
                .get_typed_func::<(), (i32,)>(&mut store, &proc_alloc_name)
                .expect("module::proc_alloc could not be found");
            let proc_alloc_result = proc_alloc
                .call(&mut store, ())
                .expect("module::proc_alloc had a trap");
            if proc_alloc_result.0 != 1 {
                panic!("module::proc_alloc result not 1");
            }
            let main = instance
                .get_typed_func::<(i32, i32), (i32,)>(&mut store, &main_function_name)
                .expect("module::main could not be found");
            main.call(&mut store, (main_argc_value, main_argv_value))
                .expect("module::main had a trap")
        })
    });

    threads.map(|thread| thread.join().unwrap().0).collect()
}
