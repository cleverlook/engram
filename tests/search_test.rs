mod helpers;

use tempfile::TempDir;

const NODE_A: &str = r#"id: auth:oauth:google
content: Google OAuth uses Authorization Code Flow with PKCE.
weight: 65
status: active
created: 2024-03-10
touched: 2024-03-10
edges: []
"#;

const NODE_B: &str = r#"id: redis:session_store
content: Redis stores session tokens with 24h TTL.
weight: 50
status: active
created: 2024-03-10
touched: 2024-03-10
edges: []
"#;

#[test]
fn test_search_finds_matching_node() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), NODE_A);
    helpers::create_node(dir.path(), NODE_B);

    let output = helpers::run_engram(dir.path(), &["search", "OAuth"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("auth:oauth:google"));
    assert!(!stdout.contains("redis:session_store"));
}

#[test]
fn test_search_no_results() {
    let dir = TempDir::new().unwrap();
    helpers::setup_engram(dir.path());
    helpers::create_node(dir.path(), NODE_A);

    let output = helpers::run_engram(dir.path(), &["search", "nonexistent"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No results"));
}
