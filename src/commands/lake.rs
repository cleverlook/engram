use anyhow::{Result, bail};
use console::style;
use std::fs;
use std::path::Path;

use crate::db;
use crate::indexing;
use crate::storage;

pub fn add(path: &Path, file: &str, link: Option<&str>) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let lake_dir = engram_dir.join("data_lake");

    let source = Path::new(file);
    if !source.exists() {
        bail!("File not found: {}", file);
    }

    let filename = source
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;
    let dest = lake_dir.join(filename);

    if dest.exists() {
        bail!(
            "File '{}' already exists in data lake",
            filename.to_string_lossy()
        );
    }

    fs::copy(source, &dest)?;
    println!(
        "Added {} to data lake",
        style(filename.to_string_lossy()).bold()
    );

    // Link to node if specified
    if let Some(node_id) = link {
        let mut node = storage::load_node(&engram_dir, node_id)?;
        let dl_name = filename.to_string_lossy().to_string();
        if !node.data_lake.contains(&dl_name) {
            node.data_lake.push(dl_name.clone());
            storage::save_node(&engram_dir, &node)?;
            indexing::update_index_for_node(&engram_dir, &node)?;
            db::upsert_node(&engram_dir, &node)?;
            println!("Linked to node {}", style(node_id).bold());
        }
    }

    Ok(())
}

pub fn list(path: &Path) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let lake_dir = engram_dir.join("data_lake");

    let mut files: Vec<String> = Vec::new();
    if lake_dir.is_dir() {
        for entry in fs::read_dir(&lake_dir)? {
            let entry = entry?;
            if entry.path().is_file() {
                files.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    if files.is_empty() {
        println!("{}", style("Data lake is empty.").dim());
    } else {
        files.sort();
        println!(
            "{}",
            style(format!("Data lake ({} files):", files.len())).dim()
        );
        for f in &files {
            println!("  {}", f);
        }
    }

    Ok(())
}

pub fn remove(path: &Path, file: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let lake_path = engram_dir.join("data_lake").join(file);

    if !lake_path.exists() {
        bail!("File '{}' not found in data lake", file);
    }

    fs::remove_file(&lake_path)?;
    println!("Removed {} from data lake", style(file).bold());
    Ok(())
}
