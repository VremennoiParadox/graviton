//! Phase 1 exit criterion: headless Earth–Moon simulation.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn earth_moon_headless_produces_diagnostics() {
    let scenario = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scenarios")
        .join("earth-moon.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_orrery-tui"))
        .args([
            "run",
            scenario.to_str().unwrap(),
            "--headless",
            "--steps",
            "100",
        ])
        .output()
        .expect("failed to run orrery-tui");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Earth-Moon"));
    assert!(stdout.contains("Energy (J)"));
    assert!(stdout.contains("drift"));
}
