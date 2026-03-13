mod helpers;

use tempfile::TempDir;

const NODE_ROOT: &str = r#"id: auth:oauth:google
content: Google OAuth uses PKCE.
weight: 65
status: active
created: 2024-03-10T00:00:00Z
touched: 2024-03-10T00:00:00Z
edges:
  - to: redis:session_store
    type: uses
    weight: 80
  - to: auth:oauth:flow
    type: implements
    weight: 90
"#;

const NODE_REDIS: &str = r#"id: redis:session_store
content: Redis stores session tokens.
weight: 50
status: active
created: 2024-03-10T00:00:00Z
touched: 2024-03-10T00:00:00Z
edges: []
"#;

const NODE_FLOW: &str = r#"id: auth:oauth:flow
content: Standard OAuth2 flow.
weight: 80
status: active
created: 2024-03-10T00:00:00Z
touched: 2024-03-10T00:00:00Z
edges: []
"#;

const NODE_DEPRECATED: &str = r#"id: old:deprecated_thing
content: This is deprecated.
weight: 10
status: deprecated
created: 2024-01-01T00:00:00Z
touched: 2024-01-01T00:00:00Z
edges: []
"#;

#[test]
fn test_traverse_visits_connected_nodes() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), NODE_ROOT);
    helpers::create_node(dir.path(), NODE_REDIS);
    helpers::create_node(dir.path(), NODE_FLOW);

    let output = helpers::run_engram(dir.path(), &["traverse", "auth:oauth:google"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("auth:oauth:google"));
    assert!(stdout.contains("redis:session_store"));
    assert!(stdout.contains("auth:oauth:flow"));
}

#[test]
fn test_traverse_respects_depth() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), NODE_ROOT);
    helpers::create_node(dir.path(), NODE_REDIS);
    helpers::create_node(dir.path(), NODE_FLOW);

    let output = helpers::run_engram(
        dir.path(),
        &["traverse", "auth:oauth:google", "--depth", "0"],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("auth:oauth:google"));
    assert!(!stdout.contains("redis:session_store"));
}

#[test]
fn test_traverse_skips_deprecated() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());

    let root_with_dep_edge = r#"id: test:root
content: Root node.
weight: 50
status: active
created: 2024-03-10T00:00:00Z
touched: 2024-03-10T00:00:00Z
edges:
  - to: old:deprecated_thing
    type: related
    weight: 50
"#;

    helpers::create_node(dir.path(), root_with_dep_edge);
    helpers::create_node(dir.path(), NODE_DEPRECATED);

    let output = helpers::run_engram(dir.path(), &["traverse", "test:root"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("test:root"));
    assert!(!stdout.contains("deprecated_thing"));
}

#[test]
fn test_traverse_respects_min_weight() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), NODE_ROOT);
    helpers::create_node(dir.path(), NODE_REDIS);
    helpers::create_node(dir.path(), NODE_FLOW);

    // min-weight 85 should only follow edge with weight 90 (flow), skip 80 (redis)
    let output = helpers::run_engram(
        dir.path(),
        &["traverse", "auth:oauth:google", "--min-weight", "85"],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("auth:oauth:flow"));
    assert!(!stdout.contains("redis:session_store"));
}
