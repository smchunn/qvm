//! Path utility functions

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Get the qvm_home directory path
pub fn qvm_home() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("no home directory found"))?
        .join("qvm"))
}

/// Resolve path under root directory
pub fn resolve_under_root(root: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}

/// Get VM configuration file path
pub fn conf_path(root: &Path) -> PathBuf {
    root.join("vm.json")
}

/// Find VM directory by name
pub fn find_vm_dir(name: &str) -> Result<PathBuf> {
    let qvm_home = qvm_home()?;
    let vm_dir = qvm_home.join(format!("{}.qvm", name));

    if !vm_dir.exists() {
        return Err(anyhow!("VM '{}' not found in {}", name, qvm_home.display()));
    }

    Ok(vm_dir)
}