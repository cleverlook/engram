mod cli;
mod commands;
mod db;
mod indexing;
mod models;
mod output;
mod storage;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Command, LakeAction, NodeAction};
use std::env;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir()?;

    match cli.command {
        Command::Init => commands::init::run(&cwd),
        Command::Node { action } => match action {
            NodeAction::Get { id } => commands::node::get(&cwd, &id),
            NodeAction::Create {
                id,
                content,
                weight,
                data_lake,
                edit,
            } => commands::node::create(&cwd, id, content, weight, data_lake, edit),
            NodeAction::Update {
                id,
                content,
                weight,
                add_data_lake,
                remove_data_lake,
                edit,
            } => commands::node::update(
                &cwd,
                &id,
                content,
                weight,
                add_data_lake,
                remove_data_lake,
                edit,
            ),
            NodeAction::Deprecate { id } => commands::node::deprecate(&cwd, &id),
        },
        Command::Search { query } => commands::search::run(&cwd, &query),
        Command::Traverse {
            id,
            depth,
            min_weight,
            budget,
        } => commands::traverse::run(&cwd, &id, depth, min_weight, budget),
        Command::Backlinks { id } => commands::backlinks::run(&cwd, &id),
        Command::Status => commands::status::run(&cwd),
        Command::Check => commands::check::run(&cwd),
        Command::RebuildIndex => commands::rebuild_index::run(&cwd),
        Command::Lake { action } => match action {
            LakeAction::Add { file, link } => commands::lake::add(&cwd, &file, link.as_deref()),
            LakeAction::List => commands::lake::list(&cwd),
            LakeAction::Remove { file } => commands::lake::remove(&cwd, &file),
        },
        Command::Completion { shell } => {
            generate(shell, &mut Cli::command(), "engram", &mut std::io::stdout());
            Ok(())
        }
        Command::GenerateCompletions { outdir } => {
            let outdir = std::path::PathBuf::from(&outdir);
            std::fs::create_dir_all(&outdir)?;
            let mut cmd = Cli::command();
            for shell in [
                clap_complete::Shell::Bash,
                clap_complete::Shell::Zsh,
                clap_complete::Shell::Fish,
                clap_complete::Shell::PowerShell,
            ] {
                clap_complete::generate_to(shell, &mut cmd, "engram", &outdir)?;
            }
            println!("Generated completions in {}", outdir.display());
            Ok(())
        }
    }
}
