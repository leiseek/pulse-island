//! W3 Link local object naming contract tests.

use pulse_win32::LinkLocalObjectNames;

#[test]
fn link_local_names_use_expected_scopes_and_protocol_version() {
    let names = LinkLocalObjectNames::derive(
        "install-123",
        "S-1-5-21-1004336348-1177238915-682003330-1001",
        "logon-session-0x3e7",
        1,
    );

    assert!(names.mutex.starts_with(r"Local\PulseIsland.Link."));
    assert!(names
        .ready_event
        .starts_with(r"Local\PulseIsland.LinkReady."));
    assert!(names.ingress_pipe.starts_with(r"\\.\pipe\PulseIsland."));
    assert!(names.island_pipe.starts_with(r"\\.\pipe\PulseIsland."));
    assert!(names.mutex.ends_with(".v1"));
    assert!(names.ingress_pipe.ends_with(".ingress.v1"));
    assert!(names.island_pipe.ends_with(".island.v1"));
    assert!(names.ready_event.ends_with(".v1"));
}

#[test]
fn link_local_names_do_not_leak_raw_sid_session_or_install_id() {
    let install_id = "install-raw-secretish";
    let user_sid = "S-1-5-21-raw-user-sid";
    let logon_session = "raw-logon-session";
    let names = LinkLocalObjectNames::derive(install_id, user_sid, logon_session, 1);

    for name in names.as_slice() {
        assert!(!name.contains(install_id));
        assert!(!name.contains(user_sid));
        assert!(!name.contains(logon_session));
        assert!(!name.contains("raw"));
    }
}

#[test]
fn link_local_names_are_stable_and_session_scoped() {
    let first = LinkLocalObjectNames::derive("install", "sid", "session-a", 1);
    let repeated = LinkLocalObjectNames::derive("install", "sid", "session-a", 1);
    let other_session = LinkLocalObjectNames::derive("install", "sid", "session-b", 1);

    assert_eq!(first, repeated);
    assert_ne!(first.ingress_pipe, other_session.ingress_pipe);
    assert_ne!(first.mutex, other_session.mutex);
}
