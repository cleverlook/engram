use anyhow::Result;
use chrono::Utc;
use console::style;
use std::collections::HashSet;
use std::path::Path;

use crate::storage;

pub fn run(path: &Path) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let nodes = storage::load_all_nodes(&engram_dir)?;
    let today = Utc::now();

    let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

    let mut has_incoming: HashSet<&str> = HashSet::new();
    for node in &nodes {
        for edge in &node.edges {
            has_incoming.insert(&edge.to);
        }
    }

    let mut issues = 0u32;

    for node in &nodes {
        for edge in &node.edges {
            if !node_ids.contains(edge.to.as_str()) {
                println!(
                    "  {} {} → {} (node not found)",
                    style("broken edge").red(),
                    style(&node.id).bold(),
                    edge.to
                );
                issues += 1;
            }
        }

        for dl_ref in &node.data_lake {
            let dl_path = engram_dir.join("data_lake").join(dl_ref);
            if !dl_path.exists() {
                println!(
                    "  {} {} → {} (file not found)",
                    style("broken data_lake").red(),
                    style(&node.id).bold(),
                    dl_ref
                );
                issues += 1;
            }
        }

        let has_outgoing = !node.edges.is_empty();
        let has_inc = has_incoming.contains(node.id.as_str());
        let days_untouched = (today - node.touched).num_days();

        if !has_outgoing && !has_inc && node.weight < 10 && days_untouched > 180 {
            println!(
                "  {} {} (no edges, w:{}, {} days untouched)",
                style("orphan").yellow(),
                style(&node.id).bold(),
                node.weight,
                days_untouched
            );
            issues += 1;
        }
    }

    if issues == 0 {
        println!("{}", style("Graph integrity check passed.").green());
    } else {
        println!("\nFound {} issue(s).", style(issues).red().bold());
    }

    Ok(())
}
