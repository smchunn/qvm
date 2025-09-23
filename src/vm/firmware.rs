//! Firmware detection and management

use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// From qemu-system-* realpath, derive/share and find firmware in Nix paths
pub fn locate_firmware_from_qemu(qemu_bin: &Path, arch: &str) -> Result<(PathBuf, PathBuf)> {
    let bin_real = qemu_bin
        .canonicalize()
        .unwrap_or_else(|_| qemu_bin.to_path_buf());
    let bin_str = bin_real.to_string_lossy();

    // Try to derive .../share/qemu next to .../bin/qemu-system-*
    let derived_share = if bin_str.contains("/bin/qemu-system-") {
        if arch == "aarch64" {
            PathBuf::from(bin_str.replace("/bin/qemu-system-aarch64", "/share/qemu"))
        } else {
            PathBuf::from(bin_str.replace("/bin/qemu-system-x86_64", "/share/qemu"))
        }
    } else {
        qemu_bin
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("share/qemu"))
            .unwrap_or_else(|| PathBuf::from("/run/current-system/sw/share/qemu"))
    };

    let mut dirs = vec![
        derived_share,
        PathBuf::from("/run/current-system/sw/share/qemu"),
        PathBuf::from("/nix/var/nix/profiles/system/sw/share/qemu"),
    ];

    // Also scan /nix/store/*-qemu-*/share/qemu
    if let Ok(iter) = fs::read_dir("/nix/store") {
        for e in iter.flatten() {
            let p = e.path();
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.contains("-qemu-") {
                    let q = p.join("share/qemu");
                    if q.is_dir() {
                        dirs.push(q);
                    }
                }
            }
        }
    }

    let pairs: &[(&str, &str)] = match arch {
        "aarch64" => &[
            ("edk2-aarch64-code.fd", "edk2-arm-vars.fd"),
            ("edk2-aarch64-code.fd", "edk2-aarch64-vars.fd"),
        ],
        "x86_64" => &[
            ("OVMF_CODE.fd", "OVMF_VARS.fd"),
            ("edk2-x86_64-code.fd", "edk2-x86_64-vars.fd"),
            ("edk2-x86_64-code.fd", "edk2-i386-vars.fd"),
        ],
        _ => return Err(anyhow!("Unsupported arch: {}", arch)),
    };

    for d in dirs {
        if !d.is_dir() {
            continue;
        }
        for (code, vars) in pairs {
            let c = d.join(code);
            let v = d.join(vars);
            if c.is_file() && v.is_file() {
                return Ok((c, v));
            }
        }
    }
    Err(anyhow!("UEFI firmware not found in Nix paths for {}", arch))
}

/// Get default firmware paths for architecture
pub fn get_default_firmware_paths(arch: &str) -> (PathBuf, PathBuf) {
    if arch == "aarch64" {
        (
            PathBuf::from("/run/current-system/sw/share/qemu/edk2-aarch64-code.fd"),
            PathBuf::from("/run/current-system/sw/share/qemu/edk2-arm-vars.fd"),
        )
    } else {
        (
            PathBuf::from("/run/current-system/sw/share/qemu/OVMF_CODE.fd"),
            PathBuf::from("/run/current-system/sw/share/qemu/OVMF_VARS.fd"),
        )
    }
}