use std::collections::{HashMap, HashSet};
use std::path::Path;
use anyhow::Result;
use chrono::Local;

use crate::storage;

pub fn run(path: &Path) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let nodes = storage::load_all_nodes(&engram_dir)?;
    let today = Local::now().date_naive();

    let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

    // Track incoming edges per node
    let mut has_incoming: HashSet<&str> = HashSet::new();
    for node in &nodes {
        for edge in &node.edges {
            has_incoming.insert(&edge.to);
        }
    }

    let mut issues = 0u32;

    for node in &nodes {
        // Broken edges
        for edge in &node.edges {
            if !node_ids.contains(edge.to.as_str()) {
                println!("  broken edge: {} -> {} (node not found)", node.id, edge.to);
                issues += 1;
            }
        }

        // Broken data_lake refs
        for dl_ref in &node.data_lake {
            let dl_path = engram_dir.join("data_lake").join(dl_ref);
            if !dl_path.exists() {
                println!("  broken data_lake ref: {} -> {} (file not found)", node.id, dl_ref);
                issues += 1;
            }
        }

        // Orphan nodes
        let has_outgoing = !node.edges.is_empty();
        let has_inc = has_incoming.contains(node.id.as_str());
        let days_untouched = (today - node.touched).num_days();

        if !has_outgoing && !has_inc && node.weight < 10 && days_untouched > 180 {
            println!("  orphan: {} (no edges, weight {}, {} days untouched)", node.id, node.weight, days_untouched);
            issues += 1;
        }
    }

    if issues == 0 {
        println!("Graph integrity check passed. No issues found.");
    } else {
        println!("\nFound {} issue(s).", issues);
    }

    Ok(())
}
