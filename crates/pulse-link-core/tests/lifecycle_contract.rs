//! W3 Link lifecycle contract tests.

use pulse_link_core::{LinkLifecycle, LinkLifecycleEvent, LinkLifecycleState};

#[test]
fn wake_then_ready_enters_warm_without_becoming_daemon_idle() {
    let lifecycle = LinkLifecycle::new()
        .apply(LinkLifecycleEvent::WakeRequested)
        .apply(LinkLifecycleEvent::RuntimeReady);

    assert_eq!(lifecycle.state(), LinkLifecycleState::Warm);
    assert!(!lifecycle.is_terminal());
}

#[test]
fn first_valid_event_enters_active_and_grace_event_resumes_active() {
    let lifecycle = LinkLifecycle::new()
        .apply(LinkLifecycleEvent::WakeRequested)
        .apply(LinkLifecycleEvent::RuntimeReady)
        .apply(LinkLifecycleEvent::ValidActiveTaskEvent);

    assert_eq!(lifecycle.state(), LinkLifecycleState::Active);

    let resumed = lifecycle
        .apply(LinkLifecycleEvent::LastActiveTaskTerminal)
        .apply(LinkLifecycleEvent::ValidActiveTaskEvent);

    assert_eq!(resumed.state(), LinkLifecycleState::Active);
}

#[test]
fn island_attach_detach_moves_between_island_active_and_drop_mode() {
    let lifecycle = LinkLifecycle::new()
        .apply(LinkLifecycleEvent::WakeRequested)
        .apply(LinkLifecycleEvent::RuntimeReady)
        .apply(LinkLifecycleEvent::ValidActiveTaskEvent)
        .apply(LinkLifecycleEvent::IslandAttached);

    assert_eq!(lifecycle.state(), LinkLifecycleState::IslandActive);

    let detached = lifecycle.apply(LinkLifecycleEvent::IslandDetached);

    assert_eq!(detached.state(), LinkLifecycleState::DropMode);
}

#[test]
fn terminal_task_starts_grace_then_checkpoint_exit_then_not_running() {
    let lifecycle = LinkLifecycle::new()
        .apply(LinkLifecycleEvent::WakeRequested)
        .apply(LinkLifecycleEvent::RuntimeReady)
        .apply(LinkLifecycleEvent::ValidActiveTaskEvent)
        .apply(LinkLifecycleEvent::LastActiveTaskTerminal);

    assert_eq!(lifecycle.state(), LinkLifecycleState::GracePeriod);

    let checkpointing = lifecycle.apply(LinkLifecycleEvent::GraceExpired);

    assert_eq!(checkpointing.state(), LinkLifecycleState::CheckpointAndExit);

    let stopped = checkpointing.apply(LinkLifecycleEvent::CheckpointComplete);

    assert_eq!(stopped.state(), LinkLifecycleState::NotRunning);
    assert!(stopped.is_terminal());
}
