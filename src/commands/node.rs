use anyhow::{Context, Result, bail};
use chrono::Utc;
use dialoguer::{Confirm, Input, Select};
use serde_yaml::Value;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use crate::db;
use crate::indexing;
use crate::models::node::{Node, NodeStatus};
use crate::storage;

pub struct CreateArgs {
    pub id: Option<String>,
    pub content: Option<String>,
    pub weight: u8,
    pub data_lake: Vec<String>,
    pub add_edge: Vec<String>,
    pub add_source_file: Vec<String>,
    pub edit: bool,
}

pub struct UpdateArgs {
    pub id: String,
    pub content: Option<String>,
    pub weight: Option<u8>,
    pub add_data_lake: Vec<String>,
    pub remove_data_lake: Vec<String>,
    pub add_edge: Vec<String>,
    pub remove_edge: Vec<String>,
    pub add_source_file: Vec<String>,
    pub remove_source_file: Vec<String>,
    pub edit: bool,
}

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

pub fn create(path: &Path, args: CreateArgs) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let is_tty = io::stdin().is_terminal();
    let mut is_stdin_input = false;

    let input = if args.edit {
        let id_str = args.id.unwrap_or_else(|| "namespace:node_name".to_string());
        let content_str = args
            .content
            .unwrap_or_else(|| "Describe the knowledge here.".to_string());
        let today = Utc::now().to_rfc3339();
        let indented = indent_content(&content_str);

        let template = NODE_TEMPLATE
            .replace("{id}", &id_str)
            .replace("{content}", &indented)
            .replace("{weight}", &args.weight.to_string())
            .replace("{date}", &today);

        edit_in_editor(&template)?
    } else if let Some(id) = args.id {
        let content = args
            .content
            .unwrap_or_else(|| "TODO: add content".to_string());
        let today = Utc::now().to_rfc3339();
        let indented = indent_content(&content);

        NODE_TEMPLATE
            .replace("{id}", &id)
            .replace("{content}", &indented)
            .replace("{weight}", &args.weight.to_string())
            .replace("{date}", &today)
    } else if is_tty {
        // Interactive mode
        interactive_create()?
    } else {
        // Stdin mode
        is_stdin_input = true;
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            bail!("No input on stdin.");
        }
        buf
    };

    let mut node: Node = if is_stdin_input {
        parse_stdin_create(&input)?
    } else {
        serde_yaml::from_str(&input)?
    };

    for dl in args.data_lake {
        if !node.data_lake.contains(&dl) {
            node.data_lake.push(dl);
        }
    }
    for edge_str in &args.add_edge {
        node.edges.push(parse_edge_flag(edge_str)?);
    }
    for sf in args.add_source_file {
        if !node.source_files.contains(&sf) {
            node.source_files.push(sf);
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

pub fn update(path: &Path, args: UpdateArgs) -> Result<()> {
    let engram_dir = storage::find_engram_dir(path)?;
    let existing = storage::load_node(&engram_dir, &args.id)?;
    let is_tty = io::stdin().is_terminal();

    let has_flags = args.content.is_some()
        || args.weight.is_some()
        || !args.add_data_lake.is_empty()
        || !args.remove_data_lake.is_empty()
        || !args.add_edge.is_empty()
        || !args.remove_edge.is_empty()
        || !args.add_source_file.is_empty()
        || !args.remove_source_file.is_empty();

    let mut node = if args.edit {
        let existing_yaml = serde_yaml::to_string(&existing)?;
        let input = edit_in_editor(&existing_yaml)?;
        serde_yaml::from_str(&input)?
    } else if has_flags {
        let mut node = existing.clone();
        if let Some(c) = args.content {
            node.content = c;
        }
        if let Some(w) = args.weight {
            node.weight = w;
        }
        for dl in &args.add_data_lake {
            if !node.data_lake.contains(dl) {
                node.data_lake.push(dl.clone());
            }
        }
        node.data_lake
            .retain(|dl| !args.remove_data_lake.contains(dl));
        for edge_str in &args.add_edge {
            node.edges.push(parse_edge_flag(edge_str)?);
        }
        node.edges.retain(|e| !args.remove_edge.contains(&e.to));
        for sf in &args.add_source_file {
            if !node.source_files.contains(sf) {
                node.source_files.push(sf.clone());
            }
        }
        node.source_files
            .retain(|sf| !args.remove_source_file.contains(sf));
        node
    } else if is_tty {
        interactive_update(&existing)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            bail!("No input. Use --content, --weight, --edit, or pipe YAML to stdin.");
        }
        parse_stdin_update(&buf, &existing)?
    };

    node.touched = Utc::now();

    indexing::remove_backlinks_from_source(&engram_dir, &args.id, &existing)?;
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

    let indented = indent_content(&content);
    let mut yaml = NODE_TEMPLATE
        .replace("{id}", &id)
        .replace("{content}", &indented)
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

/// Parse edge flag format: "namespace:id:type:weight" e.g. "auth:session:uses:50"
/// The last two colon-separated segments are type and weight; everything before is the target node id.
fn parse_edge_flag(s: &str) -> Result<crate::models::node::Edge> {
    // Split from the right: weight, type, then everything else is the target id
    let parts: Vec<&str> = s.rsplitn(3, ':').collect();
    if parts.len() < 3 {
        bail!(
            "Invalid edge format: '{}'. Expected 'target:type:weight' e.g. 'auth:session:uses:50'",
            s
        );
    }
    let weight: u8 = parts[0].parse().map_err(|_| {
        anyhow::anyhow!(
            "Invalid edge weight '{}' in '{}'. Must be 0-100.",
            parts[0],
            s
        )
    })?;
    let edge_type = match parts[1] {
        "uses" => crate::models::node::EdgeType::Uses,
        "depends_on" => crate::models::node::EdgeType::DependsOn,
        "implements" => crate::models::node::EdgeType::Implements,
        "rationale" => crate::models::node::EdgeType::Rationale,
        "related" => crate::models::node::EdgeType::Related,
        other => bail!(
            "Unknown edge type '{}' in '{}'. Valid: uses, depends_on, implements, rationale, related",
            other,
            s
        ),
    };
    let target = parts[2].to_string();
    Ok(crate::models::node::Edge {
        to: target,
        edge_type,
        weight,
    })
}

/// Merge partial YAML from stdin into a base Value (defaults or existing node).
/// Only keys present in `partial` overwrite keys in `base`.
fn merge_yaml(base: &Value, partial: &Value) -> Value {
    match (base, partial) {
        (Value::Mapping(base_map), Value::Mapping(partial_map)) => {
            let mut merged = base_map.clone();
            for (key, val) in partial_map {
                merged.insert(key.clone(), val.clone());
            }
            Value::Mapping(merged)
        }
        _ => partial.clone(),
    }
}

/// Build a default Node as serde_yaml::Value for stdin create merge.
fn default_node_value() -> Value {
    let now = Utc::now().to_rfc3339();
    let default_node = format!(
        "id: placeholder\ncontent: ''\nweight: 50\nstatus: active\ncreated: {now}\ntouched: {now}"
    );
    serde_yaml::from_str(&default_node).expect("default node YAML is valid")
}

/// Parse stdin YAML into a Node, merging with defaults (create) or existing node (update).
fn parse_stdin_create(buf: &str) -> Result<Node> {
    let partial: Value = serde_yaml::from_str(buf)
        .context("Invalid YAML on stdin. Check indentation and field names.")?;
    let base = default_node_value();
    let merged = merge_yaml(&base, &partial);
    serde_yaml::from_value(merged).context("Merged YAML is not a valid node. Required field: 'id'.")
}

fn parse_stdin_update(buf: &str, existing: &Node) -> Result<Node> {
    let partial: Value = serde_yaml::from_str(buf)
        .context("Invalid YAML on stdin. Check indentation and field names.")?;
    let base: Value = serde_yaml::to_value(existing)?;
    let merged = merge_yaml(&base, &partial);
    serde_yaml::from_value(merged).context("Merged YAML is not a valid node. Check field types.")
}

/// Indent multiline content for YAML literal block scalar (content: |).
/// First line replaces {content} which is already indented 2 spaces in template.
/// Subsequent lines need 2-space indent to stay inside the block.
fn indent_content(content: &str) -> String {
    let mut lines = content.lines();
    let first = lines.next().unwrap_or("");
    let rest: Vec<String> = lines.map(|l| format!("  {}", l)).collect();
    if rest.is_empty() {
        first.to_string()
    } else {
        format!("{}\n{}", first, rest.join("\n"))
    }
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
