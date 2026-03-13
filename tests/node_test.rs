mod helpers;

use tempfile::TempDir;

const TEST_NODE: &str = r#"id: auth:oauth:google
content: Google OAuth uses PKCE.
weight: 65
status: active
created: 2024-03-10
touched: 2024-03-10
edges:
  - to: redis:session_store
    type: uses
    weight: 80
"#;

const TEST_NODE_REDIS: &str = r#"id: redis:session_store
content: Redis stores session tokens.
weight: 50
status: active
created: 2024-03-10
touched: 2024-03-10
edges: []
"#;

#[test]
fn test_node_create_and_get() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), TEST_NODE);

    // Verify YAML file exists
    assert!(dir.path().join(".engram/nodes/auth/oauth/google.yaml").is_file());

    // Verify get works
    let output = helpers::run_engram(dir.path(), &["node", "get", "auth:oauth:google"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("auth:oauth:google"));
    assert!(stdout.contains("Google OAuth uses PKCE"));
}

#[test]
fn test_node_create_duplicate_fails() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), TEST_NODE);

    // Second create should fail
    let output = std::process::Command::new(helpers::engram_bin())
        .args(["node", "create"])
        .current_dir(dir.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(TEST_NODE.as_bytes()).unwrap();
            child.wait_with_output()
        })
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_node_deprecate() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), TEST_NODE);

    let output = helpers::run_engram(dir.path(), &["node", "deprecate", "auth:oauth:google"]);
    assert!(output.status.success());

    let output = helpers::run_engram(dir.path(), &["node", "get", "auth:oauth:google"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("deprecated"));
}

#[test]
fn test_node_get_nonexistent_fails() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let output = helpers::run_engram(dir.path(), &["node", "get", "does:not:exist"]);
    assert!(!output.status.success());
}

#[test]
fn test_index_updated_on_create() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), TEST_NODE);

    // Check namespace index
    let index = std::fs::read_to_string(dir.path().join(".engram/nodes/auth/oauth/_index.yaml")).unwrap();
    assert!(index.contains("auth:oauth:google"));

    // Check top-level index
    let top_index = std::fs::read_to_string(dir.path().join(".engram/nodes/_index.yaml")).unwrap();
    assert!(top_index.contains("auth"));
}

#[test]
fn test_backlinks_created_on_node_create() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), TEST_NODE);

    // Check backlinks for redis:session_store
    let bl_path = dir.path().join(".engram/nodes/redis/_backlinks.yaml");
    assert!(bl_path.is_file());
    let content = std::fs::read_to_string(&bl_path).unwrap();
    assert!(content.contains("auth:oauth:google"));
    assert!(content.contains("uses"));
}
