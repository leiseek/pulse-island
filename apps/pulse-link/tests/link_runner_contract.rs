//! W3 pure Link runner contract tests.

use pulse_domain::{BoundedText, Lifecycle, PrivacyProfile, TimestampMs};
use pulse_link::{
    apply_header_only_ingress, DropModeGraceDriver, LinkRuntime, LinkRuntimeReport,
    SPIKE_C_GRACE_PERIOD_MS,
};
use pulse_link_core::{LinkFrameHeader, LinkLifecycleState, LinkMessageKind};
use pulse_persistence::{BreadcrumbStore, FileBreadcrumbStore};
use pulse_protocol::{AdmittedEvent, EvidenceKind};
use std::path::PathBuf;

#[test]
fn synthetic_start_event_enters_active_and_checkpoints_active_breadcrumb(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LinkRuntime::new();

    let report = runtime.apply_event(
        event("task-a", EvidenceKind::Started)?,
        PrivacyProfile::Minimal,
    )?;

    assert_eq!(
        report,
        LinkRuntimeReport {
            lifecycle_state: LinkLifecycleState::Active,
            active_tasks: 1,
            recent_terminal_tasks: 0,
            checkpoint_written: true,
        }
    );
    assert_eq!(runtime.load_breadcrumbs()?.active_tasks.len(), 1);
    Ok(())
}

#[test]
fn header_only_ingress_reduces_to_a_bounded_active_breadcrumb(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LinkRuntime::new();
    let report = apply_header_only_ingress(
        &mut runtime,
        LinkFrameHeader {
            message_kind: LinkMessageKind::HookEnvelope,
            request_id: 7,
            payload_length: 0,
        },
    )?;

    assert!(report.checkpoint_written);
    assert_eq!(report.active_tasks, 1);
    Ok(())
}

#[test]
fn minimal_terminal_event_enters_grace_and_moves_breadcrumb_to_recent_terminal(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LinkRuntime::new();

    runtime.apply_event(
        event("task-a", EvidenceKind::Started)?,
        PrivacyProfile::Minimal,
    )?;
    let report = runtime.apply_event(
        event("task-a", EvidenceKind::Completed)?,
        PrivacyProfile::Minimal,
    )?;
    let breadcrumbs = runtime.load_breadcrumbs()?;

    assert_eq!(report.lifecycle_state, LinkLifecycleState::GracePeriod);
    assert_eq!(breadcrumbs.active_tasks.len(), 0);
    assert_eq!(breadcrumbs.recent_terminal_tasks.len(), 1);
    assert_eq!(
        breadcrumbs.recent_terminal_tasks[0].lifecycle,
        Lifecycle::Completed
    );
    Ok(())
}

#[test]
fn strict_terminal_event_checkpoints_without_retaining_terminal_breadcrumb(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LinkRuntime::new();

    runtime.apply_event(
        event("task-a", EvidenceKind::Started)?,
        PrivacyProfile::Strict,
    )?;
    let report = runtime.apply_event(
        event("task-a", EvidenceKind::Completed)?,
        PrivacyProfile::Strict,
    )?;
    let breadcrumbs = runtime.load_breadcrumbs()?;

    assert_eq!(report.lifecycle_state, LinkLifecycleState::GracePeriod);
    assert!(breadcrumbs.active_tasks.is_empty());
    assert!(breadcrumbs.recent_terminal_tasks.is_empty());
    Ok(())
}

#[test]
fn file_backed_runtime_checkpoints_and_recovers_degraded_breadcrumb(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("runtime-file-store")?;
    let path = dir.join("breadcrumbs.snapshot");
    let mut runtime = LinkRuntime::with_store(FileBreadcrumbStore::new(path.clone()));

    runtime.apply_event(
        event("task-a", EvidenceKind::Started)?,
        PrivacyProfile::Minimal,
    )?;
    let persisted = FileBreadcrumbStore::new(path.clone()).load()?;
    assert_eq!(persisted.active_tasks.len(), 1);

    let mut recovered = LinkRuntime::with_store(FileBreadcrumbStore::new(path));
    recovered.recover_degraded_from_breadcrumbs(persisted)?;
    let recovered_breadcrumbs = recovered.load_breadcrumbs()?;

    assert_eq!(recovered.lifecycle_state(), LinkLifecycleState::Active);
    assert_eq!(recovered_breadcrumbs.active_tasks.len(), 1);
    assert_eq!(
        recovered_breadcrumbs.active_tasks[0].health,
        pulse_domain::TaskHealth::Degraded
    );

    remove_dir_if_exists(dir)?;
    Ok(())
}

#[test]
fn drop_mode_grace_driver_waits_ninety_seconds_then_final_checkpoints_and_stops(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LinkRuntime::new();
    let mut grace = DropModeGraceDriver::spike_c();

    runtime.apply_event(
        event_at("task-a", EvidenceKind::Started, TimestampMs(1))?,
        PrivacyProfile::Minimal,
    )?;
    runtime.apply_event(
        event_at("task-a", EvidenceKind::Completed, TimestampMs(10))?,
        PrivacyProfile::Minimal,
    )?;

    let armed = grace.observe_runtime(&runtime, TimestampMs(10));
    assert_eq!(
        armed.grace_deadline,
        Some(TimestampMs(10 + SPIKE_C_GRACE_PERIOD_MS))
    );
    assert_eq!(runtime.lifecycle_state(), LinkLifecycleState::GracePeriod);

    let before_deadline =
        grace.tick(&mut runtime, TimestampMs(10 + SPIKE_C_GRACE_PERIOD_MS - 1))?;
    assert!(!before_deadline.stopped_link);
    assert_eq!(runtime.lifecycle_state(), LinkLifecycleState::GracePeriod);

    let after_deadline = grace.tick(&mut runtime, TimestampMs(10 + SPIKE_C_GRACE_PERIOD_MS))?;
    assert!(after_deadline.final_checkpoint_written);
    assert!(after_deadline.stopped_link);
    assert_eq!(runtime.lifecycle_state(), LinkLifecycleState::NotRunning);
    assert_eq!(
        runtime.load_breadcrumbs()?.written_at,
        TimestampMs(10 + SPIKE_C_GRACE_PERIOD_MS)
    );
    Ok(())
}

#[test]
fn drop_mode_grace_driver_cancels_pending_exit_when_new_relevant_event_arrives(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LinkRuntime::new();
    let mut grace = DropModeGraceDriver::spike_c();

    runtime.apply_event(
        event_at("task-a", EvidenceKind::Started, TimestampMs(1))?,
        PrivacyProfile::Minimal,
    )?;
    runtime.apply_event(
        event_at("task-a", EvidenceKind::Completed, TimestampMs(10))?,
        PrivacyProfile::Minimal,
    )?;
    let _ = grace.observe_runtime(&runtime, TimestampMs(10));

    runtime.apply_event(
        event_at("task-b", EvidenceKind::Started, TimestampMs(20))?,
        PrivacyProfile::Minimal,
    )?;
    let cancelled = grace.observe_runtime(&runtime, TimestampMs(20));
    assert_eq!(cancelled.grace_deadline, None);

    let report = grace.tick(&mut runtime, TimestampMs(10 + SPIKE_C_GRACE_PERIOD_MS))?;
    assert!(!report.stopped_link);
    assert_eq!(runtime.lifecycle_state(), LinkLifecycleState::Active);
    assert_eq!(runtime.load_breadcrumbs()?.active_tasks.len(), 1);
    Ok(())
}

fn event(task: &str, evidence: EvidenceKind) -> Result<AdmittedEvent, Box<dyn std::error::Error>> {
    event_at(task, evidence, TimestampMs(1))
}

fn event_at(
    task: &str,
    evidence: EvidenceKind,
    occurred_at: TimestampMs,
) -> Result<AdmittedEvent, Box<dyn std::error::Error>> {
    Ok(AdmittedEvent {
        provider: BoundedText::new("synthetic")?,
        task: BoundedText::new(task)?,
        evidence,
        occurred_at,
    })
}

fn unique_test_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "pulse-link-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn remove_dir_if_exists(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
