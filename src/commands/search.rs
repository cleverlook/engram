use anyhow::Result;
use console::style;
use std::path::Path;

use crate::db;
use crate::output;
use crate::storage;

pub fn run(path: &Path, query: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let ids = db::search(&engram_dir, query)?;

    if ids.is_empty() {
        println!("{}", style("No results found.").dim());
        return Ok(());
    }

    println!("{}", style(format!("Found {} result(s):", ids.len())).dim());
    println!();

    for (i, id) in ids.iter().enumerate() {
        let node = storage::load_node(&engram_dir, id)?;
        output::print_node_header(&node);
        println!("  {}", node.content.trim().lines().next().unwrap_or(""));
        if i < ids.len() - 1 {
            println!();
        }
    }

    Ok(())
}
