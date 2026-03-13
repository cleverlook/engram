use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::models::backlinks::{IncomingEdge, NamespaceBacklinks, NodeBacklinks};
use crate::models::index::{IndexEntry, NamespaceIndex, NamespaceSummary};
use crate::models::node::Node;
use crate::storage;

/// Extract the namespace from a node id. "auth:oauth:google" -> "auth:oauth"
/// Single-segment ids like "readme" have no namespace (top-level).
fn parent_namespace(id: &str) -> Option<&str> {
    id.rsplit_once(':').map(|(ns, _)| ns)
}

/// Extract the top-level namespace from a node id. "auth:oauth:google" -> "auth"
fn top_namespace(id: &str) -> &str {
    id.split(':').next().unwrap()
}

/// Load a namespace index from disk, or return a default empty one.
fn load_index(path: &Path) -> Result<NamespaceIndex> {
    if path.exists() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let index: NamespaceIndex = serde_yaml::from_str(&content)?;
        Ok(index)
    } else {
        Ok(NamespaceIndex {
            namespace: None,
            source_paths: None,
            nodes: Vec::new(),
            namespaces: Vec::new(),
        })
    }
}

fn save_index(path: &Path, index: &NamespaceIndex) -> Result<()> {
    let yaml = serde_yaml::to_string(index)?;
    fs::write(path, yaml)?;
    Ok(())
}

fn load_backlinks(path: &Path) -> Result<NamespaceBacklinks> {
    if path.exists() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let bl: NamespaceBacklinks = serde_yaml::from_str(&content)?;
        Ok(bl)
    } else {
        Ok(NamespaceBacklinks {
            namespace: String::new(),
            backlinks: Vec::new(),
        })
    }
}

fn save_backlinks(path: &Path, bl: &NamespaceBacklinks) -> Result<()> {
    let yaml = serde_yaml::to_string(bl)?;
    fs::write(path, yaml)?;
    Ok(())
}

/// Update the namespace _index.yaml where this node lives, adding or updating the entry.
pub fn update_index_for_node(engram_dir: &Path, node: &Node) -> Result<()> {
    let ns_dir = storage::namespace_dir(engram_dir, &node.id);
    fs::create_dir_all(&ns_dir)?;
    let index_path = ns_dir.join("_index.yaml");

    let mut index = load_index(&index_path)?;
    index.namespace = parent_namespace(&node.id).map(|s| s.to_string());

    // Update or add node entry
    let status_str = serde_yaml::to_string(&node.status)?;
    let entry = IndexEntry {
        id: node.id.clone(),
        weight: node.weight,
        status: status_str.trim().to_string(),
        tags: Vec::new(),
    };

    if let Some(existing) = index.nodes.iter_mut().find(|n| n.id == node.id) {
        *existing = entry;
    } else {
        index.nodes.push(entry);
    }

    save_index(&index_path, &index)?;

    // Update top-level index
    update_top_level_index(engram_dir, node)?;

    Ok(())
}

/// Remove a node entry from its namespace _index.yaml.
pub fn remove_from_index(engram_dir: &Path, node_id: &str) -> Result<()> {
    let ns_dir = storage::namespace_dir(engram_dir, node_id);
    let index_path = ns_dir.join("_index.yaml");

    if index_path.exists() {
        let mut index = load_index(&index_path)?;
        index.nodes.retain(|n| n.id != node_id);
        save_index(&index_path, &index)?;
    }

    Ok(())
}

/// Update the top-level _index.yaml with namespace summaries.
fn update_top_level_index(engram_dir: &Path, node: &Node) -> Result<()> {
    let top_index_path = engram_dir.join("nodes").join("_index.yaml");
    let mut index = load_index(&top_index_path)?;

    let top_ns = top_namespace(&node.id);

    if let Some(existing) = index.namespaces.iter_mut().find(|ns| ns.name == top_ns) {
        // Increment count
        let count = existing.node_count.unwrap_or(0);
        existing.node_count = Some(count + 1);
    } else {
        index.namespaces.push(NamespaceSummary {
            name: top_ns.to_string(),
            node_count: Some(1),
            tags: Vec::new(),
        });
    }

    save_index(&top_index_path, &index)?;
    Ok(())
}

/// Update _backlinks.yaml for all target nodes referenced by this node's edges.
pub fn update_backlinks_for_node(engram_dir: &Path, node: &Node) -> Result<()> {
    for edge in &node.edges {
        let target_ns_dir = storage::namespace_dir(engram_dir, &edge.to);
        fs::create_dir_all(&target_ns_dir)?;
        let bl_path = target_ns_dir.join("_backlinks.yaml");

        let target_ns = parent_namespace(&edge.to).unwrap_or("").to_string();

        let mut bl = load_backlinks(&bl_path)?;
        bl.namespace = target_ns;

        let edge_type_str = serde_yaml::to_string(&edge.edge_type)?;
        let incoming = IncomingEdge {
            from: node.id.clone(),
            edge_type: edge_type_str.trim().to_string(),
            weight: edge.weight,
        };

        // Find or create the NodeBacklinks entry for the target
        if let Some(node_bl) = bl.backlinks.iter_mut().find(|b| b.node == edge.to) {
            // Remove existing backlink from this source if any
            node_bl.incoming.retain(|i| i.from != node.id);
            node_bl.incoming.push(incoming);
        } else {
            bl.backlinks.push(NodeBacklinks {
                node: edge.to.clone(),
                incoming: vec![incoming],
            });
        }

        save_backlinks(&bl_path, &bl)?;
    }

    Ok(())
}

/// Remove all backlinks originating from a given source node.
pub fn remove_backlinks_from_source(engram_dir: &Path, source_id: &str, node: &Node) -> Result<()> {
    for edge in &node.edges {
        let target_ns_dir = storage::namespace_dir(engram_dir, &edge.to);
        let bl_path = target_ns_dir.join("_backlinks.yaml");

        if bl_path.exists() {
            let mut bl = load_backlinks(&bl_path)?;
            for node_bl in &mut bl.backlinks {
                node_bl.incoming.retain(|i| i.from != source_id);
            }
            bl.backlinks.retain(|b| !b.incoming.is_empty());
            save_backlinks(&bl_path, &bl)?;
        }
    }

    Ok(())
}
