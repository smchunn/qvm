//! CLI command definitions

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

/// QVM CLI (Rust)
#[derive(Parser, Debug)]
#[command(name = "qvm", about = "QEMU VM manager in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
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

    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Install Fish shell completions automatically
    InstallFish,

    /// Generate man page
    ManPage,
}