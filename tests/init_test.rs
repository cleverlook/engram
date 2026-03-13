mod helpers;

use tempfile::TempDir;

#[test]
fn test_init_creates_structure() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    assert!(dir.path().join(".engram").is_dir());
    assert!(dir.path().join(".engram/nodes").is_dir());
    assert!(dir.path().join(".engram/data_lake").is_dir());
    assert!(dir.path().join(".engram/.gitignore").is_file());
    assert!(dir.path().join(".engram/nodes/_index.yaml").is_file());
    assert!(dir.path().join(".engram/engram.db").is_file());
    assert!(dir.path().join("SKILL.md").is_file());
}

#[test]
fn test_init_fails_if_already_exists() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let output = helpers::run_engram(dir.path(), &["init"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn test_gitignore_contains_engram_db() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let content = std::fs::read_to_string(dir.path().join(".engram/.gitignore")).unwrap();
    assert!(content.contains("engram.db"));
}
