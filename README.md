# QVM - QEMU VM Manager

A modern, efficient command-line tool for managing QEMU virtual machines, written in Rust.

## Features

- **VM Lifecycle Management**: Create, start, stop, and delete VMs with ease
- **Multiple Architectures**: Support for ARM64 (aarch64) and x86_64 architectures
- **Flexible Display Options**: Cocoa, VNC, SPICE, and headless modes
- **Network Configurations**: VMnet-shared, VMnet-bridged, and user networking
- **Shell Completions**: Auto-generate and install completions for various shells
- **Comprehensive Testing**: Full unit and integration test coverage
- **Safety Features**: Running VM protection, confirmation prompts

## Installation

### From Source

```bash
git clone <repository-url>
cd qvm
cargo build --release
sudo cp target/release/qvm /usr/local/bin/
```

### Shell Completions

#### Automatic Fish Installation
```bash
qvm install-fish
```

#### Manual Installation for Other Shells
```bash
# Bash
qvm completions bash > /etc/bash_completion.d/qvm
# or
qvm completions bash > ~/.local/share/bash-completion/completions/qvm

# Zsh
qvm completions zsh > ~/.local/share/zsh/site-functions/_qvm

# Fish
qvm completions fish > ~/.config/fish/completions/qvm.fish

# PowerShell
qvm completions powershell > ~/.config/powershell/completions/qvm.ps1
```

## Usage

### Creating a VM

```bash
# Basic VM creation
qvm create my-vm

# Customized VM with specific options
qvm create my-vm \\
  --arch aarch64 \\
  --cpu-model host \\
  --mem 8192 \\
  --disk-size 64G \\
  --display-mode vnc \\
  --net-mode vmnet-shared
```

#### Create Command Options

- `--arch`: Guest architecture (aarch64|x86_64, default: aarch64)
- `--cpu-model`: CPU model (host, qemu64, max, etc., default: host)
- `--smp`: Simple vCPU count
- `--sockets`, `--cores`, `--threads`: CPU topology (overrides --smp)
- `--mem`: Memory in MB (default: 4096)
- `--net-mode`: Network mode (vmnet-shared|vmnet-bridged|user, default: vmnet-shared)
- `--bridge-if`: Bridge interface for vmnet-bridged (default: en0)
- `--display-mode`: Display mode (cocoa|vnc|spice|headless, default: cocoa)
- `--disk`: Disk path (default: disk.qcow2)
- `--disk-size`: Create qcow2 disk if absent (e.g., 64G, 100G)

#### VNC Options
- `--vnc-host`: VNC host (default: 127.0.0.1)
- `--vnc-display`: VNC display number (default: 1)
- `--vnc-sock`: VNC socket path
- `--vnc-unix`: Use VNC UNIX socket

#### SPICE Options
- `--spice-addr`: SPICE address (default: 127.0.0.1)
- `--spice-port`: SPICE port (default: 5930)
- `--spice-sock`: SPICE socket path
- `--spice-unix`: Use SPICE UNIX socket
- `--spice-disable-ticketing`: Disable SPICE authentication (default: true)

### Managing VMs

```bash
# Start a VM
qvm start my-vm

# Start with ISO attached
qvm start my-vm --iso /path/to/installer.iso

# Start with different display mode
qvm start my-vm --display vnc

# Start in daemon mode
qvm start my-vm --daemon

# Stop a VM
qvm stop my-vm

# Delete a VM (with confirmation)
qvm delete my-vm

# Force delete without confirmation
qvm delete my-vm --force
```

### Display Configuration

```bash
# Change display mode permanently
qvm set-display my-vm cocoa

# Configure VNC
qvm set-display my-vm vnc --vnc-host 0.0.0.0 --vnc-display 2

# Configure SPICE with UNIX socket
qvm set-display my-vm spice --spice-unix --spice-sock /tmp/spice.sock
```

### Documentation and Help

```bash
# Generate man page
qvm man-page > /usr/local/share/man/man1/qvm.1

# Command help
qvm --help
qvm create --help
qvm start --help
```

## VM Storage Structure

VMs are stored in `~/qvm/` with the following structure:

```
~/qvm/
├── my-vm.qvm/
│   ├── vm.json          # VM configuration
│   ├── disk.qcow2       # Virtual disk
│   ├── efi_vars.fd      # EFI variables
│   ├── vm.pid           # Process ID (when running)
│   ├── vnc.sock         # VNC socket (if using UNIX sockets)
│   └── spice.sock       # SPICE socket (if using UNIX sockets)
```

## Configuration Format

The `vm.json` file contains the complete VM configuration:

```json
{
  "meta": {
    "version": 1,
    "generated": "2024-01-01T00:00:00Z",
    "name": "my-vm",
    "arch": "aarch64",
    "uuid": "550e8400-e29b-41d4-a716-446655440000"
  },
  "paths": {
    "root": "/Users/username/qvm/my-vm.qvm",
    "disk": "disk.qcow2",
    "efi_vars": "efi_vars.fd"
  },
  "hardware": {
    "cpu_model": "host",
    "sockets": 1,
    "cores": 4,
    "threads": 1,
    "mem_mb": 4096,
    "machine": "virt,gic-version=3",
    "accel": "hvf",
    "mac": "52:54:00:12:34:56"
  },
  "firmware": {
    "code": "/path/to/edk2-aarch64-code.fd",
    "vars_template": "/path/to/edk2-arm-vars.fd"
  },
  "network": {
    "mode": "vmnet-shared",
    "bridge_if": "en0",
    "forwards": {
      "ssh": 0,
      "meye": 0
    }
  },
  "display": {
    "mode": "cocoa",
    "vnc": {
      "use_unix": false,
      "host": "127.0.0.1",
      "display": 1,
      "sock": "vnc.sock"
    },
    "spice": {
      "use_unix": false,
      "addr": "127.0.0.1",
      "port": 5930,
      "disable_ticketing": true,
      "sock": "spice.sock"
    }
  }
}
```

## Architecture Support

### ARM64 (aarch64)
- **Machine Type**: `virt,gic-version=3`
- **Acceleration**: `hvf` (Hypervisor.framework on macOS)
- **Firmware**: EDK2 AARCH64 UEFI
- **Default CPU**: `host`

### x86_64
- **Machine Type**: `q35`
- **Acceleration**: `kvm`
- **Firmware**: OVMF UEFI
- **Default CPU**: `qemu64` (for portability)

## Network Modes

### vmnet-shared (Default)
- Provides internet access via host NAT
- VMs get IP addresses in a private subnet
- No external network access to VMs

### vmnet-bridged
- Bridges VM network to host interface
- VMs appear as separate devices on the network
- Requires `--bridge-if` specification

### user
- User-mode networking (SLIRP)
- Most compatible but with limitations
- Port forwarding required for external access

## Display Modes

### Cocoa (Default on macOS)
- Native macOS windowing
- Best performance and integration
- Hardware acceleration support

### VNC
- Remote access via VNC protocol
- TCP or UNIX socket support
- Cross-platform clients available

### SPICE
- Advanced remote desktop protocol
- Better performance than VNC
- Supports advanced features like USB redirection

### Headless
- No display output
- Useful for servers or automated setups
- Serial console access recommended

## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run all tests with output
cargo test -- --nocapture
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass
5. Submit a pull request

## Troubleshooting

### Common Issues

#### QEMU Not Found
```
Error: qemu-system-aarch64 not found (Nix)
```
**Solution**: Install QEMU using your package manager or ensure it's in PATH.

#### Firmware Not Found
```
Error: UEFI firmware not found in Nix paths for aarch64
```
**Solution**: Install EDK2 or OVMF firmware packages.

#### Permission Denied (vmnet)
```
Error: Could not configure vmnet
```
**Solution**: Run with appropriate permissions or use user networking mode.

#### VM Won't Delete
```
Error: Cannot delete VM 'my-vm': VM is currently running
```
**Solution**: Stop the VM first with `qvm stop my-vm`.

### Debug Mode

For verbose output and debugging:

```bash
RUST_LOG=debug qvm create my-vm
```

## License

[License information]

## Support

For issues and feature requests, please use the GitHub issue tracker.