use std::path::Path;

pub fn run(cwd: &Path) -> anyhow::Result<()> {
    let engram_dir = crate::storage::find_engram_dir(cwd)?;
    println!("TUI launching from: {}", engram_dir.display());
    Ok(())
}
