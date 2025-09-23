//! System utility functions

use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get current UTC timestamp in RFC3339 format
pub fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Check if VM is currently running
pub fn is_vm_running(name: &str) -> Result<bool> {
    use crate::utils::paths::find_vm_dir;

    let vm_dir = find_vm_dir(name)?;
    let pid_file = vm_dir.join("vm.pid");

    if !pid_file.exists() {
        return Ok(false);
    }

    // Read PID and check if process exists
    match fs::read_to_string(&pid_file) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process exists (cross-platform way)
                #[cfg(unix)]
                {
                    let output = Command::new("ps")
                        .args(["-p", &pid.to_string()])
                        .output()?;
                    return Ok(output.status.success());
                }

                #[cfg(windows)]
                {
                    let output = Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", pid)])
                        .output()?;
                    return Ok(output.status.success() &&
                             String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()));
                }
            }
        }
        Err(_) => {
            // If we can't read the PID file, assume it's stale
            let _ = fs::remove_file(&pid_file);
        }
    }

    Ok(false)
}

/// Pick qemu-system-* path, Nix-aware
pub fn pick_qemu_bin(arch: &str) -> Result<PathBuf> {
    let candidates: &[&str] = match arch {
        "aarch64" => &[
            "/run/current-system/sw/bin/qemu-system-aarch64",
            "qemu-system-aarch64",
        ],
        "x86_64" => &[
            "/run/current-system/sw/bin/qemu-system-x86_64",
            "qemu-system-x86_64",
        ],
        other => return Err(anyhow!("Unsupported arch '{}'", other)),
    };

    for c in candidates {
        let p = if c.starts_with('/') {
            PathBuf::from(c)
        } else {
            which::which(c).unwrap_or_else(|_| PathBuf::from(c))
        };
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(anyhow!("qemu-system-{} not found (Nix)", arch))
}