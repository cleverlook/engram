mod cli;
mod commands;
mod db;
mod indexing;
mod models;
mod output;
mod storage;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Command, NodeAction};
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
        Command::Completion { shell } => {
            generate(shell, &mut Cli::command(), "engram", &mut std::io::stdout());
            Ok(())
        }
    }
}
