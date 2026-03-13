use std::io::{self, Read};
use std::path::Path;
use anyhow::{bail, Result};
use chrono::Local;

use crate::indexing;
use crate::models::node::{Node, NodeStatus};
use crate::storage;

pub fn get(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let node = storage::load_node(&engram_dir, id)?;
    let yaml = serde_yaml::to_string(&node)?;
    print!("{}", yaml);
    Ok(())
}

pub fn create(path: &Path) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        bail!("No input provided. Pipe YAML to stdin.");
    }

    let node: Node = serde_yaml::from_str(&input)?;

    // Check if node already exists
    let node_path = storage::node_path(&engram_dir, &node.id);
    if node_path.exists() {
        bail!("Node '{}' already exists", node.id);
    }

    storage::save_node(&engram_dir, &node)?;
    indexing::update_index_for_node(&engram_dir, &node)?;
    indexing::update_backlinks_for_node(&engram_dir, &node)?;
    println!("Created node '{}'", node.id);
    Ok(())
}

pub fn update(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;

    // Load existing to clean up old backlinks
    let existing = storage::load_node(&engram_dir, id)?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        bail!("No input provided. Pipe YAML to stdin.");
    }

    let mut node: Node = serde_yaml::from_str(&input)?;
    node.touched = Local::now().date_naive();

    // Remove old backlinks, then add new ones
    indexing::remove_backlinks_from_source(&engram_dir, id, &existing)?;
    storage::save_node(&engram_dir, &node)?;
    indexing::update_index_for_node(&engram_dir, &node)?;
    indexing::update_backlinks_for_node(&engram_dir, &node)?;
    println!("Updated node '{}'", node.id);
    Ok(())
}

pub fn deprecate(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let mut node = storage::load_node(&engram_dir, id)?;

    node.status = NodeStatus::Deprecated;
    node.touched = Local::now().date_naive();

    storage::save_node(&engram_dir, &node)?;
    indexing::update_index_for_node(&engram_dir, &node)?;
    println!("Deprecated node '{}'", node.id);
    Ok(())
}
