//! Shell completion utilities

use crate::cli::commands::Cli;
use crate::Result;
use anyhow::anyhow;
use clap::{CommandFactory};
use clap_complete::{generate, Generator, Shell};
use std::fs::{self, File};

/// Generate shell completions
pub fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

/// Install Fish completions automatically
pub fn install_fish_completions() -> Result<()> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?;

    let fish_dir = home_dir
        .join(".config")
        .join("fish")
        .join("completions");

    fs::create_dir_all(&fish_dir)?;

    let completion_file = fish_dir.join("qvm.fish");
    let mut cmd = Cli::command();
    let mut file = File::create(&completion_file)?;

    generate(Shell::Fish, &mut cmd, "qvm", &mut file);

    println!("Fish completions installed to: {}", completion_file.display());
    Ok(())
}

/// Generate man page
pub fn generate_man_page() -> Result<()> {
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer)?;

    print!("{}", String::from_utf8(buffer)?);
    Ok(())
}