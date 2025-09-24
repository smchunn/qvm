# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

QVM is a QEMU VM Manager written in Rust that provides a modern CLI interface for managing QEMU virtual machines. The tool handles VM lifecycle management (create, start, stop, delete), supports multiple architectures (ARM64/x86_64), and provides flexible display options (Cocoa, VNC, SPICE, headless).

## Core Architecture

### Module Structure

- **`src/cli/`**: Command-line interface using Clap
  - `commands.rs`: CLI command definitions and argument parsing
  - `completions.rs`: Shell completion generation (Fish, Bash, Zsh, PowerShell)
- **`src/vm/`**: Virtual machine management core
  - `config.rs`: VM configuration loading/saving to JSON
  - `creator.rs`: VM creation logic and disk provisioning
  - `manager.rs`: VM lifecycle operations (start/stop/delete)
  - `firmware.rs`: UEFI firmware detection and setup
- **`src/config/`**: Configuration schema and validation
  - `schema.rs`: Serde-based VM configuration data structures
- **`src/utils/`**: Shared utilities
  - `paths.rs`: Path resolution and VM directory management
  - `system.rs`: System utilities (QEMU detection, process management)

### Key Data Structures

**VmConfig** (`config/schema.rs`): The central configuration structure serialized to `vm.json`:
- `Meta`: VM metadata (name, arch, UUID, version)
- `Paths`: File paths (root, disk, EFI vars) - may be relative to VM root
- `Hardware`: CPU, memory, machine type, acceleration settings
- `Firmware`: UEFI firmware code and vars template paths
- `Network`: Networking mode (vmnet-shared/bridged/user) and port forwarding
- `Display`: Display configuration for Cocoa/VNC/SPICE/headless modes

### VM Storage Structure

VMs are stored in `~/qvm/` with each VM in its own directory:
```
~/qvm/
├── my-vm.qvm/
│   ├── vm.json          # VM configuration
│   ├── disk.qcow2       # Virtual disk
│   ├── efi_vars.fd      # EFI variables
│   ├── vm.pid           # Process ID (when running)
│   └── *.sock           # VNC/SPICE sockets (if using UNIX sockets)
```

## Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run integration tests specifically
cargo test --test integration_tests

# Run tests with output visible
cargo test -- --nocapture

# Build release version
cargo build --release
```

### Installation and Shell Completions
```bash
# Install Fish completions to ~/.config/fish/completions/
cargo run --bin qvm install-fish

# Generate completions for other shells
cargo run --bin qvm completions bash
cargo run --bin qvm completions zsh
cargo run --bin qvm completions fish

# Generate man page
cargo run --bin qvm man-page
```

### Architecture-Specific Details

**ARM64 (aarch64)**:
- Machine type: `virt,gic-version=3`
- Acceleration: `hvf` (macOS Hypervisor.framework)
- Firmware: EDK2 AARCH64 UEFI
- Default CPU: `host`

**x86_64**:
- Machine type: `q35`
- Acceleration: `kvm`
- Firmware: OVMF UEFI
- Default CPU: `qemu64`

### Network Modes
- **vmnet-shared**: Default, provides NAT with internet access
- **vmnet-bridged**: Bridges to host interface (requires `--bridge-if`)
- **user**: SLIRP user-mode networking (most compatible)

### Important Implementation Notes

1. **Path Resolution**: The `resolve_under_root()` function handles both absolute and relative paths in VM configurations. Relative paths in `vm.json` are resolved relative to the VM's root directory.

2. **VM State Management**: The `is_vm_running()` function checks for `vm.pid` files to determine if a VM is running before allowing operations like deletion.

3. **QEMU Binary Detection**: The `pick_qemu_bin()` function automatically detects the appropriate QEMU binary (`qemu-system-aarch64` or `qemu-system-x86_64`) based on architecture.

4. **Fish Completions Path**: The `install_fish_completions()` function explicitly uses `~/.config/fish/completions/` instead of the system default to follow standard conventions.

5. **Configuration Versioning**: VM configurations include a `version` field in metadata for future schema evolution.

6. **Error Handling**: The project uses `anyhow::Result` throughout for consistent error handling.

### Testing Strategy

The project includes comprehensive unit tests in `src/lib.rs` covering:
- Path resolution utilities
- VM configuration serialization/deserialization
- QEMU binary detection
- Core utility functions

Integration tests are located in `tests/integration_tests.rs` and test the full CLI interface using `assert_cmd`.

## Commit Message Convention

This project follows the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification:

### Format
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types
- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files

### Scopes (optional)
- **cli**: Command-line interface changes
- **vm**: Virtual machine management
- **config**: Configuration handling
- **utils**: Utility functions
- **docs**: Documentation

### Examples
```
feat(cli): add support for custom CPU topology options
fix(vm): resolve path resolution issue for relative disk paths
docs: update README with new network mode examples
refactor(config): simplify VM configuration schema validation
test(utils): add comprehensive path resolution tests
```

### Autocommit Indicator

When committing automatically (e.g., via tooling or CI), prefix the commit message with:
```
[AUTOCOMMIT] <type>[optional scope]: <description>
```

This clearly identifies automated commits and distinguishes them from manual commits.

### Guidelines

- Use present tense ("add feature" not "added feature")
- Use imperative mood ("move cursor to..." not "moves cursor to...")
- Keep the first line under 72 characters
- Reference issues and pull requests when applicable
- Do not use emojis in commit messages or documentation