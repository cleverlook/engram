use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use crate::models::node::Node;

pub fn open(engram_dir: &Path) -> Result<Connection> {
    let db_path = engram_dir.join("engram.db");
    let conn = Connection::open(db_path)?;
    Ok(conn)
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            namespace TEXT,
            weight INTEGER,
            status TEXT,
            source_hash TEXT,
            touched TEXT
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(id, content, tags);",
    )?;
    Ok(())
}

/// Extract namespace from node id: "auth:oauth:google" -> "auth:oauth"
fn namespace_of(id: &str) -> &str {
    id.rsplit_once(':').map(|(ns, _)| ns).unwrap_or("")
}

pub fn upsert_node(engram_dir: &Path, node: &Node) -> Result<()> {
    let conn = open(engram_dir)?;

    let status_str = serde_yaml::to_string(&node.status)?;
    let namespace = namespace_of(&node.id);

    conn.execute(
        "INSERT INTO nodes (id, namespace, weight, status, source_hash, touched)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            namespace = excluded.namespace,
            weight = excluded.weight,
            status = excluded.status,
            source_hash = excluded.source_hash,
            touched = excluded.touched",
        rusqlite::params![
            node.id,
            namespace,
            node.weight,
            status_str.trim(),
            node.source_hash,
            node.touched.to_string(),
        ],
    )?;

    // FTS: delete old entry, insert new
    conn.execute("DELETE FROM nodes_fts WHERE id = ?1", [&node.id])?;
    conn.execute(
        "INSERT INTO nodes_fts (id, content, tags) VALUES (?1, ?2, ?3)",
        rusqlite::params![node.id, node.content, ""],
    )?;

    Ok(())
}

pub fn delete_node(engram_dir: &Path, id: &str) -> Result<()> {
    let conn = open(engram_dir)?;
    conn.execute("DELETE FROM nodes WHERE id = ?1", [id])?;
    conn.execute("DELETE FROM nodes_fts WHERE id = ?1", [id])?;
    Ok(())
}

pub fn search(engram_dir: &Path, query: &str) -> Result<Vec<String>> {
    let conn = open(engram_dir)?;
    let mut stmt =
        conn.prepare("SELECT id FROM nodes_fts WHERE nodes_fts MATCH ?1 ORDER BY rank")?;
    let ids: Vec<String> = stmt
        .query_map([query], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}

/// Rebuild all SQLite data from YAML files.
pub fn rebuild(engram_dir: &Path) -> Result<()> {
    let conn = open(engram_dir)?;
    create_tables(&conn)?;
    conn.execute("DELETE FROM nodes", [])?;
    conn.execute("DELETE FROM nodes_fts", [])?;

    let nodes_dir = engram_dir.join("nodes");
    rebuild_from_dir(&conn, &nodes_dir)?;

    Ok(())
}

fn rebuild_from_dir(conn: &Connection, dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            rebuild_from_dir(conn, &path)?;
        } else if path.extension().is_some_and(|e| e == "yaml") {
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename.starts_with('_') {
                continue; // skip _index.yaml, _backlinks.yaml
            }
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(node) = serde_yaml::from_str::<Node>(&content) {
                    let status_str = serde_yaml::to_string(&node.status)?;
                    let namespace = namespace_of(&node.id);

                    conn.execute(
                        "INSERT OR REPLACE INTO nodes (id, namespace, weight, status, source_hash, touched)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![
                            node.id,
                            namespace,
                            node.weight,
                            status_str.trim(),
                            node.source_hash,
                            node.touched.to_string(),
                        ],
                    )?;
                    conn.execute(
                        "INSERT INTO nodes_fts (id, content, tags) VALUES (?1, ?2, ?3)",
                        rusqlite::params![node.id, node.content, ""],
                    )?;
                }
        }
    }

    Ok(())
}
