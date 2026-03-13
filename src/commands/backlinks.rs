use anyhow::Result;
use console::style;
use std::fs;
use std::path::Path;

use crate::models::backlinks::NamespaceBacklinks;
use crate::storage;

pub fn run(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let ns_dir = storage::namespace_dir(&engram_dir, id);
    let bl_path = ns_dir.join("_backlinks.yaml");

    if !bl_path.exists() {
        println!(
            "{}",
            style(format!("No backlinks found for '{}'.", id)).dim()
        );
        return Ok(());
    }

    let content = fs::read_to_string(&bl_path)?;
    let bl: NamespaceBacklinks = serde_yaml::from_str(&content)?;

    let node_bl = bl.backlinks.iter().find(|b| b.node == id);

    match node_bl {
        Some(entry) => {
            println!("Backlinks for {}:", style(id).bold());
            for incoming in &entry.incoming {
                println!(
                    "  {} {} {} {}",
                    style("←").cyan(),
                    style(&incoming.from).bold(),
                    style(&incoming.edge_type).dim(),
                    style(format!("w:{}", incoming.weight)).dim(),
                );
            }
        }
        None => {
            println!(
                "{}",
                style(format!("No backlinks found for '{}'.", id)).dim()
            );
        }
    }

    Ok(())
}
