//! Process-level contract for the short-lived shim executable.

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn diagnostic_mode_reports_bounded_preflight_without_echoing_input(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pulse-link-shim"))
        .arg("--diagnostic")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("shim stdin should be piped"))?;
    stdin.write_all(br#"{"version":1,"event":"synthetic"}"#)?;
    drop(stdin);

    let output = child.wait_with_output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("shim_exit=success"));
    assert!(stdout.contains("forwarded=false"));
    assert!(stdout.contains("rejection=none"));
    assert!(!stdout.contains("synthetic"));
    Ok(())
}

#[test]
fn process_path_accepts_documented_codex_hook_event_names_without_echoing_input(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pulse-link-shim"))
        .args([
            "--diagnostic",
            "--scope",
            "process-contract",
            "--provider",
            "codex_cli",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("shim stdin should be piped"))?;
    stdin.write_all(br#"{"session_id":"process-session","hook_event_name":"UserPromptSubmit"}"#)?;
    drop(stdin);
    let output = child.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());
    assert!(stdout.contains("shim_exit=success"));
    assert!(stdout.contains("rejection=none"));
    assert!(!stdout.contains("process-session"));
    Ok(())
}
