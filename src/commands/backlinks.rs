use std::fs;
use std::path::Path;
use anyhow::Result;

use crate::models::backlinks::NamespaceBacklinks;
use crate::storage;

pub fn run(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let ns_dir = storage::namespace_dir(&engram_dir, id);
    let bl_path = ns_dir.join("_backlinks.yaml");

    if !bl_path.exists() {
        println!("No backlinks found for '{}'.", id);
        return Ok(());
    }

    let content = fs::read_to_string(&bl_path)?;
    let bl: NamespaceBacklinks = serde_yaml::from_str(&content)?;

    let node_bl = bl.backlinks.iter().find(|b| b.node == id);

    match node_bl {
        Some(entry) => {
            println!("Backlinks for '{}':", id);
            for incoming in &entry.incoming {
                println!("  <- {} ({}, weight: {})", incoming.from, incoming.edge_type, incoming.weight);
            }
        }
        None => {
            println!("No backlinks found for '{}'.", id);
        }
    }

    Ok(())
}
