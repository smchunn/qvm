//! VM configuration management

use crate::config::schema::VmConfig;
use crate::utils::paths::{conf_path, qvm_home};
use crate::Result;
use std::fs::File;

/// Save VM configuration to file
pub fn save_conf(cfg: &VmConfig) -> Result<()> {
    let f = File::create(conf_path(&cfg.paths.root))?;
    serde_json::to_writer_pretty(f, cfg)?;
    Ok(())
}

/// Load VM configuration from file
pub fn load_conf(name: &str) -> Result<VmConfig> {
    let root = qvm_home()?.join(format!("{name}.qvm"));
    let f = File::open(conf_path(&root))?;
    let cfg: VmConfig = serde_json::from_reader(f)?;
    Ok(cfg)
}

/// Load VM configuration from directory
pub fn load_conf_from_dir(vm_dir: &std::path::Path) -> Result<VmConfig> {
    let f = File::open(conf_path(vm_dir))?;
    let cfg: VmConfig = serde_json::from_reader(f)?;
    Ok(cfg)
}