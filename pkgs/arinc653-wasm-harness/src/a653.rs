use std::{
    cell::UnsafeCell,
    sync::{Arc, RwLock},
    time::Duration,
};

use a653rs::{
    bindings::PortDirection,
    prelude::{ErrorCode, OperatingMode, Validity},
};
use anyhow::{Context, Result};
use binrw::BinReaderExt;
use log::{debug, error, info, trace};
use wasmtime::{
    AsContext, AsContextMut, Caller, Func, Instance, Linker, Module, SharedMemory, Store,
};

use crate::{
    channel::{SamplingMessage, SamplingPort, SamplingPortTable},
    config::Config,
    process::{ProcAttrBuffer, Process, ProcessAttribute, ProcessTable},
};

pub struct PartitionContext {
    pub config: Config,
    pub module: Module,
    pub shared_memory: SharedMemory,
    pub sampling_ports: RwLock<SamplingPortTable>,
    pub processes: RwLock<ProcessTable>,
    pub mode: RwLock<OperatingMode>,
    pub shutdown_signal: std::sync::Condvar,
    pub shutdown_lock: std::sync::Mutex<bool>,
}

impl PartitionContext {
    fn get_current_process_name(&self) -> String {
        let id = std::thread::current().id();
        self.processes
            .read()
            .unwrap()
            .get_from_tid(&id)
            .map_or_else(|| String::from("main"), Process::name)
    }
}

pub trait ArincProvider {
    fn create_process(&self, attributes: ProcessAttribute) -> Result<i64>;
    fn report_application_message(&self, msg: &str) -> Result<()>;
    fn raise_application_error(&self, err: ErrorCode, msg: &str) -> Result<()>;
    fn start(&self, pid: i64) -> Result<()>;
    fn set_partition_mode(&self, mode: OperatingMode) -> Result<()>;
    fn create_sampling_port(
        &self,
        name: &str,
        max_size: usize,
        direction: PortDirection,
        refresh: Duration,
    ) -> Result<i64>;
    fn write_sampling_message(&self, sid: i64, msg: &[u8]) -> Result<()>;
    fn read_sampling_message(&self, sid: i64) -> Result<(SamplingMessage, Validity)>;
}

impl ArincProvider for Arc<PartitionContext> {
    fn create_process(&self, attributes: ProcessAttribute) -> Result<i64> {
        let running_proc_name = self.get_current_process_name();
        let mut table = self.processes.write().unwrap();
        let name = attributes.name.clone();
        let process = Process::new(attributes);
        let pid = table.insert(process).unwrap();
        debug!("[{running_proc_name}] Assigned ARINC653 PID({pid}) to Process({name})",);
        Ok(pid)
    }

    fn report_application_message(&self, msg: &str) -> Result<()> {
        let name = self.get_current_process_name();
        info!("[{name}] \"{msg}\"");
        Ok(())
    }

    fn raise_application_error(&self, err: ErrorCode, msg: &str) -> Result<()> {
        let name = self.get_current_process_name();
        error!("[{name}] ApplicationError({err:?}): {msg}");
        Ok(())
    }
    fn start(&self, pid: i64) -> Result<()> {
        let mut table = self.processes.write().unwrap();
        table.get_from_pid_mut(pid).unwrap().enable();
        Ok(())
    }

    fn set_partition_mode(&self, mode: OperatingMode) -> Result<()> {
        if let OperatingMode::Normal = mode {
            {
                let mut table = self.processes.write().unwrap();
                table.spawn_all(self)?;
            }
            {
                let mut mode = self.mode.write().unwrap();
                *mode = OperatingMode::Normal;
            }
            let mut shutdown = self.shutdown_lock.lock().unwrap();
            while !*shutdown {
                shutdown = self.shutdown_signal.wait(shutdown).unwrap();
            }
        }
        Ok(())
    }

    fn create_sampling_port(
        &self,
        name: &str,
        max_size: usize,
        direction: PortDirection,
        refresh: Duration,
    ) -> Result<i64> {
        let channel = self
            .config
            .arinc_config
            .sampling_ports
            .iter()
            .find(|port| port.name.eq_ignore_ascii_case(name))
            .context(format!("SamplingPort({name}) is not defined"))?;
        let port = SamplingPort::new(channel.clone(), direction, max_size, refresh)?;
        let mut ports = self.sampling_ports.write().unwrap();
        let sid = ports.insert(port)?;
        Ok(sid)
    }

    fn write_sampling_message(&self, sid: i64, msg: &[u8]) -> Result<()> {
        let mut ports = self.sampling_ports.write().unwrap();
        let port = ports
            .get_port_mut(sid)
            .context(format!("No SamplingPort({sid}) created"))?;
        port.write(msg)
    }

    fn read_sampling_message(&self, sid: i64) -> Result<(SamplingMessage, Validity)> {
        let mut ports = self.sampling_ports.write().unwrap();
        let port = ports
            .get_port_mut(sid)
            .context(format!("No SamplingPort({sid}) created"))?;
        let msg = port.read()?;
        let val = msg.validity(port.refresh());
        Ok((msg, val))
    }
}

pub trait CallerExt {
    fn extract_shmem(&mut self) -> Result<SharedMemory>;
    fn name(&self) -> String;
}

impl CallerExt for Caller<'_, Arc<PartitionContext>> {
    fn extract_shmem(&mut self) -> Result<SharedMemory> {
        let mem = self.get_export("memory").context("No Shmem found")?;
        mem.into_shared_memory().context("No Shmem found")
    }

    fn name(&self) -> String {
        self.data().get_current_process_name()
    }
}

pub trait ShmemExt {
    fn extract_proc_attrs(&self, attr_ptr: i32) -> Result<ProcessAttribute>;
    fn extract_unsafe_cell_byte_slice(&self, ptr: i32, len: usize) -> Result<&[UnsafeCell<u8>]>;
    fn extract_byte_slice(&self, ptr: i32, len: usize) -> Result<Vec<u8>>;
    fn extract_str_slice(&self, ptr: i32, len: usize) -> Result<String>;
    fn write_byte_slice(&self, ptr: i32, val: &[u8]) -> Result<()>;
    fn write_i64(&self, ptr: i32, val: i64) -> Result<()>;
    fn write_i32(&self, ptr: i32, val: i32) -> Result<()>;
    fn write_u8(&self, ptr: i32, val: u8) -> Result<()>;
}

impl ShmemExt for SharedMemory {
    fn extract_proc_attrs(&self, attr_ptr: i32) -> Result<ProcessAttribute> {
        let attr_ptr = unsafe { self.data().as_ptr().byte_add(attr_ptr as usize) };
        let attr_buffer: ProcAttrBuffer = unsafe { std::ptr::read_volatile(attr_ptr as _) };
        let mut attr_reader = binrw::io::Cursor::new(attr_buffer);
        let attr: ProcessAttribute = attr_reader.read_le().unwrap();
        Ok(attr)
    }

    fn extract_unsafe_cell_byte_slice(&self, ptr: i32, len: usize) -> Result<&[UnsafeCell<u8>]> {
        let ptr = ptr as usize;
        let shmem_len = self.data_size();
        self.data().get(ptr..(ptr + len)).context(format!(
            "Shmem out of range: ptr({ptr}), len({len}), shmem_len({shmem_len})"
        ))
    }

    fn extract_byte_slice(&self, ptr: i32, len: usize) -> Result<Vec<u8>> {
        let slice = self.extract_unsafe_cell_byte_slice(ptr, len)?;
        let slice_buffer = slice
            .iter()
            .map(|byte| unsafe { std::ptr::read_volatile(byte.get()) })
            .collect();
        Ok(slice_buffer)
    }

    fn extract_str_slice(&self, ptr: i32, len: usize) -> Result<String> {
        let bytes = self.extract_byte_slice(ptr, len)?;
        let str_bytes = bytes.split(|&b| b == 0).next().unwrap_or(&bytes);
        Ok(std::str::from_utf8(str_bytes)?.to_string())
    }

    fn write_byte_slice(&self, ptr: i32, val: &[u8]) -> Result<()> {
        let slice = self.extract_unsafe_cell_byte_slice(ptr, val.len())?;
        for (dst, src) in slice.iter().zip(val) {
            unsafe { std::ptr::write_volatile(dst.get(), *src) };
        }
        Ok(())
    }

    fn write_i64(&self, ptr: i32, val: i64) -> Result<()> {
        let slice = self.extract_unsafe_cell_byte_slice(ptr, std::mem::size_of::<i64>())?;
        let val = val.to_le_bytes();
        assert_eq!(slice.len(), val.len());
        for (src, dst) in val.into_iter().zip(slice) {
            unsafe { std::ptr::write_volatile(dst.get(), src) };
        }
        Ok(())
    }

    fn write_i32(&self, ptr: i32, val: i32) -> Result<()> {
        let slice = self.extract_unsafe_cell_byte_slice(ptr, std::mem::size_of::<i32>())?;
        let val = val.to_le_bytes();
        assert_eq!(slice.len(), val.len());
        for (src, dst) in val.into_iter().zip(slice) {
            unsafe { std::ptr::write_volatile(dst.get(), src) };
        }
        Ok(())
    }

    fn write_u8(&self, ptr: i32, val: u8) -> Result<()> {
        let slice = self.extract_unsafe_cell_byte_slice(ptr, std::mem::size_of::<u8>())?;
        let val = val.to_le_bytes();
        assert_eq!(slice.len(), val.len());
        for (src, dst) in val.into_iter().zip(slice) {
            unsafe { std::ptr::write_volatile(dst.get(), src) };
        }
        Ok(())
    }
}

pub fn host_create_process(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    attr_ptr: i32,
    pid_ptr: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] CREATE_PROCESS({attr_ptr:#x}, {pid_ptr:#x}, {ret_ptr:#x})");

    let mem = caller.extract_shmem()?;
    let provider = caller.data();
    let attr = mem.extract_proc_attrs(attr_ptr)?;
    debug!("[{name}] Got {attr:?}");
    let pid = provider.create_process(attr)?;

    mem.write_i64(pid_ptr, pid)?;
    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;

    Ok(())
}

pub fn host_report_application_message(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    msg_ptr: i32,
    len: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] REPORT_APPLICATION_MESSAGE({msg_ptr:#x}, {len}, {ret_ptr:#x})");
    let mem = caller.extract_shmem()?;
    let provider = caller.data();
    let msg = mem.extract_str_slice(msg_ptr, len as usize)?;
    provider.report_application_message(&msg)?;

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;
    Ok(())
}

pub fn host_raise_application_error(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    error: i32,
    msg_ptr: i32,
    len: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] RAISE_APPLICATION_MESSAGE({error}, {msg_ptr:#x}, {len}, {ret_ptr:#x})");
    let mem = caller.extract_shmem()?;
    let provider = caller.data();
    let msg = mem.extract_str_slice(msg_ptr, len as usize)?;
    let error = ErrorCode::from_repr(error as u32);
    provider.raise_application_error(error.unwrap(), &msg)?;

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;
    Ok(())
}

pub fn host_start(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    process_id: i64,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] START({process_id}, {ret_ptr:#x})");
    let mem = caller.extract_shmem()?;
    let provider = caller.data();
    provider.start(process_id)?;

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;
    Ok(())
}

pub fn host_set_partition_mode(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    operating_mode: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] SET_PARTITION_MODE({operating_mode}, {ret_ptr:#x})");
    let mem = caller.extract_shmem()?;
    let provider = caller.data();
    let mode = OperatingMode::from_repr(operating_mode as u32);
    provider.set_partition_mode(mode.unwrap())?;

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;
    Ok(())
}

pub fn host_periodic_wait(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] PERIODIC_WAIT({ret_ptr:#x})");
    let mem = caller.extract_shmem()?;
    debug!("[{name}] PERIODIC_WAIT is a noop");

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;
    Ok(())
}

pub fn host_create_sampling_port(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    name_ptr: i32,
    max_msg_size: i32,
    direction: i32,
    refresh: i64,
    sid_ptr: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!(
        "[{name}] CREATE_SAMPLING_PORT({name_ptr:#x}, {max_msg_size}, {direction}, {refresh}, {sid_ptr:#x}, {ret_ptr:#x})"
    );

    let mem = caller.extract_shmem()?;
    let provider = caller.data();

    let name = mem.extract_str_slice(name_ptr, 32)?;
    let refresh = Duration::from_nanos(refresh as u64);
    let dir = PortDirection::from_repr(direction as u32).unwrap();
    let sid = provider.create_sampling_port(&name, max_msg_size as usize, dir, refresh)?;

    mem.write_i64(sid_ptr, sid)?;
    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;

    Ok(())
}

pub fn host_write_sampling_message(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    sid: i64,
    msg_ptr: i32,
    len: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!("[{name}] WRITE_SAMPLING_MESSAGE({sid}, {msg_ptr:#x}, {len}, {ret_ptr:#x})");

    let mem = caller.extract_shmem()?;
    let provider = caller.data();

    let bytes = mem.extract_byte_slice(msg_ptr, len as usize)?;
    provider.write_sampling_message(sid, &bytes)?;

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;

    Ok(())
}

pub fn host_read_sampling_message(
    mut caller: Caller<'_, Arc<PartitionContext>>,
    sid: i64,
    msg_ptr: i32,
    len_ptr: i32,
    val_ptr: i32,
    ret_ptr: i32,
) -> anyhow::Result<()> {
    let name = caller.name();
    trace!(
        "[{name}] READ_SAMPLING_MESSAGE({sid}, {msg_ptr:#x}, {len_ptr:#x}, {val_ptr:#x}, {ret_ptr:#x})"
    );

    let mem = caller.extract_shmem()?;
    let provider = caller.data();

    let (msg, val) = provider.read_sampling_message(sid)?;
    mem.write_byte_slice(msg_ptr, msg.msg())?;
    mem.write_i32(val_ptr, val as i32)?;
    mem.write_i32(len_ptr, msg.msg().len() as i32)?;

    // TODO return correct return value
    mem.write_u8(ret_ptr, 0)?;

    Ok(())
}

pub fn register_arinc_functions(linker: &mut Linker<Arc<PartitionContext>>) -> Result<()> {
    linker.func_wrap("arinc653:p1@0.1.0", "CREATE_PROCESS", host_create_process)?;
    linker.func_wrap(
        "arinc653:p1@0.1.0",
        "REPORT_APPLICATION_MESSAGE",
        host_report_application_message,
    )?;
    linker.func_wrap("arinc653:p1@0.1.0", "START", host_start)?;
    linker.func_wrap(
        "arinc653:p1@0.1.0",
        "SET_PARTITION_MODE",
        host_set_partition_mode,
    )?;
    linker.func_wrap(
        "arinc653:p1@0.1.0",
        "RAISE_APPLICATION_ERROR",
        host_raise_application_error,
    )?;
    linker.func_wrap("arinc653:p1@0.1.0", "PERIODIC_WAIT", host_periodic_wait)?;
    linker.func_wrap(
        "arinc653:p1@0.1.0",
        "CREATE_SAMPLING_PORT",
        host_create_sampling_port,
    )?;
    linker.func_wrap(
        "arinc653:p1@0.1.0",
        "WRITE_SAMPLING_MESSAGE",
        host_write_sampling_message,
    )?;
    linker.func_wrap(
        "arinc653:p1@0.1.0",
        "READ_SAMPLING_MESSAGE",
        host_read_sampling_message,
    )?;
    Ok(())
}

pub fn register_shared_memory(
    linker: &mut Linker<Arc<PartitionContext>>,
    store: impl AsContext<Data = Arc<PartitionContext>>,
) -> Result<()> {
    let wasm_config = &store.as_context().data().config.wasm_config;
    linker.define(
        &store,
        &wasm_config.host_module_name,
        &wasm_config.shared_memory_name,
        store.as_context().data().shared_memory.clone(),
    )?;
    Ok(())
}

pub fn instantiate_process(
    linker: &mut Linker<Arc<PartitionContext>>,
    mut store: impl AsContextMut<Data = Arc<PartitionContext>>,
) -> Result<Instance> {
    let ctx = store.as_context().data().clone();
    let instance = linker
        .instantiate(&mut store, &ctx.module)
        .expect("module could not be instantiated");
    let proc_alloc = instance
        .get_typed_func::<(), (i32,)>(&mut store, &ctx.config.wasm_config.proc_alloc_name)
        .expect("module::proc_alloc could not be found");
    let proc_alloc_result = proc_alloc
        .call(&mut store, ())
        .expect("module::proc_alloc had a trap");
    if proc_alloc_result.0 != 1 {
        panic!("module::proc_alloc result not 1");
    }
    Ok(instance)
}

pub fn init_process(
    ctx: &Arc<PartitionContext>,
) -> Result<(Instance, Store<Arc<PartitionContext>>)> {
    let mut store = Store::new(ctx.module.engine(), ctx.clone());
    let mut linker = Linker::new(ctx.module.engine());
    register_shared_memory(&mut linker, &store)?;
    register_arinc_functions(&mut linker)?;
    let instance = instantiate_process(&mut linker, &mut store)?;

    Ok((instance, store))
}

pub fn find_function(
    func_ref: u64,
    instance: &mut Instance,
    mut store: impl AsContextMut<Data = Arc<PartitionContext>>,
) -> Option<Func> {
    instance
        .get_export(&mut store, "__indirect_function_table")?
        .into_table()?
        .get(store.as_context_mut(), func_ref)?
        .unwrap_func()
        .copied()
}
