use anyhow::Result;
use chrono::Utc;
use console::style;
use std::fs;
use std::path::Path;

use crate::models::node::NodeStatus;
use crate::storage;

pub fn run(path: &Path) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let nodes = storage::load_all_nodes(&engram_dir)?;
    let today = Utc::now();

    let mut dirty_count = 0u32;
    let mut stale_count = 0u32;
    let mut decayed_count = 0u32;

    for mut node in nodes {
        let mut changed = false;

        // Dirty detection: check source_hash against current files
        if !node.source_files.is_empty()
            && let Some(ref stored_hash) = node.source_hash
        {
            let current_hash = compute_source_hash(path, &node.source_files);
            if let Some(ref hash) = current_hash
                && hash != stored_hash
                && node.status == NodeStatus::Active
            {
                node.status = NodeStatus::Dirty;
                changed = true;
                dirty_count += 1;
                println!(
                    "  {} {} (source files changed)",
                    style("dirty").yellow(),
                    style(&node.id).bold()
                );
            }
        }

        // Stale detection
        let days_untouched = (today - node.touched).num_days();
        if days_untouched > 30 && node.status == NodeStatus::Active {
            stale_count += 1;
            println!(
                "  {} {} ({} days untouched)",
                style("stale").yellow(),
                style(&node.id).bold(),
                days_untouched
            );
        }

        // Weight decay
        if days_untouched > 90 {
            let new_weight = node.weight.saturating_sub(5);
            if new_weight != node.weight {
                node.weight = new_weight;
                changed = true;
                decayed_count += 1;
            }
        } else if days_untouched > 30 {
            let new_weight = node.weight.saturating_sub(1);
            if new_weight != node.weight {
                node.weight = new_weight;
                changed = true;
                decayed_count += 1;
            }
        }

        if changed {
            storage::save_node(&engram_dir, &node)?;
        }
    }

    println!();
    println!(
        "Status: {} dirty, {} stale, {} decayed",
        style(dirty_count).yellow().bold(),
        style(stale_count).yellow(),
        style(decayed_count).dim()
    );
    Ok(())
}

/// Compute MD5 hash of concatenated source files contents.
fn compute_source_hash(project_root: &Path, source_files: &[String]) -> Option<String> {
    let mut combined = String::new();
    for file in source_files {
        let file_path = project_root.join(file);
        match fs::read_to_string(&file_path) {
            Ok(content) => combined.push_str(&content),
            Err(_) => return None, // file doesn't exist, can't compute hash
        }
    }
    Some(format!("{:x}", md5::compute(combined.as_bytes())))
}
