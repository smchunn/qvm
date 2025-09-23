//! VM lifecycle management

use crate::utils::paths::{find_vm_dir, resolve_under_root};
use crate::utils::system::is_vm_running;
use crate::vm::config::load_conf;
use crate::Result;
use anyhow::anyhow;
use std::fs;
use std::io::{self, Write};

/// VM Manager for lifecycle operations
pub struct VmManager;

impl VmManager {
    /// Create a new VM manager instance
    pub fn new() -> Self {
        Self
    }

    /// Delete a VM by name
    pub fn delete_vm(&self, name: &str, force: bool) -> Result<()> {
        // Check if VM exists
        let vm_dir = find_vm_dir(name)?;

        // Check if VM is running
        if is_vm_running(name)? {
            return Err(anyhow!(
                "Cannot delete VM '{}': VM is currently running. Stop it first with 'qvm stop {}'",
                name, name
            ));
        }

        // Load config to show user what will be deleted
        let config = load_conf(name)?;

        if !force {
            println!("About to delete VM '{}':", name);
            println!("  VM Directory: {}", vm_dir.display());
            println!("  Disk: {}", resolve_under_root(&vm_dir, &config.paths.disk).display());
            println!("  EFI Vars: {}", resolve_under_root(&vm_dir, &config.paths.efi_vars).display());
            println!();
            print!("Are you sure you want to delete this VM? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                println!("Deletion cancelled.");
                return Ok(());
            }
        }

        // Remove the entire VM directory
        fs::remove_dir_all(&vm_dir)?;

        println!("Successfully deleted VM '{}'", name);
        Ok(())
    }

    /// Start a VM (placeholder for future implementation)
    pub fn start_vm(&self, name: &str) -> Result<()> {
        println!("Starting VM '{}' (not implemented)", name);
        Ok(())
    }

    /// Stop a VM (placeholder for future implementation)
    pub fn stop_vm(&self, name: &str) -> Result<()> {
        println!("Stopping VM '{}' (not implemented)", name);
        Ok(())
    }
}

impl Default for VmManager {
    fn default() -> Self {
        Self::new()
    }
}