use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// QVM CLI (Rust)
#[derive(Parser, Debug)]
#[command(name = "qvm", about = "QEMU VM manager in Rust")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Create a new VM (writes vm.json; can create qcow2 disk)
    Create {
        name: String,

        /// Guest architecture (aarch64|x86_64)
        #[arg(long, default_value = "aarch64")]
        arch: String,

        /// CPU model (e.g., host, qemu64, max, …)
        #[arg(long, default_value = "host")]
        cpu_model: String,

        /// Simple vCPU count (ignored if sockets/cores/threads provided)
        #[arg(long)]
        smp: Option<u32>,

        /// CPU topology: sockets (wins over --smp if any topology flag set)
        #[arg(long)]
        sockets: Option<u32>,

        /// CPU topology: cores per socket
        #[arg(long)]
        cores: Option<u32>,

        /// CPU topology: threads per core
        #[arg(long)]
        threads: Option<u32>,

        /// Memory (MB)
        #[arg(long, default_value_t = 4096)]
        mem: u32,

        /// Network mode (vmnet-shared|vmnet-bridged|user)
        #[arg(long, default_value = "vmnet-shared")]
        net_mode: String,

        /// Bridge interface (when vmnet-bridged)
        #[arg(long, default_value = "en0")]
        bridge_if: String,

        /// Display mode (cocoa|vnc|spice|headless)
        #[arg(long, default_value = "cocoa")]
        display_mode: String,

        // Disk options
        /// Disk path (qcow2). If relative, it's under the VM root.
        #[arg(long)]
        disk: Option<PathBuf>,

        /// Create qcow2 disk if absent (e.g., 64G, 100G)
        #[arg(long)]
        disk_size: Option<String>,

        // VNC
        #[arg(long, default_value = "127.0.0.1")]
        vnc_host: String,
        #[arg(long, default_value_t = 1)]
        vnc_display: u8,
        #[arg(long)]
        vnc_sock: Option<PathBuf>,
        /// Use VNC UNIX socket (boolean flag; default false)
        #[arg(long)]
        vnc_unix: bool,

        // SPICE
        #[arg(long, default_value = "127.0.0.1")]
        spice_addr: String,
        #[arg(long, default_value_t = 5930)]
        spice_port: u16,
        #[arg(long)]
        spice_sock: Option<PathBuf>,
        /// Use SPICE UNIX socket (boolean flag; default false)
        #[arg(long)]
        spice_unix: bool,
        #[arg(long, default_value_t = true)]
        spice_disable_ticketing: bool,
    },

    /// Start a VM (optionally override display, attach ISO, pick console, daemonize)
    Start {
        name: String,
        #[arg(long)]
        iso: Option<PathBuf>,
        #[arg(long, value_parser = ["cocoa","vnc","spice","headless"])]
        display: Option<String>,
        #[arg(long, value_parser = ["gui","serial"], default_value = "gui")]
        console: String,
        #[arg(long)]
        daemon: bool,
    },

    /// Stop a VM (reads vm.pid and kills)
    Stop { name: String },

    /// Delete a VM and its associated files
    Delete {
        name: String,
        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },

    /// Persist display settings in vm.json
    SetDisplay {
        name: String,
        #[arg(value_parser = ["cocoa","vnc","spice","headless"])]
        mode: String,

        // VNC
        #[arg(long)]
        vnc_unix: bool,
        #[arg(long)]
        vnc_host: Option<String>,
        #[arg(long)]
        vnc_display: Option<u8>,
        #[arg(long)]
        vnc_sock: Option<PathBuf>,

        // SPICE
        #[arg(long)]
        spice_unix: bool,
        #[arg(long)]
        spice_addr: Option<String>,
        #[arg(long)]
        spice_port: Option<u16>,
        #[arg(long)]
        spice_sock: Option<PathBuf>,
        #[arg(long)]
        spice_disable_ticketing: Option<bool>,
    },
}

/// JSON schema
#[derive(Serialize, Deserialize, Debug, Clone)]
struct VmConfig {
    meta: Meta,
    paths: Paths,
    hardware: Hardware,
    firmware: Firmware,
    network: Network,
    display: Display,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Meta {
    version: u32,
    generated: String,
    name: String,
    arch: String,
    uuid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Paths {
    root: PathBuf,
    disk: PathBuf,     // may be relative to root
    efi_vars: PathBuf, // may be relative to root
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Hardware {
    cpu_model: String,
    sockets: u32,
    cores: u32,
    threads: u32,
    mem_mb: u32,
    machine: String,
    accel: String,
    mac: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Firmware {
    code: PathBuf,          // absolute path to firmware code
    vars_template: PathBuf, // absolute path to firmware vars template
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Network {
    mode: String,      // vmnet-shared | vmnet-bridged | user
    bridge_if: String, // for vmnet-bridged
    forwards: Forwards,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Forwards {
    ssh: u16,
    meye: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Display {
    mode: String, // cocoa | vnc | spice | headless
    vnc: Vnc,
    spice: Spice,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Vnc {
    use_unix: bool,
    host: String,
    display: u8,
    sock: PathBuf, // may be relative to root
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Spice {
    use_unix: bool,
    addr: String,
    port: u16,
    disable_ticketing: bool,
    sock: PathBuf, // may be relative to root
}

/// ---- helpers ----
fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn rm_if_exists(p: &Path) {
    let _ = fs::remove_file(p);
}

fn resolve_under_root(root: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}

fn conf_path(root: &Path) -> PathBuf {
    root.join("vm.json")
}

fn save_conf(cfg: &VmConfig) -> Result<()> {
    let f = File::create(conf_path(&cfg.paths.root))?;
    serde_json::to_writer_pretty(f, cfg)?;
    Ok(())
}

fn load_conf(name: &str) -> Result<VmConfig> {
    let root = qvm_home()?.join(format!("{name}.qvm"));
    let f = File::open(conf_path(&root))?;
    let cfg: VmConfig = serde_json::from_reader(f)?;
    Ok(cfg)
}

/// Get the qvm_home directory path
fn qvm_home() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("no home directory found"))?
        .join("qvm"))
}

/// Find VM directory by name
fn find_vm_dir(name: &str) -> Result<PathBuf> {
    let qvm_home = qvm_home()?;
    let vm_dir = qvm_home.join(format!("{}.qvm", name));

    if !vm_dir.exists() {
        return Err(anyhow!("VM '{}' not found in {}", name, qvm_home.display()));
    }

    Ok(vm_dir)
}

/// Check if VM is currently running
fn is_vm_running(name: &str) -> Result<bool> {
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

/// Delete VM implementation
fn delete_vm(name: &str, force: bool) -> Result<()> {
    // Check if VM exists
    let vm_dir = find_vm_dir(name)?;

    // Check if VM is running
    if is_vm_running(name)? {
        return Err(anyhow!("Cannot delete VM '{}': VM is currently running. Stop it first with 'qvm stop {}'", name, name));
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

fn cpu_total(s: u32, c: u32, t: u32) -> u32 {
    s.saturating_mul(c).saturating_mul(t)
}

/// Pick qemu-system-* path, Nix-aware
fn pick_qemu_bin(arch: &str) -> Result<PathBuf> {
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

/// From qemu-system-* realpath, derive/share and find firmware in Nix paths
fn locate_firmware_from_qemu(qemu_bin: &Path, arch: &str) -> Result<(PathBuf, PathBuf)> {
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

fn display_args(cfg: &VmConfig) -> Result<Vec<String>> {
    let mut base = vec![
        "-vga".into(),
        "none".into(),
        "-device".into(),
        "virtio-gpu-pci".into(),
    ];
    let root = &cfg.paths.root;

    let mut v = match cfg.display.mode.as_str() {
        "cocoa" => vec!["-display".into(), "cocoa".into()],
        "headless" => vec!["-display".into(), "none".into()],
        "vnc" => {
            if cfg.display.vnc.use_unix {
                let sock = resolve_under_root(root, &cfg.display.vnc.sock);
                vec!["-vnc".into(), format!("unix:{}", sock.display())]
            } else {
                vec![
                    "-vnc".into(),
                    format!("{}:{}", cfg.display.vnc.host, cfg.display.vnc.display),
                ]
            }
        }
        "spice" => {
            if cfg.display.spice.use_unix {
                let sock = resolve_under_root(root, &cfg.display.spice.sock);
                vec![
                    "-spice".into(),
                    format!(
                        "unix=on,addr={},disable-ticketing={},image-compression=off,playback-compression=off,streaming-video=off",
                        sock.display(),
                        if cfg.display.spice.disable_ticketing { "on" } else { "off" }
                    ),
                ]
            } else {
                vec![
                    "-spice".into(),
                    format!(
                        "addr={},port={},disable-ticketing={},image-compression=off,playback-compression=off,streaming-video=off",
                        cfg.display.spice.addr,
                        cfg.display.spice.port,
                        if cfg.display.spice.disable_ticketing { "on" } else { "off" }
                    ),
                ]
            }
        }
        other => return Err(anyhow!("Unsupported display mode {other}")),
    };

    base.append(&mut v);
    Ok(base)
}

/// ---- main ----
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Create {
            name,
            arch,
            cpu_model,
            smp,
            sockets,
            cores,
            threads,
            mem,
            net_mode,
            bridge_if,
            display_mode,
            // disk
            disk,
            disk_size,
            // vnc
            vnc_host,
            vnc_display,
            vnc_sock,
            vnc_unix,
            // spice
            spice_addr,
            spice_port,
            spice_sock,
            spice_unix,
            spice_disable_ticketing,
        } => {
            // VM root
            let root = qvm_home()?.join(format!("{name}.qvm"));
            fs::create_dir_all(&root)?;

            // Disk path (keep relative in JSON if user provided relative)
            let disk_rel_or_abs = disk.unwrap_or_else(|| PathBuf::from("disk.qcow2"));
            let disk_abs = resolve_under_root(&root, &disk_rel_or_abs);

            // Create disk if size requested and file not present
            if let Some(sz) = &disk_size {
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
            let cpu_model_final = if arch == "x86_64" && cpu_model == "host" {
                "qemu64".to_string()
            } else {
                cpu_model.clone()
            };

            // Topology decision
            let (skt, cor, thr) = if sockets.is_some() || cores.is_some() || threads.is_some() {
                (
                    sockets.unwrap_or(1),
                    cores.unwrap_or(1),
                    threads.unwrap_or(1),
                )
            } else if let Some(n) = smp {
                (1, n.max(1), 1) // simple: cores=N
            } else {
                (1, 4, 1) // defaults
            };

            // Resolve qemu bin (Nix aware) and firmware from it
            let qemu_bin = pick_qemu_bin(&arch)?;
            let (fw_code_path, fw_vars_tpl_path) = locate_firmware_from_qemu(&qemu_bin, &arch)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: {e}");
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
                });

            let cfg = VmConfig {
                meta: Meta {
                    version: 1,
                    generated: now_utc(),
                    name: name.clone(),
                    arch: arch.clone(),
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
                    mem_mb: mem,
                    machine: if arch == "aarch64" {
                        "virt,gic-version=3".into()
                    } else {
                        "q35".into()
                    },
                    accel: if arch == "aarch64" {
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
                    mode: net_mode,
                    bridge_if,
                    forwards: Forwards::default(),
                },
                display: Display {
                    mode: display_mode,
                    vnc: Vnc {
                        use_unix: vnc_unix,
                        host: vnc_host,
                        display: vnc_display,
                        sock: vnc_sock.unwrap_or_else(|| PathBuf::from("vnc.sock")),
                    },
                    spice: Spice {
                        use_unix: spice_unix,
                        addr: spice_addr,
                        port: spice_port,
                        disable_ticketing: spice_disable_ticketing,
                        sock: spice_sock.unwrap_or_else(|| PathBuf::from("spice.sock")),
                    },
                },
            };

            save_conf(&cfg)?;
            println!("Created VM '{}' at {}", name, root.display());
        }

        Cmd::Start {
            name,
            iso,
            display,
            console,
            daemon
        } => {
            // Implementation would go here - this is just a placeholder
            println!("Starting VM '{}' (not implemented in this example)", name);
        }

        Cmd::Stop { name } => {
            // Implementation would go here - this is just a placeholder
            println!("Stopping VM '{}' (not implemented in this example)", name);
        }

        Cmd::Delete { name, force } => {
            delete_vm(&name, force)?;
        }

        Cmd::SetDisplay {
            name,
            mode,
            vnc_unix,
            vnc_host,
            vnc_display,
            vnc_sock,
            spice_unix,
            spice_addr,
            spice_port,
            spice_sock,
            spice_disable_ticketing
        } => {
            // Implementation would go here - this is just a placeholder
            println!("Setting display for VM '{}' to '{}' (not implemented in this example)", name, mode);
        }
    }

    Ok(())
}
