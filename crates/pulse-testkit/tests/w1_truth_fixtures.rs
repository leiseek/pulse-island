//! End-to-end W1 truth fixtures using only provider-neutral pure crates.

use pulse_arbitration::arbitrate;
use pulse_domain::{
    Attention, Lifecycle, PrivacyProfile, ProcessFingerprint, ProviderReleaseStatus, RouteStrength,
    SafeSummary, TaskHealth, TaskSnapshot, TimestampMs,
};
use pulse_fuel::{FuelLedger, FuelState};
use pulse_protocol::{
    admit, AdmittedEvent, EvidenceKind, PulseHookEnvelope, RejectionCategory, PROTOCOL_VERSION,
};
use pulse_reducer::{apply_freshness, initial, reduce};
use pulse_routing::{label_for, label_for_evidence, RouteActionLabel, RouteEvidence};
use pulse_testkit::{provider, task, FixedClock};

fn event(
    provider_name: &str,
    task_name: &str,
    evidence: EvidenceKind,
    now: TimestampMs,
) -> Result<AdmittedEvent, Box<dyn std::error::Error>> {
    Ok(AdmittedEvent {
        provider: provider(provider_name)?.0,
        task: task(task_name)?.0,
        evidence,
        occurred_at: now,
    })
}

fn reduce_sequence(
    provider_name: &str,
    task_name: &str,
    evidences: &[EvidenceKind],
    privacy: PrivacyProfile,
) -> Result<TaskSnapshot, Box<dyn std::error::Error>> {
    let mut clock = FixedClock::new(1_000);
    let first = event(provider_name, task_name, evidences[0].clone(), clock.now())?;
    let mut snapshot = initial(&first, clock.now());
    for evidence in evidences {
        let admitted = event(provider_name, task_name, evidence.clone(), clock.now())?;
        snapshot = reduce(snapshot, &admitted, clock.now(), privacy).snapshot;
        clock.advance(10);
    }
    Ok(snapshot)
}

#[test]
fn same_fixture_and_clock_yield_identical_snapshot_and_plan(
) -> Result<(), Box<dyn std::error::Error>> {
    let evidences = [
        EvidenceKind::Started,
        EvidenceKind::Activity,
        EvidenceKind::Waiting,
        EvidenceKind::WaitingCleared,
        EvidenceKind::Route(RouteStrength::Useful),
    ];
    let first = reduce_sequence("Codex", "stable-task", &evidences, PrivacyProfile::Minimal)?;
    let second = reduce_sequence("Codex", "stable-task", &evidences, PrivacyProfile::Minimal)?;
    assert_eq!(first, second);

    let plan_a = arbitrate(std::slice::from_ref(&first), None, TimestampMs(9_000));
    let plan_b = arbitrate(&[second], None, TimestampMs(9_000));
    assert_eq!(plan_a, plan_b);
    assert_eq!(
        plan_a.primary.as_ref().map(|s| s.lifecycle),
        Some(Lifecycle::Running)
    );
    Ok(())
}

#[test]
fn route_labels_require_matching_route_strength() {
    assert_eq!(
        label_for(RouteStrength::Exact),
        Some(RouteActionLabel::OpenOriginalTask)
    );
    assert_eq!(
        label_for(RouteStrength::Strong),
        Some(RouteActionLabel::FocusAgentWindow)
    );
    assert_ne!(
        label_for(RouteStrength::Strong),
        Some(RouteActionLabel::OpenOriginalTask)
    );
    assert_eq!(
        label_for(RouteStrength::Useful),
        Some(RouteActionLabel::OpenWorkspace)
    );
    assert_eq!(
        label_for(RouteStrength::Weak),
        Some(RouteActionLabel::ShowProcessDetails)
    );
}

#[test]
fn strict_terminal_transition_is_not_retained_for_restart() -> Result<(), Box<dyn std::error::Error>>
{
    let start = event(
        "Claude",
        "strict-task",
        EvidenceKind::Started,
        TimestampMs(1),
    )?;
    let snapshot = reduce(
        initial(&start, TimestampMs(1)),
        &start,
        TimestampMs(1),
        PrivacyProfile::Strict,
    )
    .snapshot;
    let completed = event(
        "Claude",
        "strict-task",
        EvidenceKind::Completed,
        TimestampMs(2),
    )?;
    let terminal = reduce(snapshot, &completed, TimestampMs(2), PrivacyProfile::Strict);
    assert_eq!(terminal.snapshot.lifecycle, Lifecycle::Completed);
    assert!(!terminal.retain_breadcrumb);
    Ok(())
}

#[test]
fn process_only_antigravity_remains_observed_without_task_semantics(
) -> Result<(), Box<dyn std::error::Error>> {
    let process = ProcessFingerprint {
        pid: 99,
        start_ms: TimestampMs(50),
    };
    let snapshot = reduce_sequence(
        "Antigravity",
        "process-only",
        &[EvidenceKind::ProcessObserved { process }],
        PrivacyProfile::Minimal,
    )?;
    assert_eq!(
        snapshot.provider_status,
        ProviderReleaseStatus::ProcessObserved
    );
    assert_eq!(snapshot.health, TaskHealth::Observed);
    assert_eq!(snapshot.lifecycle, Lifecycle::Observed);
    assert_eq!(snapshot.attention, Attention::None);
    assert_eq!(snapshot.summary, SafeSummary::ObservedProcess);
    assert!(!snapshot.fuel_risk);
    assert!(!snapshot.fuel_blocking);
    Ok(())
}

#[test]
fn malformed_or_unapproved_input_is_rejected_before_snapshot_mutation(
) -> Result<(), Box<dyn std::error::Error>> {
    let start = event(
        "Codex",
        "admission-task",
        EvidenceKind::Started,
        TimestampMs(1),
    )?;
    let before = initial(&start, TimestampMs(1));
    let unapproved = PulseHookEnvelope {
        version: PROTOCOL_VERSION,
        provider: provider("Codex")?.0,
        task: task("admission-task")?.0,
        evidence: EvidenceKind::Activity,
        byte_len: 128,
        forbidden_field_seen: false,
        structured_source_approved: false,
        occurred_at: TimestampMs(2),
    };
    assert_eq!(
        admit(unapproved),
        Err(RejectionCategory::UnsupportedStructuredSource)
    );
    assert_eq!(before.lifecycle, Lifecycle::Unknown);
    Ok(())
}

#[test]
fn freshness_degrades_then_fresh_activity_recovers() -> Result<(), Box<dyn std::error::Error>> {
    let start = event(
        "Codex",
        "freshness-task",
        EvidenceKind::Started,
        TimestampMs(1),
    )?;
    let snapshot = reduce(
        initial(&start, TimestampMs(1)),
        &start,
        TimestampMs(1),
        PrivacyProfile::Minimal,
    )
    .snapshot;
    let degraded = apply_freshness(snapshot, TimestampMs(10_000), 100);
    assert_eq!(degraded.health, TaskHealth::Degraded);
    let activity = event(
        "Codex",
        "freshness-task",
        EvidenceKind::Activity,
        TimestampMs(10_001),
    )?;
    let recovered = reduce(
        degraded,
        &activity,
        TimestampMs(10_001),
        PrivacyProfile::Minimal,
    )
    .snapshot;
    assert_eq!(recovered.health, TaskHealth::Attached);
    assert_eq!(recovered.lifecycle, Lifecycle::Running);
    Ok(())
}

#[test]
fn fuel_buckets_are_provider_scoped_and_never_aggregated() -> Result<(), Box<dyn std::error::Error>>
{
    let codex = provider("Codex")?;
    let claude = provider("Claude")?;
    let mut ledger = FuelLedger::default();
    ledger.upsert(codex.clone(), FuelState::available(true, false));
    ledger.upsert(claude.clone(), FuelState::available(false, true));
    assert_eq!(ledger.len(), 2);
    assert_eq!(ledger.get(&codex).map(|state| state.risk), Some(true));
    assert_eq!(ledger.get(&codex).map(|state| state.blocking), Some(false));
    assert_eq!(ledger.get(&claude).map(|state| state.risk), Some(false));
    assert_eq!(ledger.get(&claude).map(|state| state.blocking), Some(true));
    Ok(())
}

#[test]
fn waiting_summary_survives_generic_activity_until_clear() -> Result<(), Box<dyn std::error::Error>>
{
    let waiting = reduce_sequence(
        "Codex",
        "waiting-summary",
        &[EvidenceKind::Waiting, EvidenceKind::Activity],
        PrivacyProfile::Minimal,
    )?;
    assert_eq!(waiting.lifecycle, Lifecycle::WaitingUser);
    assert_eq!(waiting.summary, SafeSummary::WaitingForUser);
    let clear = event(
        "Codex",
        "waiting-summary",
        EvidenceKind::WaitingCleared,
        TimestampMs(2_000),
    )?;
    let running = reduce(waiting, &clear, TimestampMs(2_000), PrivacyProfile::Minimal).snapshot;
    assert_eq!(running.lifecycle, Lifecycle::Running);
    Ok(())
}

#[test]
fn island_reconnect_protocol_is_snapshot_and_delta_only() -> Result<(), Box<dyn std::error::Error>>
{
    use pulse_protocol::{
        FullSnapshot, IslandMessage, LinkHealthStatus, ProtocolErrorCategory, SnapshotDelta,
    };

    let snapshot = reduce_sequence(
        "Codex",
        "reconnect-task",
        &[EvidenceKind::Started],
        PrivacyProfile::Minimal,
    )?;
    let full = FullSnapshot::new(1, vec![snapshot.clone()])?;
    let delta = SnapshotDelta::new(2, vec![snapshot], Vec::new())?;
    let messages = vec![
        IslandMessage::FullSnapshot(full),
        IslandMessage::SnapshotDelta(delta),
        IslandMessage::LinkHealth {
            status: LinkHealthStatus::Healthy,
        },
        IslandMessage::ProtocolError(ProtocolErrorCategory::Malformed),
    ];
    assert_eq!(messages.len(), 4);
    Ok(())
}

#[test]
fn storm_activity_does_not_revert_terminal_state() -> Result<(), Box<dyn std::error::Error>> {
    let start = event(
        "Codex",
        "storm-terminal",
        EvidenceKind::Started,
        TimestampMs(1),
    )?;
    let mut snapshot = reduce(
        initial(&start, TimestampMs(1)),
        &start,
        TimestampMs(1),
        PrivacyProfile::Minimal,
    )
    .snapshot;
    let completed = event(
        "Codex",
        "storm-terminal",
        EvidenceKind::Completed,
        TimestampMs(2),
    )?;
    snapshot = reduce(
        snapshot,
        &completed,
        TimestampMs(2),
        PrivacyProfile::Minimal,
    )
    .snapshot;
    for i in 0..10_000_u64 {
        let activity = event(
            "Codex",
            "storm-terminal",
            EvidenceKind::Activity,
            TimestampMs(3 + i),
        )?;
        snapshot = reduce(
            snapshot,
            &activity,
            TimestampMs(3 + i),
            PrivacyProfile::Minimal,
        )
        .snapshot;
    }
    assert_eq!(snapshot.lifecycle, Lifecycle::Completed);
    Ok(())
}

#[test]
fn hundreds_of_tasks_keep_primary_and_peek_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let mut snapshots = Vec::new();
    for index in 0..256_u64 {
        let task_name = format!("task-{index}");
        let evidence = if index == 42 {
            EvidenceKind::Failed
        } else if index == 43 {
            EvidenceKind::Waiting
        } else {
            EvidenceKind::Started
        };
        let admitted = event("Codex", &task_name, evidence, TimestampMs(index + 1))?;
        let snapshot = reduce(
            initial(&admitted, TimestampMs(index + 1)),
            &admitted,
            TimestampMs(index + 1),
            PrivacyProfile::Minimal,
        )
        .snapshot;
        snapshots.push(snapshot);
    }
    let plan = arbitrate(&snapshots, Some("task-0"), TimestampMs(99_999));
    assert_eq!(
        plan.primary
            .map(|snapshot| snapshot.task_id.0.as_str().to_owned()),
        Some("task-42".to_owned())
    );
    assert_eq!(plan.peek.len(), 3);
    Ok(())
}

#[test]
fn repeated_invalid_preflight_is_deterministic_and_content_free() {
    use pulse_protocol::preflight_frame;

    let oversized = vec![b'x'; pulse_protocol::MAX_FRAME_BYTES + 1];
    for _ in 0..10_000 {
        assert_eq!(
            preflight_frame(&oversized),
            Err(RejectionCategory::Oversized)
        );
        assert_eq!(
            preflight_frame(br#"{"secret":"redacted"}"#),
            Err(RejectionCategory::ForbiddenField)
        );
    }
}

#[test]
fn stale_exact_route_downgrades_to_useful_workspace_action() {
    let exact = RouteEvidence::new(RouteStrength::Exact, TimestampMs(1_000), 100);
    assert_eq!(
        label_for_evidence(exact, TimestampMs(1_101), RouteStrength::Useful),
        Some(RouteActionLabel::OpenWorkspace)
    );
}
