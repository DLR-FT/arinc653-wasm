use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser};
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};
use url::Url;

#[derive(Parser, Debug)]
pub struct Cli {
    // #[arg(short, env)]
    // pub config: Option<PathBuf>,
    #[command(flatten)]
    pub config_delegate: Config,
}

#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[command(flatten)]
    pub wasm_config: WasmConfig,
    #[command(flatten)]
    pub arinc_config: ArincConfig,
}

impl Config {
    pub fn load_from_file<T: AsRef<Path>>(file: T) -> Result<Config> {
        let content = std::fs::read_to_string(file)?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
pub struct WasmConfig {
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
    pub wasm_module_path: String,
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
pub struct ArincConfig {
    /// <"udp"/"tcp">://<name>@<addr>:<port>
    #[arg(short, long = "sampling-port", id = "SAMPLING_PORT")]
    pub sampling_ports: Vec<Port>,
    // /// <"udp"/"tcp">://<name>@<addr>:<port>
    // #[arg(short, long = "queuing-port", id = "QUEUING_PORT")]
    // pub queuing_ports: Vec<Channel>,
}

#[derive(Debug, Clone, Copy, EnumString, Deserialize, Serialize, VariantNames)]
#[strum(ascii_case_insensitive)]
pub enum Protocol {
    UDP,
    TCP,
}

#[derive(Args, Debug, Deserialize, Serialize, Clone)]
pub struct Port {
    pub name: String,
    pub addrs: Vec<SocketAddr>,
    pub protocol: Protocol,
}

impl PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}

impl ToSocketAddrs for Port {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(self.addrs.clone().into_iter())
    }
}

impl FromStr for Port {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        let scheme = url.scheme();
        let name = url.username().to_string();
        if name.is_empty() {
            bail!("name of the channel needs to be defined as \"<name>@<addr>\" in the uri");
        }
        let variants = Protocol::VARIANTS;
        let error = format!("Protocol \"{scheme}\" not found. Possible {variants:?}");
        let protocol = Protocol::from_str(scheme).context(error)?;
        let addrs = url.socket_addrs(|| None)?;
        Ok(Port {
            name,
            addrs,
            protocol,
        })
    }
}
