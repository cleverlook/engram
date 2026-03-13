use std::path::PathBuf;
use std::process::Command;

pub fn engram_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("engram");
    path
}

pub fn setup_engram(dir: &std::path::Path) {
    let output = Command::new(engram_bin())
        .arg("init")
        .current_dir(dir)
        .output()
        .expect("failed to run engram init");
    assert!(
        output.status.success(),
        "engram init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn create_node(dir: &std::path::Path, yaml: &str) {
    let output = Command::new(engram_bin())
        .args(["node", "create"])
        .current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(yaml.as_bytes())?;
            child.wait_with_output()
        })
        .expect("failed to run engram node create");
    assert!(
        output.status.success(),
        "node create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn run_engram(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(engram_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run engram")
}
