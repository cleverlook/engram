use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::path::Path;
use anyhow::Result;
use chrono::Local;

use crate::db;
use crate::models::node::NodeStatus;
use crate::storage;

#[derive(Eq, PartialEq)]
struct QueueEntry {
    id: String,
    priority: u8,
    depth: u32,
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn run(path: &Path, id: &str, max_depth: u32, min_weight: u8, budget: usize) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let today = Local::now().date_naive();

    let mut visited = HashSet::new();
    let mut queue = BinaryHeap::new();
    let mut remaining_budget = budget;

    queue.push(QueueEntry {
        id: id.to_string(),
        priority: 100,
        depth: 0,
    });

    while let Some(entry) = queue.pop() {
        if remaining_budget == 0 {
            break;
        }

        if visited.contains(&entry.id) {
            continue;
        }

        if entry.depth > max_depth {
            continue;
        }

        let mut node = match storage::load_node(&engram_dir, &entry.id) {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Warning: node '{}' not found, skipping", entry.id);
                continue;
            }
        };

        if node.status == NodeStatus::Deprecated {
            continue;
        }

        visited.insert(entry.id.clone());

        // Print node
        let stale_marker = if node.status == NodeStatus::Dirty {
            " [POSSIBLY STALE]"
        } else {
            ""
        };
        println!("--- {} (weight: {}, depth: {}){} ---", node.id, node.weight, entry.depth, stale_marker);
        println!("{}", node.content.trim());
        println!();

        // Deduct from budget
        let content_len = node.content.len();
        if content_len >= remaining_budget {
            remaining_budget = 0;
        } else {
            remaining_budget -= content_len;
        }

        // Update touched and weight
        node.touched = today;
        node.weight = node.weight.saturating_add(1).min(100);
        storage::save_node(&engram_dir, &node)?;
        let _ = db::upsert_node(&engram_dir, &node);

        // Enqueue edges
        for edge in &node.edges {
            if edge.weight >= min_weight && !visited.contains(&edge.to) {
                queue.push(QueueEntry {
                    id: edge.to.clone(),
                    priority: edge.weight,
                    depth: entry.depth + 1,
                });
            }
        }
    }

    Ok(())
}
