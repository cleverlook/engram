use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::models::node::{Edge, Node, NodeStatus};
use crate::{db, indexing, storage};

pub fn create_node(engram_dir: &Path, id: &str, content: &str, weight: u8) -> Result<Node> {
    let now = Utc::now();
    let node = Node {
        id: id.to_string(),
        content: content.to_string(),
        weight,
        status: NodeStatus::Active,
        source_files: vec![],
        source_hash: None,
        created: now,
        touched: now,
        data_lake: vec![],
        edges: vec![],
    };

    let node_path = storage::node_path(engram_dir, &node.id);
    if node_path.exists() {
        let existing = storage::load_node(engram_dir, &node.id)?;
        if existing.status != NodeStatus::Deprecated {
            anyhow::bail!("Node '{}' already exists", node.id);
        }
    }

    storage::save_node(engram_dir, &node)?;
    indexing::update_index_for_node(engram_dir, &node)?;
    indexing::update_backlinks_for_node(engram_dir, &node)?;
    db::upsert_node(engram_dir, &node)?;
    Ok(node)
}

pub fn update_node_from_yaml(engram_dir: &Path, id: &str, yaml: &str) -> Result<Node> {
    let existing = storage::load_node(engram_dir, id)?;
    let mut node: Node = serde_yaml::from_str(yaml)?;
    node.touched = Utc::now();

    indexing::remove_backlinks_from_source(engram_dir, id, &existing)?;
    storage::save_node(engram_dir, &node)?;
    indexing::update_index_for_node(engram_dir, &node)?;
    indexing::update_backlinks_for_node(engram_dir, &node)?;
    db::upsert_node(engram_dir, &node)?;
    Ok(node)
}

pub fn deprecate_node(engram_dir: &Path, id: &str) -> Result<()> {
    let mut node = storage::load_node(engram_dir, id)?;
    node.status = NodeStatus::Deprecated;
    node.touched = Utc::now();

    storage::save_node(engram_dir, &node)?;
    indexing::remove_from_index(engram_dir, id)?;
    db::delete_node(engram_dir, id)?;
    Ok(())
}

pub fn add_edge(engram_dir: &Path, from_id: &str, edge: Edge) -> Result<Node> {
    let mut node = storage::load_node(engram_dir, from_id)?;
    node.edges.push(edge);
    node.touched = Utc::now();

    storage::save_node(engram_dir, &node)?;
    indexing::update_index_for_node(engram_dir, &node)?;
    indexing::update_backlinks_for_node(engram_dir, &node)?;
    db::upsert_node(engram_dir, &node)?;
    Ok(node)
}
