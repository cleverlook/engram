use clap::CommandFactory;
use std::env;
use std::path::PathBuf;

include!("src/cli.rs");

fn main() {
    let outdir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut cmd = Cli::command();

    for shell in [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
        clap_complete::Shell::PowerShell,
    ] {
        clap_complete::generate_to(shell, &mut cmd, "engram", &outdir).unwrap();
    }

    println!("cargo::rerun-if-changed=src/cli.rs");
}
