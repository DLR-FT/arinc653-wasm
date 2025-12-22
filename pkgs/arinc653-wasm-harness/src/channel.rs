use std::net::{ToSocketAddrs, UdpSocket};
use std::ops::Deref;
use std::time::{Duration, Instant};

use a653rs::bindings::PortDirection;
use a653rs::prelude::Validity;
use anyhow::{Result, bail};

use crate::config::Port;

#[derive(Default)]
pub struct SamplingPortTable(Vec<SamplingPort>);

impl SamplingPortTable {
    pub const fn new() -> Self {
        SamplingPortTable(Vec::new())
    }

    pub fn insert(&mut self, port: SamplingPort) -> Result<i64> {
        let name = &port.name;
        if self.0.contains(&port) {
            bail!("SamplingPort({name}) already exists");
        }
        let cid = self.0.len();
        self.0.push(port);
        Ok(cid as i64)
    }

    pub fn get_port_mut(&mut self, id: i64) -> Option<&mut SamplingPort> {
        self.0.get_mut(id as usize)
    }
}

#[derive(Debug)]
pub struct SamplingPort {
    config: Port,
    port: UdpPort,
    refresh: Duration,
    last_msg: SamplingMessage,
}

impl PartialEq for SamplingPort {
    fn eq(&self, other: &Self) -> bool {
        self.config.eq(&other.config)
    }
}

impl Deref for SamplingPort {
    type Target = Port;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl SamplingPort {
    pub fn new(
        config: Port,
        direction: PortDirection,
        max_size: usize,
        refresh: Duration,
    ) -> Result<Self> {
        let port = UdpPort::new(&config, direction, max_size)?;
        Ok(Self {
            config,
            port,
            refresh,
            last_msg: SamplingMessage::default(),
        })
    }

    pub fn refresh(&self) -> Duration {
        self.refresh
    }

    pub fn read(&mut self) -> Result<SamplingMessage> {
        let msg = self.port.read();
        match msg {
            Ok(msg) => self.last_msg = msg,
            Err(e) => log::warn!("{e:?}"),
        }
        Ok(self.last_msg.clone())
    }

    pub fn write(&mut self, msg: &[u8]) -> Result<()> {
        if let Err(e) = self.port.write(msg) {
            let name = &self.config.name;
            log::warn!("[SamplingPort(\"{name}\")] {e:?}")
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SamplingMessage {
    bytes: Vec<u8>,
    when: Instant,
}

impl SamplingMessage {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            when: Instant::now(),
        }
    }

    pub fn msg(&self) -> &[u8] {
        &self.bytes
    }

    pub fn validity(&self, refresh: Duration) -> Validity {
        let elapsed = self.when.elapsed();
        if elapsed > refresh {
            return Validity::Invalid;
        }
        Validity::Valid
    }
}

impl Default for SamplingMessage {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            when: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct UdpPort {
    socket: UdpSocket,
    direction: PortDirection,
    buffer: Vec<u8>,
}

impl UdpPort {
    pub fn new(channel: &Port, direction: PortDirection, max_size: usize) -> Result<Self> {
        let socket;
        let local_addr = "127.0.0.1:0";
        match direction {
            PortDirection::Source => {
                socket = UdpSocket::bind(local_addr).unwrap();
                socket.connect(channel).unwrap();
                log::debug!(
                    "Created UDP Port({}) with Peer({})",
                    socket.local_addr()?,
                    socket.peer_addr()?
                );
            }
            PortDirection::Destination => {
                let addrs = channel.to_socket_addrs()?;
                let local_addrs = local_addr.to_socket_addrs()?;
                let all_addrs: Vec<_> = addrs.chain(local_addrs).collect();
                socket = UdpSocket::bind(all_addrs.as_slice())?;
                log::debug!("Created UDP Port({})", socket.local_addr()?);
            }
        };
        socket.set_nonblocking(true)?;
        Ok(UdpPort {
            socket,
            direction,
            buffer: vec![0u8; max_size],
        })
    }

    fn read(&mut self) -> Result<SamplingMessage> {
        _ = ..|| ..;
        if self.direction != PortDirection::Destination {
            bail!("Can not read \"SOURCE\" Port");
        }
        let size = self.socket.recv(&mut self.buffer)?;
        let msg = &self.buffer[..size];
        Ok(SamplingMessage::new(msg))
    }

    fn write(&mut self, msg: &[u8]) -> Result<()> {
        if self.direction != PortDirection::Source {
            bail!("Can not read \"DESTINATION\" Port");
        }
        self.socket.send(msg)?;
        Ok(())
    }
}
