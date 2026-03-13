use anyhow::{Result, bail};
use chrono::Local;
use std::io::{self, Read};
use std::path::Path;

use crate::db;
use crate::indexing;
use crate::models::node::{Node, NodeStatus};
use crate::storage;

const NODE_TEMPLATE: &str = r#"id: {id}
content: |
  {content}
weight: {weight}
status: active
# source_files: []
# source_hash:
created: {date}
touched: {date}
# edges:
#   - to: namespace:node_id
#     type: uses
#     weight: 50
"#;

pub fn get(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let node = storage::load_node(&engram_dir, id)?;
    let yaml = serde_yaml::to_string(&node)?;
    print!("{}", yaml);
    Ok(())
}

pub fn create(
    path: &Path,
    id: Option<String>,
    content: Option<String>,
    weight: u8,
    edit: bool,
) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;

    let input = if edit {
        // Mode 3: open $EDITOR with template
        let id_str = id.unwrap_or_else(|| "namespace:node_name".to_string());
        let content_str = content.unwrap_or_else(|| "Describe the knowledge here.".to_string());
        let today = Local::now().date_naive().to_string();

        let template = NODE_TEMPLATE
            .replace("{id}", &id_str)
            .replace("{content}", &content_str)
            .replace("{weight}", &weight.to_string())
            .replace("{date}", &today);

        edit_in_editor(&template)?
    } else if let Some(id) = id {
        // Mode 2: flags
        let content = content.unwrap_or_else(|| "TODO: add content".to_string());
        let today = Local::now().date_naive().to_string();

        NODE_TEMPLATE
            .replace("{id}", &id)
            .replace("{content}", &content)
            .replace("{weight}", &weight.to_string())
            .replace("{date}", &today)
    } else {
        // Mode 1: stdin
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            bail!(
                "No input. Use: engram node create <id> --content \"...\" or --edit or pipe YAML to stdin."
            );
        }
        buf
    };

    let node: Node = serde_yaml::from_str(&input)?;

    let node_path = storage::node_path(&engram_dir, &node.id);
    if node_path.exists() {
        bail!("Node '{}' already exists", node.id);
    }

    storage::save_node(&engram_dir, &node)?;
    indexing::update_index_for_node(&engram_dir, &node)?;
    indexing::update_backlinks_for_node(&engram_dir, &node)?;
    db::upsert_node(&engram_dir, &node)?;
    println!("Created node '{}'", node.id);
    Ok(())
}

pub fn update(
    path: &Path,
    id: &str,
    content: Option<String>,
    weight: Option<u8>,
    edit: bool,
) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let existing = storage::load_node(&engram_dir, id)?;

    let mut node = if edit {
        // Mode: open in editor
        let existing_yaml = serde_yaml::to_string(&existing)?;
        let input = edit_in_editor(&existing_yaml)?;
        serde_yaml::from_str(&input)?
    } else if content.is_some() || weight.is_some() {
        // Mode: partial update via flags
        let mut node = existing.clone();
        if let Some(c) = content {
            node.content = c;
        }
        if let Some(w) = weight {
            node.weight = w;
        }
        node
    } else {
        // Mode: full YAML from stdin
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            bail!("No input. Use --content, --weight, --edit, or pipe YAML to stdin.");
        }
        serde_yaml::from_str(&buf)?
    };

    node.touched = Local::now().date_naive();

    indexing::remove_backlinks_from_source(&engram_dir, id, &existing)?;
    storage::save_node(&engram_dir, &node)?;
    indexing::update_index_for_node(&engram_dir, &node)?;
    indexing::update_backlinks_for_node(&engram_dir, &node)?;
    db::upsert_node(&engram_dir, &node)?;
    println!("Updated node '{}'", node.id);
    Ok(())
}

pub fn deprecate(path: &Path, id: &str) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let mut node = storage::load_node(&engram_dir, id)?;

    node.status = NodeStatus::Deprecated;
    node.touched = Local::now().date_naive();

    storage::save_node(&engram_dir, &node)?;
    indexing::remove_from_index(&engram_dir, id)?;
    db::delete_node(&engram_dir, id)?;
    println!("Deprecated node '{}'", node.id);
    Ok(())
}

fn edit_in_editor(template: &str) -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let tmp = std::env::temp_dir().join(format!("engram-{}.yaml", std::process::id()));

    std::fs::write(&tmp, template)?;

    let status = std::process::Command::new(&editor).arg(&tmp).status()?;

    if !status.success() {
        std::fs::remove_file(&tmp).ok();
        bail!("Editor exited with error");
    }

    let content = std::fs::read_to_string(&tmp)?;
    std::fs::remove_file(&tmp).ok();

    if content.trim().is_empty() || content == template {
        bail!("Aborted: no changes made");
    }

    Ok(content)
}
