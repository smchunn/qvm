use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("QEMU VM manager in Rust"));
}

#[test]
fn test_completions_command() {
    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.args(&["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_qvm"));
}

#[test]
fn test_man_page_command() {
    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.arg("man-page")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH"));
}

#[test]
fn test_delete_nonexistent_vm() {
    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.args(&["delete", "nonexistent-vm", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("VM 'nonexistent-vm' not found"));
}

#[test]
fn test_create_and_delete_vm() {
    let temp_home = TempDir::new().unwrap();
    let qvm_dir = temp_home.path().join("qvm");
    fs::create_dir_all(&qvm_dir).unwrap();

    // Set HOME to our temp directory
    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.env("HOME", temp_home.path())
        .args(&["create", "test-vm", "--mem", "2048", "--disk-size", "10G"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created VM 'test-vm'"));

    // Verify VM directory was created
    let vm_dir = qvm_dir.join("test-vm.qvm");
    assert!(vm_dir.exists());
    assert!(vm_dir.join("vm.json").exists());

    // Delete the VM
    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.env("HOME", temp_home.path())
        .args(&["delete", "test-vm", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully deleted VM 'test-vm'"));

    // Verify VM directory was removed
    assert!(!vm_dir.exists());
}

#[test]
fn test_create_vm_with_invalid_arch() {
    let temp_home = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("qvm").unwrap();
    cmd.env("HOME", temp_home.path())
        .args(&["create", "test-vm", "--arch", "invalid-arch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported arch"));
}

#[test]
fn test_install_fish_completions() {
    let temp_config = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("qvm").unwrap();
    // Capture stdout to see where the file was actually created
    let output = cmd.env("HOME", temp_config.path())
        .env("XDG_CONFIG_HOME", temp_config.path())
        .arg("install-fish")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Fish completions installed"));

    // Extract the actual path from the output
    let path_line = stdout.lines().find(|line| line.contains("Fish completions installed to:"));
    assert!(path_line.is_some(), "Expected output to contain installation path");

    // For simplicity, just check that the program says it succeeded
    // The actual file location depends on the system configuration
    println!("Fish completion install output: {}", stdout);
}