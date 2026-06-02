//! Integration smoke tests for Phase 0.

use std::process::Command;

#[test]
fn binary_help_exits_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_graviton"))
        .arg("--help")
        .output()
        .expect("failed to run graviton --help");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("graviton"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("fetch"));
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("list-scenarios"));
    assert!(stdout.contains("bench"));
}

#[test]
fn list_scenarios_exits_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_graviton"))
        .arg("list-scenarios")
        .output()
        .expect("failed to run graviton list-scenarios");

    assert!(output.status.success());
}
