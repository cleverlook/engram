use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

use crate::db;
use crate::indexing;
use crate::storage;

pub fn run(path: &Path) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;

    // Load all nodes
    let nodes = storage::load_all_nodes(&engram_dir)?;
    println!("Found {} nodes.", nodes.len());

    // Clean existing index and backlinks files
    clean_derived_files(&engram_dir.join("nodes"))?;

    // Rebuild _index.yaml at each namespace level
    for node in &nodes {
        indexing::update_index_for_node(&engram_dir, node)?;
    }
    println!("Rebuilt _index.yaml files.");

    // Rebuild _backlinks.yaml
    for node in &nodes {
        indexing::update_backlinks_for_node(&engram_dir, node)?;
    }
    println!("Rebuilt _backlinks.yaml files.");

    // Rebuild SQLite
    db::rebuild(&engram_dir)?;
    println!("Rebuilt SQLite FTS index.");

    println!("Done.");
    Ok(())
}

/// Remove all _index.yaml and _backlinks.yaml files so they can be rebuilt fresh.
fn clean_derived_files(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            clean_derived_files(&path)?;
        } else if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename == "_index.yaml" || filename == "_backlinks.yaml" {
                fs::remove_file(&path)?;
            }
        }
    }

    Ok(())
}
