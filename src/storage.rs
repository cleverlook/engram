use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};

use crate::models::node::Node;

/// Returns the path to the .engram directory, searching upward from `start`.
pub fn find_engram_dir(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(".engram");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        if !current.pop() {
            bail!("No .engram/ directory found. Run `engram init` first.");
        }
    }
}

/// Maps a node id like "auth:oauth:google" to its YAML file path.
pub fn node_path(engram_dir: &Path, id: &str) -> PathBuf {
    let parts: Vec<&str> = id.split(':').collect();
    let mut path = engram_dir.join("nodes");
    for part in &parts {
        path = path.join(part);
    }
    path.with_extension("yaml")
}

/// Maps a node id to its namespace directory (parent of the node file).
pub fn namespace_dir(engram_dir: &Path, id: &str) -> PathBuf {
    node_path(engram_dir, id).parent().unwrap().to_path_buf()
}

pub fn load_node(engram_dir: &Path, id: &str) -> Result<Node> {
    let path = node_path(engram_dir, id);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Node '{}' not found at {}", id, path.display()))?;
    let node: Node = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse node '{}'", id))?;
    Ok(node)
}

pub fn save_node(engram_dir: &Path, node: &Node) -> Result<()> {
    let path = node_path(engram_dir, &node.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(node)
        .with_context(|| format!("Failed to serialize node '{}'", node.id))?;
    fs::write(&path, yaml)?;
    Ok(())
}
