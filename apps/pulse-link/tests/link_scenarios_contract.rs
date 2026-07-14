//! W3 Spike C synthetic scenario harness contract tests.

use pulse_domain::Lifecycle;
use pulse_link::{run_link_scenario, LinkScenario, LinkScenarioReport};
use pulse_link_core::LinkLifecycleState;
use pulse_protocol::ShimExitStatus;

#[test]
fn c0_existing_link_delivery_reuses_link_and_serves_late_snapshot() {
    let report = run_link_scenario(LinkScenario::C0ExistingLinkDelivery);

    assert_eq!(
        report,
        LinkScenarioReport {
            scenario: LinkScenario::C0ExistingLinkDelivery,
            shim_exit_status: ShimExitStatus::Success,
            shim_forwarded: true,
            link_process_launches: 0,
            lifecycle_state: LinkLifecycleState::IslandActive,
            active_tasks: 1,
            recent_terminal_tasks: 0,
            restored_degraded_tasks: 0,
            inherited_handoff_used: false,
            command_line_payload_leaked: false,
            island_attached: true,
            island_snapshot_revision: Some(1),
            terminal_lifecycle: None,
            provider_affected: false,
            raw_payload_persisted: false,
        }
    );
}

#[test]
fn c1_first_hook_wakes_one_link_and_late_island_sees_snapshot() {
    let report = run_link_scenario(LinkScenario::C1FirstHookWakesLink);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert!(report.shim_forwarded);
    assert_eq!(report.link_process_launches, 1);
    assert!(report.inherited_handoff_used);
    assert!(!report.command_line_payload_leaked);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::IslandActive);
    assert_eq!(report.active_tasks, 1);
    assert_eq!(report.island_snapshot_revision, Some(1));
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c2_parallel_shim_race_launches_one_link_and_keeps_state_bounded() {
    let report = run_link_scenario(LinkScenario::C2ParallelShimRace);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert!(report.shim_forwarded);
    assert_eq!(report.link_process_launches, 1);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::Active);
    assert_eq!(report.active_tasks, 1);
    assert_eq!(report.recent_terminal_tasks, 1);
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c3_link_unavailable_is_fail_open_without_launch_or_checkpoint() {
    let report = run_link_scenario(LinkScenario::C3LinkUnavailable);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert!(!report.shim_forwarded);
    assert_eq!(report.link_process_launches, 0);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::NotRunning);
    assert_eq!(report.active_tasks, 0);
    assert_eq!(report.recent_terminal_tasks, 0);
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c4_malformed_or_oversized_ingress_is_rejected_before_state_mutation() {
    let report = run_link_scenario(LinkScenario::C4MalformedOversizedIngress);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert!(!report.shim_forwarded);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::NotRunning);
    assert_eq!(report.active_tasks, 0);
    assert_eq!(report.recent_terminal_tasks, 0);
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c5_drop_mode_breadcrumb_keeps_terminal_without_ui_attachment() {
    let report = run_link_scenario(LinkScenario::C5DropModeBreadcrumb);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::GracePeriod);
    assert_eq!(report.active_tasks, 0);
    assert_eq!(report.recent_terminal_tasks, 1);
    assert!(!report.island_attached);
    assert_eq!(report.terminal_lifecycle, Some(Lifecycle::Completed));
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c6_island_attach_detach_reattach_preserves_single_task_snapshot() {
    let report = run_link_scenario(LinkScenario::C6IslandAttachDetachReattach);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert!(report.shim_forwarded);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::IslandActive);
    assert_eq!(report.active_tasks, 1);
    assert_eq!(report.recent_terminal_tasks, 0);
    assert!(report.island_attached);
    assert_eq!(report.island_snapshot_revision, Some(2));
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c7_restart_recovery_restores_active_breadcrumb_as_degraded_until_fresh_event() {
    let report = run_link_scenario(LinkScenario::C7LinkRestartRecovery);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert_eq!(report.link_process_launches, 1);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::IslandActive);
    assert_eq!(report.active_tasks, 1);
    assert_eq!(report.restored_degraded_tasks, 1);
    assert_eq!(report.island_snapshot_revision, Some(1));
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c8_grace_exit_checkpoints_and_stops_link() {
    let report = run_link_scenario(LinkScenario::C8GraceExit);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::NotRunning);
    assert_eq!(report.active_tasks, 0);
    assert_eq!(report.recent_terminal_tasks, 1);
    assert_eq!(report.terminal_lifecycle, Some(Lifecycle::Completed));
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}

#[test]
fn c9_event_storm_stays_within_breadcrumb_caps() {
    let report = run_link_scenario(LinkScenario::C9EventStorm);

    assert_eq!(report.shim_exit_status, ShimExitStatus::Success);
    assert!(report.shim_forwarded);
    assert_eq!(report.lifecycle_state, LinkLifecycleState::Active);
    assert_eq!(report.active_tasks, 128);
    assert_eq!(report.recent_terminal_tasks, 20);
    assert!(!report.provider_affected);
    assert!(!report.raw_payload_persisted);
}
