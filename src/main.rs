mod cli;
mod commands;
mod db;
mod indexing;
mod models;
mod output;
mod storage;
mod tui;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Command, LakeAction, NodeAction};
use commands::node::{CreateArgs, UpdateArgs};
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
                add_edge,
                add_source_file,
                edit,
            } => commands::node::create(
                &cwd,
                CreateArgs {
                    id,
                    content,
                    weight,
                    data_lake,
                    add_edge,
                    add_source_file,
                    edit,
                },
            ),
            NodeAction::Update {
                id,
                content,
                weight,
                add_data_lake,
                remove_data_lake,
                add_edge,
                remove_edge,
                add_source_file,
                remove_source_file,
                edit,
            } => commands::node::update(
                &cwd,
                UpdateArgs {
                    id,
                    content,
                    weight,
                    add_data_lake,
                    remove_data_lake,
                    add_edge,
                    remove_edge,
                    add_source_file,
                    remove_source_file,
                    edit,
                },
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
        Command::Completion { shell, install } => {
            if install {
                commands::completion::install(shell)?;
            } else {
                generate(shell, &mut Cli::command(), "engram", &mut std::io::stdout());
            }
            Ok(())
        }
        Command::Tui => tui::run(&cwd),
    }
}
