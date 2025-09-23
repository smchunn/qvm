//! QVM - QEMU VM Manager Library
//!
//! A modern, efficient library for managing QEMU virtual machines.

pub mod cli;
pub mod vm;
pub mod config;
pub mod utils;

// Re-export commonly used types
pub use config::schema::VmConfig;
pub use vm::manager::VmManager;
pub use cli::commands::Cli;

/// Library error type
pub type Result<T> = anyhow::Result<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use utils::system::now_utc;
    use utils::paths::{resolve_under_root, conf_path, qvm_home, find_vm_dir};
    use utils::system::pick_qemu_bin;
    use vm::config::save_conf;
    use std::fs::File;
    use std::path::PathBuf;

    // Helper function that was removed from main
    fn _cpu_total(s: u32, c: u32, t: u32) -> u32 {
        s.saturating_mul(c).saturating_mul(t)
    }

    #[test]
    fn test_now_utc() {
        let timestamp = now_utc();
        println!("Timestamp: {}", timestamp);
        assert!(timestamp.contains("T"));
        // The timestamp should end with Z (UTC) or +00:00
        assert!(timestamp.ends_with("Z") || timestamp.ends_with("+00:00"));
    }

    #[test]
    fn test_resolve_under_root() {
        let root = PathBuf::from("/tmp/test");

        // Test absolute path
        let abs_path = PathBuf::from("/absolute/path");
        assert_eq!(resolve_under_root(&root, &abs_path), abs_path);

        // Test relative path
        let rel_path = PathBuf::from("relative/path");
        assert_eq!(resolve_under_root(&root, &rel_path), root.join("relative/path"));
    }

    #[test]
    fn test_conf_path() {
        let root = PathBuf::from("/tmp/test");
        assert_eq!(conf_path(&root), root.join("vm.json"));
    }

    #[test]
    fn test_cpu_total() {
        assert_eq!(_cpu_total(2, 4, 2), 16);
        assert_eq!(_cpu_total(1, 8, 1), 8);
        assert_eq!(_cpu_total(0, 4, 2), 0);
    }

    #[test]
    fn test_qvm_home() {
        let home = qvm_home().unwrap();
        assert!(home.to_string_lossy().contains("qvm"));
    }

    #[test]
    fn test_find_vm_dir_nonexistent() {
        let result = find_vm_dir("nonexistent-vm-test-12345");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_pick_qemu_bin() {
        // Test supported architectures
        let result_aarch64 = pick_qemu_bin("aarch64");
        let result_x86_64 = pick_qemu_bin("x86_64");

        // At least one should work (depending on system)
        if result_aarch64.is_err() && result_x86_64.is_err() {
            // This is expected on systems without QEMU
            println!("QEMU not found (expected on some test systems)");
        }

        // Test unsupported architecture
        let result = pick_qemu_bin("unsupported");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported arch"));
    }

    #[test]
    fn test_vm_config_serialization() {
        use config::schema::*;

        let config = VmConfig {
            meta: Meta {
                version: 1,
                generated: now_utc(),
                name: "test-vm".to_string(),
                arch: "aarch64".to_string(),
                uuid: "test-uuid".to_string(),
            },
            paths: Paths {
                root: PathBuf::from("/tmp/test"),
                disk: PathBuf::from("disk.qcow2"),
                efi_vars: PathBuf::from("efi_vars.fd"),
            },
            hardware: Hardware {
                cpu_model: "host".to_string(),
                sockets: 1,
                cores: 4,
                threads: 1,
                mem_mb: 4096,
                machine: "virt".to_string(),
                accel: "hvf".to_string(),
                mac: "52:54:00:12:34:56".to_string(),
            },
            firmware: Firmware {
                code: PathBuf::from("/path/to/code.fd"),
                vars_template: PathBuf::from("/path/to/vars.fd"),
            },
            network: Network {
                mode: "vmnet-shared".to_string(),
                bridge_if: "en0".to_string(),
                forwards: Forwards::default(),
            },
            display: Display {
                mode: "cocoa".to_string(),
                vnc: Vnc {
                    use_unix: false,
                    host: "127.0.0.1".to_string(),
                    display: 1,
                    sock: PathBuf::from("vnc.sock"),
                },
                spice: Spice {
                    use_unix: false,
                    addr: "127.0.0.1".to_string(),
                    port: 5930,
                    disable_ticketing: true,
                    sock: PathBuf::from("spice.sock"),
                },
            },
        };

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-vm"));
        assert!(json.contains("aarch64"));

        // Test deserialization
        let parsed: VmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.meta.name, "test-vm");
        assert_eq!(parsed.meta.arch, "aarch64");
    }

    #[test]
    fn test_save_and_load_conf() {
        use config::schema::*;

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        let config = VmConfig {
            meta: Meta {
                version: 1,
                generated: now_utc(),
                name: "test-vm".to_string(),
                arch: "aarch64".to_string(),
                uuid: "test-uuid".to_string(),
            },
            paths: Paths {
                root: root.clone(),
                disk: PathBuf::from("disk.qcow2"),
                efi_vars: PathBuf::from("efi_vars.fd"),
            },
            hardware: Hardware {
                cpu_model: "host".to_string(),
                sockets: 1,
                cores: 4,
                threads: 1,
                mem_mb: 4096,
                machine: "virt".to_string(),
                accel: "hvf".to_string(),
                mac: "52:54:00:12:34:56".to_string(),
            },
            firmware: Firmware {
                code: PathBuf::from("/path/to/code.fd"),
                vars_template: PathBuf::from("/path/to/vars.fd"),
            },
            network: Network {
                mode: "vmnet-shared".to_string(),
                bridge_if: "en0".to_string(),
                forwards: Forwards::default(),
            },
            display: Display {
                mode: "cocoa".to_string(),
                vnc: Vnc {
                    use_unix: false,
                    host: "127.0.0.1".to_string(),
                    display: 1,
                    sock: PathBuf::from("vnc.sock"),
                },
                spice: Spice {
                    use_unix: false,
                    addr: "127.0.0.1".to_string(),
                    port: 5930,
                    disable_ticketing: true,
                    sock: PathBuf::from("spice.sock"),
                },
            },
        };

        // Save config
        save_conf(&config).unwrap();

        // Verify file was created
        let config_path = conf_path(&root);
        assert!(config_path.exists());

        // Load and verify config
        let loaded_config: VmConfig = {
            let file = File::open(&config_path).unwrap();
            serde_json::from_reader(file).unwrap()
        };

        assert_eq!(loaded_config.meta.name, "test-vm");
        assert_eq!(loaded_config.meta.arch, "aarch64");
        assert_eq!(loaded_config.hardware.mem_mb, 4096);
    }
}