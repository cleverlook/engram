use anyhow::{Result, bail};
use chrono::Utc;
use dialoguer::{Confirm, Input, Select};
use std::io::{self, IsTerminal, Read};
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
    crate::output::print_node_full(&node);
    Ok(())
}

pub fn create(
    path: &Path,
    id: Option<String>,
    content: Option<String>,
    weight: u8,
    data_lake: Vec<String>,
    edit: bool,
) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let is_tty = io::stdin().is_terminal();

    let input = if edit {
        let id_str = id.unwrap_or_else(|| "namespace:node_name".to_string());
        let content_str = content.unwrap_or_else(|| "Describe the knowledge here.".to_string());
        let today = Utc::now().to_rfc3339();

        let template = NODE_TEMPLATE
            .replace("{id}", &id_str)
            .replace("{content}", &content_str)
            .replace("{weight}", &weight.to_string())
            .replace("{date}", &today);

        edit_in_editor(&template)?
    } else if let Some(id) = id {
        let content = content.unwrap_or_else(|| "TODO: add content".to_string());
        let today = Utc::now().to_rfc3339();

        NODE_TEMPLATE
            .replace("{id}", &id)
            .replace("{content}", &content)
            .replace("{weight}", &weight.to_string())
            .replace("{date}", &today)
    } else if is_tty {
        // Interactive mode
        interactive_create()?
    } else {
        // Stdin mode
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            bail!("No input on stdin.");
        }
        buf
    };

    let mut node: Node = serde_yaml::from_str(&input)?;

    for dl in data_lake {
        if !node.data_lake.contains(&dl) {
            node.data_lake.push(dl);
        }
    }

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
    add_data_lake: Vec<String>,
    remove_data_lake: Vec<String>,
    edit: bool,
) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let existing = storage::load_node(&engram_dir, id)?;
    let is_tty = io::stdin().is_terminal();

    let has_flags = content.is_some()
        || weight.is_some()
        || !add_data_lake.is_empty()
        || !remove_data_lake.is_empty();

    let mut node = if edit {
        let existing_yaml = serde_yaml::to_string(&existing)?;
        let input = edit_in_editor(&existing_yaml)?;
        serde_yaml::from_str(&input)?
    } else if has_flags {
        let mut node = existing.clone();
        if let Some(c) = content {
            node.content = c;
        }
        if let Some(w) = weight {
            node.weight = w;
        }
        for dl in &add_data_lake {
            if !node.data_lake.contains(dl) {
                node.data_lake.push(dl.clone());
            }
        }
        node.data_lake.retain(|dl| !remove_data_lake.contains(dl));
        node
    } else if is_tty {
        interactive_update(&existing)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            bail!("No input. Use --content, --weight, --edit, or pipe YAML to stdin.");
        }
        serde_yaml::from_str(&buf)?
    };

    node.touched = Utc::now();

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
    node.touched = Utc::now();

    storage::save_node(&engram_dir, &node)?;
    indexing::remove_from_index(&engram_dir, id)?;
    db::delete_node(&engram_dir, id)?;
    println!("Deprecated node '{}'", node.id);
    Ok(())
}

fn interactive_create() -> Result<String> {
    let id: String = Input::new()
        .with_prompt("Node id (e.g. auth:oauth:google)")
        .interact_text()?;

    let content: String = Input::new().with_prompt("Content").interact_text()?;

    let weight: String = Input::new()
        .with_prompt("Weight (0-100)")
        .default("50".to_string())
        .interact_text()?;

    let today = Utc::now().to_rfc3339();

    let mut yaml = NODE_TEMPLATE
        .replace("{id}", &id)
        .replace("{content}", &content)
        .replace("{weight}", &weight)
        .replace("{date}", &today);

    // Ask about edges
    while Confirm::new()
        .with_prompt("Add an edge?")
        .default(false)
        .interact()?
    {
        let target: String = Input::new()
            .with_prompt("  Target node id")
            .interact_text()?;

        let edge_types = &["uses", "depends_on", "implements", "rationale", "related"];
        let edge_type_idx = Select::new()
            .with_prompt("  Edge type")
            .items(edge_types)
            .default(0)
            .interact()?;

        let edge_weight: String = Input::new()
            .with_prompt("  Edge weight (0-100)")
            .default("50".to_string())
            .interact_text()?;

        // Append edge to yaml
        if !yaml.contains("edges:") || yaml.contains("# edges:") {
            yaml = yaml.replace("# edges:", "edges:");
            yaml = yaml.replace(
                "#   - to: namespace:node_id\n#     type: uses\n#     weight: 50",
                &format!(
                    "  - to: {}\n    type: {}\n    weight: {}",
                    target, edge_types[edge_type_idx], edge_weight
                ),
            );
        } else {
            yaml.push_str(&format!(
                "  - to: {}\n    type: {}\n    weight: {}\n",
                target, edge_types[edge_type_idx], edge_weight
            ));
        }
    }

    Ok(yaml)
}

fn interactive_update(existing: &Node) -> Result<Node> {
    let mut node = existing.clone();

    let content: String = Input::new()
        .with_prompt("Content")
        .default(node.content.clone())
        .interact_text()?;
    node.content = content;

    let weight: String = Input::new()
        .with_prompt("Weight (0-100)")
        .default(node.weight.to_string())
        .interact_text()?;
    node.weight = weight.parse().unwrap_or(node.weight);

    let statuses = &["active", "dirty", "stale", "deprecated"];
    let current_idx = match node.status {
        NodeStatus::Active => 0,
        NodeStatus::Dirty => 1,
        NodeStatus::Stale => 2,
        NodeStatus::Deprecated => 3,
    };
    let status_idx = Select::new()
        .with_prompt("Status")
        .items(statuses)
        .default(current_idx)
        .interact()?;
    node.status = match status_idx {
        0 => NodeStatus::Active,
        1 => NodeStatus::Dirty,
        2 => NodeStatus::Stale,
        3 => NodeStatus::Deprecated,
        _ => NodeStatus::Active,
    };

    Ok(node)
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
