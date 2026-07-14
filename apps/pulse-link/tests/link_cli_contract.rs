//! Process-level contract for the Link executable diagnostics.

use std::io::Write;
use std::process::{Command, Stdio};

use pulse_link_core::{LinkFrameHeader, LinkMessageKind};
use pulse_persistence::{BreadcrumbStore, FileBreadcrumbStore};

#[test]
fn diagnostic_c1_reports_runtime_state_without_synthetic_payload(
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_pulse-link"))
        .arg("--diagnostic-c1")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("link_lifecycle=island_active"));
    assert!(stdout.contains("active_breadcrumbs=1"));
    assert!(stdout.contains("island_attached=true"));
    assert!(stdout.contains("provider_affected=false"));
    assert!(!stdout.contains("synthetic"));
    Ok(())
}

#[test]
fn handoff_stdin_reduces_a_fixed_header_without_echoing_it(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pulse-link"))
        .arg("--handoff-stdin")
        .arg("--diagnostic")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 19,
        payload_length: 0,
    }
    .encode();
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("Link stdin should be piped"))?;
    stdin.write_all(&frame)?;
    drop(stdin);

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("ingress_accepted=true active_breadcrumbs=1"));
    assert!(!stdout.contains("19"));
    Ok(())
}

#[test]
fn handoff_stdin_checkpoints_a_file_backed_breadcrumb() -> Result<(), Box<dyn std::error::Error>> {
    let directory = unique_test_directory("handoff-breadcrumb")?;
    let breadcrumb = directory.join("breadcrumb.snapshot");
    let mut child = Command::new(env!("CARGO_BIN_EXE_pulse-link"))
        .args([
            "--handoff-stdin",
            "--diagnostic",
            "--breadcrumb",
            breadcrumb
                .to_str()
                .ok_or_else(|| std::io::Error::other("breadcrumb path should be UTF-8"))?,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 23,
        payload_length: 0,
    }
    .encode();
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("Link stdin should be piped"))?;
    stdin.write_all(&frame)?;
    drop(stdin);

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("active_breadcrumbs=1"));
    assert_eq!(
        FileBreadcrumbStore::new(breadcrumb)
            .load()?
            .active_tasks
            .len(),
        1
    );

    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn persistent_handoff_uses_the_configured_pulse_state_root(
) -> Result<(), Box<dyn std::error::Error>> {
    let state_root = unique_test_directory("persistent-handoff")?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_pulse-link"))
        .args(["--handoff-stdin", "--persistent", "--diagnostic"])
        .env("PULSE_LINK_STATE_ROOT", &state_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 29,
        payload_length: 0,
    }
    .encode();
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("Link stdin should be piped"))?;
    stdin.write_all(&frame)?;
    drop(stdin);

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("active_breadcrumbs=1"));
    assert_eq!(
        FileBreadcrumbStore::new(state_root.join("breadcrumbs.snapshot"))
            .load()?
            .active_tasks
            .len(),
        1
    );

    std::fs::remove_dir_all(state_root)?;
    Ok(())
}

#[test]
fn snapshot_command_recovers_a_file_backed_late_attach_state(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = unique_test_directory("late-attach")?;
    let breadcrumb = directory.join("breadcrumb.snapshot");
    let frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 31,
        payload_length: 0,
    }
    .encode();
    let mut handoff = Command::new(env!("CARGO_BIN_EXE_pulse-link"))
        .args([
            "--handoff-stdin",
            "--breadcrumb",
            breadcrumb
                .to_str()
                .ok_or_else(|| std::io::Error::other("breadcrumb path should be UTF-8"))?,
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    handoff
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("Link stdin should be piped"))?
        .write_all(&frame)?;
    assert!(handoff.wait()?.success());

    let output = Command::new(env!("CARGO_BIN_EXE_pulse-link"))
        .args([
            "--snapshot",
            "--breadcrumb",
            breadcrumb
                .to_str()
                .ok_or_else(|| std::io::Error::other("breadcrumb path should be UTF-8"))?,
        ])
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("snapshot_active=1"),
        "snapshot output: {stdout}"
    );
    assert!(stdout.contains("snapshot_recent_terminal=0"));
    assert!(!stdout.contains("31"));

    std::fs::remove_dir_all(directory)?;
    Ok(())
}

fn unique_test_directory(name: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!(
        "pulse-link-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}
