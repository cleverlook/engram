mod helpers;

use tempfile::TempDir;

#[test]
fn test_status_reports_stale_nodes() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let old_node = r#"id: old:stale_node
content: Very old node.
weight: 50
status: active
created: 2023-01-01
touched: 2023-01-01
edges: []
"#;
    helpers::create_node(dir.path(), old_node);

    let output = helpers::run_engram(dir.path(), &["status"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("stale"));
    assert!(stdout.contains("old:stale_node"));
}

#[test]
fn test_status_applies_weight_decay() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let old_node = r#"id: decay:test
content: Node for decay test.
weight: 50
status: active
created: 2023-01-01
touched: 2023-01-01
edges: []
"#;
    helpers::create_node(dir.path(), old_node);

    let output = helpers::run_engram(dir.path(), &["status"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("decayed"));

    // Verify weight actually decreased
    let output = helpers::run_engram(dir.path(), &["node", "get", "decay:test"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("w:45")); // 50 - 5 (>90 days)
}

#[test]
fn test_check_finds_broken_edges() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let broken = r#"id: test:broken
content: Node with broken edge.
weight: 50
status: active
created: 2024-03-10
touched: 2024-03-10
edges:
  - to: does:not:exist
    type: uses
    weight: 80
"#;
    helpers::create_node(dir.path(), broken);

    let output = helpers::run_engram(dir.path(), &["check"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("broken edge"));
    assert!(stdout.contains("does:not:exist"));
}

#[test]
fn test_check_finds_broken_data_lake_refs() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let broken_dl = r#"id: test:broken_dl
content: Node with broken data lake ref.
weight: 50
status: active
created: 2024-03-10
touched: 2024-03-10
data_lake:
  - nonexistent.png
edges: []
"#;
    helpers::create_node(dir.path(), broken_dl);

    let output = helpers::run_engram(dir.path(), &["check"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("broken data_lake"));
}

#[test]
fn test_check_passes_clean_graph() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let clean = r#"id: clean:node
content: A clean node.
weight: 50
status: active
created: 2026-03-10
touched: 2026-03-10
edges: []
"#;
    helpers::create_node(dir.path(), clean);

    let output = helpers::run_engram(dir.path(), &["check"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("check passed"));
}
