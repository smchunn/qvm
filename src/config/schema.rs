//! VM configuration schema definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// VM configuration schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmConfig {
    pub meta: Meta,
    pub paths: Paths,
    pub hardware: Hardware,
    pub firmware: Firmware,
    pub network: Network,
    pub display: Display,
}

/// VM metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    pub version: u32,
    pub generated: String,
    pub name: String,
    pub arch: String,
    pub uuid: String,
}

/// VM file paths
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub disk: PathBuf,     // may be relative to root
    pub efi_vars: PathBuf, // may be relative to root
}

/// VM hardware configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hardware {
    pub cpu_model: String,
    pub sockets: u32,
    pub cores: u32,
    pub threads: u32,
    pub mem_mb: u32,
    pub machine: String,
    pub accel: String,
    pub mac: String,
}

/// VM firmware configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Firmware {
    pub code: PathBuf,          // absolute path to firmware code
    pub vars_template: PathBuf, // absolute path to firmware vars template
}

/// VM network configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
    pub mode: String,      // vmnet-shared | vmnet-bridged | user
    pub bridge_if: String, // for vmnet-bridged
    pub forwards: Forwards,
}

/// Port forwarding configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Forwards {
    pub ssh: u16,
    pub meye: u16,
}

/// VM display configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Display {
    pub mode: String, // cocoa | vnc | spice | headless
    pub vnc: Vnc,
    pub spice: Spice,
}

/// VNC configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vnc {
    pub use_unix: bool,
    pub host: String,
    pub display: u8,
    pub sock: PathBuf, // may be relative to root
}

/// SPICE configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spice {
    pub use_unix: bool,
    pub addr: String,
    pub port: u16,
    pub disable_ticketing: bool,
    pub sock: PathBuf, // may be relative to root
}