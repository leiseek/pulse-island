//! W3 shim fail-open contract tests.

use pulse_domain::TimestampMs;
#[cfg(target_env = "msvc")]
use pulse_link_shim::ExistingLinkIngressDelivery;
use pulse_link_shim::{
    run_shim_native_transport, run_shim_preflight, sanitize_codex_hook, NativeShimObservation,
    ShimDelivery, ShimDeliveryAttempt, ShimRunReport, SHIM_INPUT_LIMIT_BYTES,
};
use pulse_protocol::{EvidenceKind, RejectionCategory, ShimExitStatus};
use pulse_win32::LinkLocalObjectNames;
#[cfg(target_env = "msvc")]
use pulse_win32_link::serve_one_ingress_frame;
use pulse_win32_link::{InheritedHandoffPipe, LinkTransportNativeApi, RawLinkHandle};

#[derive(Default)]
struct FakeDelivery {
    attempts: Vec<ShimDeliveryAttempt>,
    delivered: bool,
}

impl ShimDelivery for FakeDelivery {
    fn deliver(&mut self, attempt: ShimDeliveryAttempt) -> bool {
        self.attempts.push(attempt);
        self.delivered
    }
}

#[test]
fn shim_safe_mode_exits_success_without_delivery() {
    let mut delivery = FakeDelivery::default();

    let report = run_shim_preflight(b"{\"version\":1}", true, &mut delivery);

    assert_eq!(
        report,
        ShimRunReport {
            exit_status: ShimExitStatus::Success,
            forwarded: false,
            rejection: None,
        }
    );
    assert!(delivery.attempts.is_empty());
}

#[test]
fn shim_oversized_input_is_fail_open_and_not_forwarded() {
    let mut delivery = FakeDelivery::default();
    let input = vec![b'x'; SHIM_INPUT_LIMIT_BYTES + 1];

    let report = run_shim_preflight(&input, false, &mut delivery);

    assert_eq!(report.exit_status, ShimExitStatus::Success);
    assert!(!report.forwarded);
    assert_eq!(report.rejection, Some(RejectionCategory::Oversized));
    assert!(delivery.attempts.is_empty());
}

#[test]
fn shim_valid_input_attempts_delivery_but_still_fails_open_when_link_is_unavailable() {
    let mut delivery = FakeDelivery::default();

    let report = run_shim_preflight(
        b"{\"version\":1,\"event\":\"activity\"}",
        false,
        &mut delivery,
    );

    assert_eq!(report.exit_status, ShimExitStatus::Success);
    assert!(!report.forwarded);
    assert_eq!(report.rejection, None);
    assert_eq!(delivery.attempts.len(), 1);
    assert_eq!(delivery.attempts[0].byte_len, 32);
}

#[test]
fn shim_accepts_spike_c_eight_kib_boundary_before_delivery() {
    let mut delivery = FakeDelivery {
        attempts: Vec::new(),
        delivered: true,
    };
    let input = vec![b'x'; SHIM_INPUT_LIMIT_BYTES];

    let report = run_shim_preflight(&input, false, &mut delivery);

    assert_eq!(report.exit_status, ShimExitStatus::Success);
    assert!(report.forwarded);
    assert_eq!(report.rejection, None);
    assert_eq!(delivery.attempts.len(), 1);
    assert_eq!(delivery.attempts[0].byte_len, SHIM_INPUT_LIMIT_BYTES);
}

#[test]
fn codex_sanitizer_keeps_only_session_identity_and_event_category(
) -> Result<(), Box<dyn std::error::Error>> {
    let envelope = sanitize_codex_hook(
        br#"{"session_id":"session-123","hook_event_name":"permissionRequest","cwd":"D:\\synthetic"}"#,
        TimestampMs(7),
    )?;

    assert_eq!(envelope.provider.as_str(), "codex_cli");
    assert_eq!(envelope.task.as_str(), "session-123");
    assert_eq!(envelope.evidence, EvidenceKind::Waiting);
    assert!(!envelope.forbidden_field_seen);
    Ok(())
}

#[test]
fn codex_sanitizer_rejects_unknown_or_content_bearing_fields() {
    let result = sanitize_codex_hook(
        br#"{"session_id":"session-123","hook_event_name":"sessionStart","prompt":"private"}"#,
        TimestampMs(7),
    );

    assert_eq!(result, Err(RejectionCategory::ForbiddenField));
}

#[test]
fn codex_sanitizer_accepts_documented_codex_event_casing() -> Result<(), Box<dyn std::error::Error>>
{
    for (event, expected) in [
        ("SessionStart", EvidenceKind::Started),
        ("UserPromptSubmit", EvidenceKind::Activity),
        ("PermissionRequest", EvidenceKind::Waiting),
        ("Stop", EvidenceKind::Activity),
    ] {
        let input = serde_json::json!({
            "session_id": "session-123",
            "hook_event_name": event,
        });
        let envelope = sanitize_codex_hook(&serde_json::to_vec(&input)?, TimestampMs(7))?;
        assert_eq!(envelope.evidence, expected);
    }
    Ok(())
}

#[test]
fn native_shim_uses_existing_link_ingress_without_launching_link(
) -> Result<(), Box<dyn std::error::Error>> {
    let api = FakeNativeShimApi::default();

    let report = run_shim_native_transport(
        br#"{"version":1,"event":"synthetic"}"#,
        false,
        test_names(),
        NativeShimObservation::ExistingLinkAcceptsIngress,
        api.clone(),
    );

    assert_eq!(report.preflight.exit_status, ShimExitStatus::Success);
    assert!(report.preflight.forwarded);
    let transport = report
        .transport
        .ok_or_else(|| std::io::Error::other("missing transport report"))?;
    assert!(transport.existing_ingress_attempted);
    assert!(!transport.link_launch_requested);
    assert!(!transport.handoff_pipe_created);
    assert!(!transport.command_line_payload_leaked);
    assert!(!transport.environment_payload_leaked);
    assert!(!transport.temp_filename_payload_leaked);
    assert_eq!(transport.native_transport_commands, 0);
    assert_eq!(api.created_handles, 0);
    Ok(())
}

#[test]
fn native_shim_creates_handoff_pipe_for_first_wake_without_payload_leaks(
) -> Result<(), Box<dyn std::error::Error>> {
    let report = run_shim_native_transport(
        br#"{"version":1,"event":"synthetic"}"#,
        false,
        test_names(),
        NativeShimObservation::NoExistingLink,
        FakeNativeShimApi::default(),
    );

    assert_eq!(report.preflight.exit_status, ShimExitStatus::Success);
    assert!(!report.preflight.forwarded);
    let transport = report
        .transport
        .ok_or_else(|| std::io::Error::other("missing transport report"))?;
    assert!(transport.existing_ingress_attempted);
    assert!(transport.link_launch_requested);
    assert!(transport.handoff_pipe_created);
    assert!(!transport.command_line_payload_leaked);
    assert!(!transport.environment_payload_leaked);
    assert!(!transport.temp_filename_payload_leaked);
    assert_eq!(transport.native_transport_commands, 1);
    Ok(())
}

#[test]
fn native_shim_fails_open_when_handoff_pipe_setup_fails() -> Result<(), Box<dyn std::error::Error>>
{
    let report = run_shim_native_transport(
        br#"{"version":1,"event":"synthetic"}"#,
        false,
        test_names(),
        NativeShimObservation::NoExistingLink,
        FakeNativeShimApi {
            fail_handoff_pipe: true,
            ..FakeNativeShimApi::default()
        },
    );

    assert_eq!(report.preflight.exit_status, ShimExitStatus::Success);
    assert!(!report.preflight.forwarded);
    let transport = report
        .transport
        .ok_or_else(|| std::io::Error::other("missing transport report"))?;
    assert!(transport.existing_ingress_attempted);
    assert!(!transport.link_launch_requested);
    assert!(!transport.handoff_pipe_created);
    assert_eq!(transport.native_transport_commands, 0);
    Ok(())
}

#[test]
fn native_shim_does_not_touch_transport_after_rejected_input() {
    let report = run_shim_native_transport(
        br#"{"prompt":"raw content"}"#,
        false,
        test_names(),
        NativeShimObservation::NoExistingLink,
        FakeNativeShimApi::default(),
    );

    assert_eq!(report.preflight.exit_status, ShimExitStatus::Success);
    assert_eq!(
        report.preflight.rejection,
        Some(RejectionCategory::ForbiddenField)
    );
    assert_eq!(report.transport, None);
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_existing_link_delivery_round_trips_a_content_free_header(
) -> Result<(), Box<dyn std::error::Error>> {
    let names = LinkLocalObjectNames::derive("shim-runtime", "sid", "session", 1);
    let server_names = names.clone();
    let server = std::thread::spawn(move || serve_one_ingress_frame(server_names, 28));
    let mut delivery = ExistingLinkIngressDelivery::new(names);
    let report = run_shim_preflight(br#"{"version":1,"event":"activity"}"#, false, &mut delivery);
    let received = server
        .join()
        .map_err(|_| std::io::Error::other("ingress server thread panicked"))?
        .map_err(|error| std::io::Error::other(format!("ingress server failed: {error:?}")))?;
    assert_eq!(report.exit_status, ShimExitStatus::Success);
    assert!(report.forwarded);
    assert_eq!(received.len(), 28);
    Ok(())
}

fn test_names() -> LinkLocalObjectNames {
    LinkLocalObjectNames::derive("install", "sid", "session", 1)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FakeNativeShimApi {
    created_handles: isize,
    fail_handoff_pipe: bool,
}

impl FakeNativeShimApi {
    fn next_handle(&mut self) -> Option<RawLinkHandle> {
        self.created_handles += 1;
        RawLinkHandle::new(self.created_handles)
    }
}

impl LinkTransportNativeApi for FakeNativeShimApi {
    fn create_mutex(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn create_ingress_pipe(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn create_island_pipe(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn create_inherited_handoff_pipe(&mut self) -> Option<InheritedHandoffPipe> {
        if self.fail_handoff_pipe {
            return None;
        }
        let read = self.next_handle()?;
        let write = self.next_handle()?;
        Some(InheritedHandoffPipe { read, write })
    }

    fn connect_island_client(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn close_handle(&mut self, _handle: RawLinkHandle) -> bool {
        true
    }
}
