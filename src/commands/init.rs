use std::fs;
use std::path::Path;
use anyhow::{bail, Result};

use crate::db;

const SKILL_MD: &str = include_str!("../../templates/SKILL.md");

const TOP_LEVEL_INDEX: &str = "namespaces: []\n";

const GITIGNORE: &str = "engram.db\n";

pub fn run(path: &Path) -> Result<()> {
    let engram_dir = path.join(".engram");

    if engram_dir.exists() {
        bail!(".engram/ already exists in {}", path.display());
    }

    // Create directory structure
    let nodes_dir = engram_dir.join("nodes");
    let data_lake_dir = engram_dir.join("data_lake");
    fs::create_dir_all(&nodes_dir)?;
    fs::create_dir_all(&data_lake_dir)?;

    // Create .gitignore
    fs::write(engram_dir.join(".gitignore"), GITIGNORE)?;

    // Create top-level _index.yaml
    fs::write(nodes_dir.join("_index.yaml"), TOP_LEVEL_INDEX)?;

    // Create SQLite DB with tables
    let conn = db::open(&engram_dir)?;
    db::create_tables(&conn)?;

    // Create SKILL.md
    fs::write(path.join("SKILL.md"), SKILL_MD)?;
    println!("Created SKILL.md");

    println!("Initialized .engram/ in {}", path.display());
    Ok(())
}
