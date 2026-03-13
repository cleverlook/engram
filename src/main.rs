mod cli;
mod commands;
mod models;

use std::env;
use clap::Parser;
use cli::{Cli, Command, NodeAction};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir()?;

    match cli.command {
        Command::Init => commands::init::run(&cwd),
        Command::Node { action } => match action {
            NodeAction::Get { id } => commands::node::get(&cwd, &id),
            NodeAction::Create => commands::node::create(&cwd),
            NodeAction::Update { id } => commands::node::update(&cwd, &id),
            NodeAction::Deprecate { id } => commands::node::deprecate(&cwd, &id),
        },
        Command::Search { query } => commands::search::run(&cwd, &query),
        Command::Traverse { id, depth, min_weight, budget } => {
            commands::traverse::run(&cwd, &id, depth, min_weight, budget)
        }
        Command::Backlinks { id } => commands::backlinks::run(&cwd, &id),
        Command::Status => commands::status::run(&cwd),
        Command::Check => commands::check::run(&cwd),
        Command::RebuildIndex => commands::rebuild_index::run(&cwd),
    }
}
