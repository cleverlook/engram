use std::path::Path;
use rusqlite::Connection;
use anyhow::Result;

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

        CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(id, content, tags);"
    )?;
    Ok(())
}
