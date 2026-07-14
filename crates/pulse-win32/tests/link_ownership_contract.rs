//! W3 Link single-instance ownership contract tests.

use pulse_win32::{
    LinkLocalObjectNames, LinkOwnershipDecision, LinkOwnershipRegistry, LinkStartupObservation,
};

#[test]
fn first_link_start_owns_mutex_and_creates_pipe_servers() {
    let names = test_names();
    let mut registry = LinkOwnershipRegistry::default();

    let decision = registry.observe_start(&names, LinkStartupObservation::NoExistingObjects);

    assert_eq!(decision, LinkOwnershipDecision::OwnInstance);
    assert_eq!(registry.owner_count(&names), 1);
}

#[test]
fn second_start_reuses_existing_link_without_creating_duplicate_runtime() {
    let names = test_names();
    let mut registry = LinkOwnershipRegistry::default();

    assert_eq!(
        registry.observe_start(&names, LinkStartupObservation::NoExistingObjects),
        LinkOwnershipDecision::OwnInstance
    );
    assert_eq!(
        registry.observe_start(&names, LinkStartupObservation::MutexAlreadyOwned),
        LinkOwnershipDecision::ConnectToExisting
    );
    assert_eq!(registry.owner_count(&names), 1);
}

#[test]
fn stale_mutex_or_pipe_retry_is_bounded_and_fails_open() {
    let names = test_names();
    let mut registry = LinkOwnershipRegistry::default();

    assert_eq!(
        registry.observe_start(&names, LinkStartupObservation::StaleMutexOrPipe),
        LinkOwnershipDecision::RetryBounded
    );
    assert_eq!(
        registry.observe_start(&names, LinkStartupObservation::StaleMutexOrPipe),
        LinkOwnershipDecision::RetryBounded
    );
    assert_eq!(
        registry.observe_start(&names, LinkStartupObservation::StaleMutexOrPipe),
        LinkOwnershipDecision::FailOpen
    );
    assert_eq!(registry.owner_count(&names), 0);
}

#[test]
fn ownership_is_scoped_by_logon_session_names() {
    let session_a = LinkLocalObjectNames::derive("install", "sid", "session-a", 1);
    let session_b = LinkLocalObjectNames::derive("install", "sid", "session-b", 1);
    let mut registry = LinkOwnershipRegistry::default();

    assert_eq!(
        registry.observe_start(&session_a, LinkStartupObservation::NoExistingObjects),
        LinkOwnershipDecision::OwnInstance
    );
    assert_eq!(
        registry.observe_start(&session_b, LinkStartupObservation::NoExistingObjects),
        LinkOwnershipDecision::OwnInstance
    );

    assert_eq!(registry.owner_count(&session_a), 1);
    assert_eq!(registry.owner_count(&session_b), 1);
}

fn test_names() -> LinkLocalObjectNames {
    LinkLocalObjectNames::derive("install", "sid", "session", 1)
}
