//! VM creation functionality

use crate::config::schema::*;
use crate::utils::paths::{qvm_home, resolve_under_root};
use crate::utils::system::{now_utc, pick_qemu_bin};
use crate::vm::config::save_conf;
use crate::vm::firmware::{locate_firmware_from_qemu, get_default_firmware_paths};
use crate::Result;
use anyhow::anyhow;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// VM Creation parameters
pub struct CreateParams {
    pub name: String,
    pub arch: String,
    pub cpu_model: String,
    pub smp: Option<u32>,
    pub sockets: Option<u32>,
    pub cores: Option<u32>,
    pub threads: Option<u32>,
    pub mem: u32,
    pub net_mode: String,
    pub bridge_if: String,
    pub display_mode: String,
    pub disk: Option<PathBuf>,
    pub disk_size: Option<String>,
    pub vnc_host: String,
    pub vnc_display: u8,
    pub vnc_sock: Option<PathBuf>,
    pub vnc_unix: bool,
    pub spice_addr: String,
    pub spice_port: u16,
    pub spice_sock: Option<PathBuf>,
    pub spice_unix: bool,
    pub spice_disable_ticketing: bool,
}

/// VM Creator
pub struct VmCreator;

impl VmCreator {
    /// Create a new VM with the given parameters
    pub fn create_vm(params: CreateParams) -> Result<()> {
        // VM root
        let root = qvm_home()?.join(format!("{}.qvm", params.name));
        fs::create_dir_all(&root)?;

        // Disk path (keep relative in JSON if user provided relative)
        let disk_rel_or_abs = params.disk.unwrap_or_else(|| PathBuf::from("disk.qcow2"));
        let disk_abs = resolve_under_root(&root, &disk_rel_or_abs);

        // Create disk if size requested and file not present
        if let Some(sz) = &params.disk_size {
            if !disk_abs.exists() {
                let status = Command::new("qemu-img")
                    .args(["create", "-f", "qcow2"])
                    .arg(&disk_abs)
                    .arg(sz)
                    .status()?;
                if !status.success() {
                    return Err(anyhow!("qemu-img failed to create disk (size: {sz})"));
                }
            }
        } else if !disk_abs.exists() {
            eprintln!(
                "Note: no disk at {} (use --disk-size to create one)",
                disk_abs.display()
            );
        }

        // CPU model normalization (like earlier: x86_64 'host' → 'qemu64' for portability)
        let cpu_model_final = if params.arch == "x86_64" && params.cpu_model == "host" {
            "qemu64".to_string()
        } else {
            params.cpu_model.clone()
        };

        // Topology decision
        let (skt, cor, thr) = if params.sockets.is_some() || params.cores.is_some() || params.threads.is_some() {
            (
                params.sockets.unwrap_or(1),
                params.cores.unwrap_or(1),
                params.threads.unwrap_or(1),
            )
        } else if let Some(n) = params.smp {
            (1, n.max(1), 1) // simple: cores=N
        } else {
            (1, 4, 1) // defaults
        };

        // Resolve qemu bin (Nix aware) and firmware from it
        let qemu_bin = pick_qemu_bin(&params.arch)?;
        let (fw_code_path, fw_vars_tpl_path) = locate_firmware_from_qemu(&qemu_bin, &params.arch)
            .unwrap_or_else(|e| {
                eprintln!("Warning: {e}");
                get_default_firmware_paths(&params.arch)
            });

        let cfg = VmConfig {
            meta: Meta {
                version: 1,
                generated: now_utc(),
                name: params.name.clone(),
                arch: params.arch.clone(),
                uuid: uuid::Uuid::new_v4().to_string(),
            },
            paths: Paths {
                root: root.clone(),
                disk: disk_rel_or_abs,
                efi_vars: PathBuf::from("efi_vars.fd"),
            },
            hardware: Hardware {
                cpu_model: cpu_model_final,
                sockets: skt,
                cores: cor,
                threads: thr,
                mem_mb: params.mem,
                machine: if params.arch == "aarch64" {
                    "virt,gic-version=3".into()
                } else {
                    "q35".into()
                },
                accel: if params.arch == "aarch64" {
                    "hvf".into()
                } else {
                    "kvm".into() // Assuming the rest was "kvm" for x86_64
                },
                mac: format!("52:54:00:{:02x}:{:02x}:{:02x}",
                    rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>()),
            },
            firmware: Firmware {
                code: fw_code_path,
                vars_template: fw_vars_tpl_path,
            },
            network: Network {
                mode: params.net_mode,
                bridge_if: params.bridge_if,
                forwards: Forwards::default(),
            },
            display: Display {
                mode: params.display_mode,
                vnc: Vnc {
                    use_unix: params.vnc_unix,
                    host: params.vnc_host,
                    display: params.vnc_display,
                    sock: params.vnc_sock.unwrap_or_else(|| PathBuf::from("vnc.sock")),
                },
                spice: Spice {
                    use_unix: params.spice_unix,
                    addr: params.spice_addr,
                    port: params.spice_port,
                    disable_ticketing: params.spice_disable_ticketing,
                    sock: params.spice_sock.unwrap_or_else(|| PathBuf::from("spice.sock")),
                },
            },
        };

        save_conf(&cfg)?;
        println!("Created VM '{}' at {}", params.name, root.display());
        Ok(())
    }
}