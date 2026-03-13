use anyhow::Result;
use std::path::Path;

use crate::db;
use crate::storage;

pub fn run(path: &Path, query: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let ids = db::search(&engram_dir, query)?;

    if ids.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    for id in &ids {
        let node = storage::load_node(&engram_dir, id)?;
        println!(
            "--- {} (weight: {}, status: {:?}) ---",
            node.id, node.weight, node.status
        );
        println!("{}", node.content.trim());
        println!();
    }

    Ok(())
}
