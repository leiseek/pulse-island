//! Process contract for the production Island host entry point.

use std::process::Command;

#[test]
fn diagnostic_mode_reports_compact_observe_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_pulse-island"))
        .arg("--diagnostic")
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("island_host_ready=true"));
    assert!(stdout.contains("ui_mode=compact_observe_only"));
    Ok(())
}

#[test]
fn snapshot_mode_is_content_free_when_state_is_missing() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!(
        "pulse-island-missing-{}.snapshot",
        std::process::id()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_pulse-island"))
        .args(["--snapshot", "--state"])
        .arg(&path)
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(
        stdout.trim(),
        "snapshot_health=empty snapshot_active=0 snapshot_recent_terminal=0"
    );
    Ok(())
}

#[test]
fn native_smoke_reports_target_capability_without_failure() -> Result<(), Box<dyn std::error::Error>>
{
    let output = Command::new(env!("CARGO_BIN_EXE_pulse-island"))
        .arg("--native-smoke")
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with("native_smoke="));
    Ok(())
}
