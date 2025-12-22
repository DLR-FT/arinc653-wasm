use std::{ops::Deref, sync::Arc};

use anyhow::{Context, Result, bail};
use binrw::{BinRead, NullString};
use log::debug;

use crate::a653::{PartitionContext, find_function, init_process};

#[derive(Debug)]
pub struct ProcessHandle(std::thread::JoinHandle<()>);

impl Deref for ProcessHandle {
    type Target = std::thread::JoinHandle<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<std::thread::ThreadId> for ProcessHandle {
    fn eq(&self, other: &std::thread::ThreadId) -> bool {
        self.thread_id().eq(other)
    }
}

impl ProcessHandle {
    pub fn thread_id(&self) -> std::thread::ThreadId {
        self.thread().id()
    }

    pub fn into_inner(self) -> std::thread::JoinHandle<()> {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct ProcessTable(Vec<Process>);

impl ProcessTable {
    pub fn insert(&mut self, process: Process) -> Result<i64> {
        let name = process.name();
        if self.0.contains(&process) {
            bail!("Process({name}) already exists");
        }
        let pid = self.0.len();
        self.0.push(process);
        Ok(pid as i64)
    }

    pub(crate) fn get_from_pid_mut(&mut self, pid: i64) -> Option<&mut Process> {
        self.0.get_mut(pid as usize)
    }

    pub(crate) fn get_from_tid(&self, tid: &std::thread::ThreadId) -> Option<&Process> {
        self.0
            .iter()
            .find(|p| p.handle.as_ref().is_some_and(|h| h.eq(tid)))
    }

    pub(crate) fn spawn_all(&mut self, ctx: &Arc<PartitionContext>) -> Result<()> {
        let iter = self.0.iter_mut().filter(|p| p.enabled);
        for proc in iter {
            proc.spawn(ctx)?;
        }
        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Vec<Process> {
        std::mem::take(&mut self.0)
    }
}

#[derive(Debug)]
pub struct Process {
    attribute: ProcessAttribute,
    handle: Option<ProcessHandle>,
    enabled: bool,
}

impl Process {
    pub fn new(attr: ProcessAttribute) -> Self {
        Self {
            attribute: attr,
            handle: None,
            enabled: false,
        }
    }

    pub fn spawn(&mut self, ctx: &Arc<PartitionContext>) -> Result<()> {
        let name = self.attribute.name.to_string();
        if self.handle.is_some() {
            bail!("Process({name}) already spawned")
        }
        if !self.enabled {
            bail!("Process({name}) is not enabled")
        }

        let (mut instance, mut store) = init_process(ctx)?;
        let entry = self.attribute.entry_point as u64;
        let func = find_function(entry, &mut instance, &mut store)
            .context(format!("No Function with ID({entry})"))?;

        let handle = std::thread::Builder::new()
            .name(name.clone())
            .stack_size(self.attribute.stack_size as usize)
            .spawn(move || {
                let res = func.call(&mut store, &[], &mut []);
                debug!("[{name}] Process ended with: {res:?}")
            })?;
        let handle = ProcessHandle(handle);
        let thread_id = handle.thread_id();
        let name = &self.attribute.name;
        debug!("Spawned Process({name}) as Thread({thread_id:?})");
        _ = self.handle.insert(handle);

        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn name(&self) -> String {
        self.attribute.name.to_string()
    }

    pub fn into_handle(self) -> Option<ProcessHandle> {
        self.handle
    }
}

impl Deref for Process {
    type Target = ProcessAttribute;

    fn deref(&self) -> &Self::Target {
        &self.attribute
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        self.attribute
            .name
            .eq_ignore_ascii_case(&other.attribute.name)
    }
}

#[derive(BinRead, Debug, Clone, PartialEq)]
#[br(little)]
pub struct ProcessAttribute {
    pub period: i64,
    pub time_capacity: i64,
    pub entry_point: i32,
    pub stack_size: u32,
    pub base_priority: i32,
    pub deadline: i32,
    pub name: NullString,
}

// size of processattribute + MAX_NAME_LENGTH(32)
const PROC_ATTR_BUFFER_SIZE: usize = std::mem::size_of::<ProcessAttribute>() + 32;

pub type ProcAttrBuffer = [u8; PROC_ATTR_BUFFER_SIZE];
