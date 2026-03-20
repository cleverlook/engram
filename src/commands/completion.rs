use anyhow::{Result, bail};
use clap::CommandFactory;
use clap_complete::Shell;
use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::output;

pub fn install(shell: Shell) -> Result<()> {
    let (dir, filename) = install_target(&shell)?;

    fs::create_dir_all(&dir)?;
    let path = dir.join(&filename);

    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "engram", &mut buf);
    fs::write(&path, buf)?;

    output::print_success(&format!(
        "Installed {} completion to {}",
        shell,
        path.display()
    ));
    output::print_info(&post_install_hint(&shell));

    Ok(())
}

fn install_target(shell: &Shell) -> Result<(PathBuf, String)> {
    let home = home_dir()?;
    match shell {
        Shell::Zsh => Ok((home.join(".zfunc"), "_engram".into())),
        Shell::Bash => Ok((
            home.join(".local/share/bash-completion/completions"),
            "engram".into(),
        )),
        Shell::Fish => Ok((home.join(".config/fish/completions"), "engram.fish".into())),
        Shell::PowerShell => Ok((
            home.join(".config/powershell/completions"),
            "_engram.ps1".into(),
        )),
        _ => bail!("Unsupported shell: {shell}"),
    }
}

fn post_install_hint(shell: &Shell) -> String {
    match shell {
        Shell::Zsh => "Ensure ~/.zfunc is in fpath. Add to .zshrc:\n  \
             fpath+=~/.zfunc; autoload -Uz compinit; compinit"
            .into(),
        Shell::Bash => "Completions will load automatically in new shells.".into(),
        Shell::Fish => "Completions will load automatically in new shells.".into(),
        Shell::PowerShell => "Add to your PowerShell profile:\n  \
             Get-ChildItem ~/.config/powershell/completions/*.ps1 | ForEach-Object { . $_ }"
            .into(),
        _ => String::new(),
    }
}

fn home_dir() -> Result<PathBuf> {
    match std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        Some(h) => Ok(PathBuf::from(h)),
        None => bail!("Cannot determine home directory"),
    }
}
