use anyhow::Result;
use clap::{Parser, CommandFactory};
use qvm::cli::commands::{Cli, Cmd};
use qvm::cli::completions::{print_completions, install_fish_completions, generate_man_page};
use qvm::vm::creator::{VmCreator, CreateParams};
use qvm::vm::manager::VmManager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let vm_manager = VmManager::new();

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
            disk,
            disk_size,
            vnc_host,
            vnc_display,
            vnc_sock,
            vnc_unix,
            spice_addr,
            spice_port,
            spice_sock,
            spice_unix,
            spice_disable_ticketing,
        } => {
            let params = CreateParams {
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
                disk,
                disk_size,
                vnc_host,
                vnc_display,
                vnc_sock,
                vnc_unix,
                spice_addr,
                spice_port,
                spice_sock,
                spice_unix,
                spice_disable_ticketing,
            };
            VmCreator::create_vm(params)?;
        }

        Cmd::Start { name, .. } => {
            vm_manager.start_vm(&name)?;
        }

        Cmd::Stop { name } => {
            vm_manager.stop_vm(&name)?;
        }

        Cmd::Delete { name, force } => {
            vm_manager.delete_vm(&name, force)?;
        }

        Cmd::SetDisplay { name, mode, .. } => {
            println!("Setting display for VM '{}' to '{}' (not implemented in this example)", name, mode);
        }

        Cmd::Completions { shell } => {
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
        }

        Cmd::InstallFish => {
            install_fish_completions()?;
        }

        Cmd::ManPage => {
            generate_man_page()?;
        }
    }

    Ok(())
}