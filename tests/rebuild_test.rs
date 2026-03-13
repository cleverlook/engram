mod helpers;

use tempfile::TempDir;

#[test]
fn test_rebuild_index_restores_indexes() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let node = r#"id: test:rebuild
content: Test rebuild.
weight: 50
status: active
created: 2024-03-10
touched: 2024-03-10
edges: []
"#;
    helpers::create_node(dir.path(), node);

    // Delete index files
    std::fs::remove_file(dir.path().join(".engram/nodes/test/_index.yaml")).unwrap();

    // Rebuild
    let output = helpers::run_engram(dir.path(), &["rebuild-index"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rebuilt"));

    // Verify index restored
    let index = std::fs::read_to_string(dir.path().join(".engram/nodes/test/_index.yaml")).unwrap();
    assert!(index.contains("test:rebuild"));
}

#[test]
fn test_rebuild_restores_search() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let node = r#"id: search:rebuild_test
content: Unique searchable content for rebuild.
weight: 50
status: active
created: 2024-03-10
touched: 2024-03-10
edges: []
"#;
    helpers::create_node(dir.path(), node);

    // Delete SQLite
    std::fs::remove_file(dir.path().join(".engram/engram.db")).unwrap();

    // Rebuild
    let output = helpers::run_engram(dir.path(), &["rebuild-index"]);
    assert!(output.status.success());

    // Search should work again
    let output = helpers::run_engram(dir.path(), &["search", "searchable"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("search:rebuild_test"));
}
