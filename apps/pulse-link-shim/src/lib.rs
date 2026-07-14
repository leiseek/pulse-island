//! Short-lived W3 Pulse Link shim.
#![deny(missing_docs)]

use pulse_domain::{BoundedText, TimestampMs};
use pulse_link_core::MAX_HOOK_INGRESS_PAYLOAD_BYTES;
#[cfg(target_env = "msvc")]
use pulse_link_core::{LinkFrameHeader, LinkMessageKind};
#[cfg(target_env = "msvc")]
use pulse_protocol::{admit, encode_ingress_payload};
use pulse_protocol::{
    preflight_frame, shim_ingress_decision, EvidenceKind, PulseHookEnvelope, RejectionCategory,
    ShimExitStatus, PROTOCOL_VERSION,
};
use pulse_win32::LinkLocalObjectNames;
#[cfg(target_env = "msvc")]
use pulse_win32_link::send_ingress_message_and_wait_ack;
use pulse_win32_link::{LinkTransportCommand, LinkTransportNativeApi, LinkTransportNativeBackend};

/// Maximum bytes the W3 shim reads from ordinary Hook input.
pub const SHIM_INPUT_LIMIT_BYTES: usize = MAX_HOOK_INGRESS_PAYLOAD_BYTES;

/// Convert one Codex Hook JSON object into a bounded Pulse envelope without retaining raw input.
///
/// The sanitizer is deliberately strict: it accepts only the small official Hook fields needed
/// for P0 lifecycle evidence and rejects unknown or content-bearing fields before transport.
pub fn sanitize_codex_hook(
    input: &[u8],
    occurred_at: TimestampMs,
) -> Result<PulseHookEnvelope, RejectionCategory> {
    preflight_codex_frame(input)?;
    let value: serde_json::Value =
        serde_json::from_slice(input).map_err(|_| RejectionCategory::Malformed)?;
    let object = value.as_object().ok_or(RejectionCategory::Malformed)?;
    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "session_id"
                | "hook_event_name"
                | "cwd"
                | "turn_id"
                | "permission_mode"
                | "source"
                | "tool_name"
        ) {
            return Err(RejectionCategory::ForbiddenField);
        }
    }
    let session_id = object
        .get("session_id")
        .and_then(serde_json::Value::as_str)
        .ok_or(RejectionCategory::Malformed)?;
    let event_name = object
        .get("hook_event_name")
        .and_then(serde_json::Value::as_str)
        .ok_or(RejectionCategory::Malformed)?;
    if let Some(cwd) = object.get("cwd") {
        if !cwd.is_string() {
            return Err(RejectionCategory::Malformed);
        }
    }
    let evidence = match event_name {
        "SessionStart" | "sessionStart" => EvidenceKind::Started,
        "UserPromptSubmit" | "PreToolUse" | "PostToolUse" | "SubagentStart" | "SubagentStop"
        | "preToolUse" | "postToolUse" | "subagentStart" | "subagentStop" => EvidenceKind::Activity,
        "PermissionRequest" | "permissionRequest" => EvidenceKind::Waiting,
        "Stop" | "stop" => EvidenceKind::Activity,
        _ => return Err(RejectionCategory::Malformed),
    };
    Ok(PulseHookEnvelope {
        version: PROTOCOL_VERSION,
        provider: BoundedText::new("codex_cli").map_err(|_| RejectionCategory::Malformed)?,
        task: BoundedText::new(session_id).map_err(|_| RejectionCategory::Malformed)?,
        evidence,
        byte_len: input.len(),
        forbidden_field_seen: false,
        structured_source_approved: true,
        occurred_at,
    })
}

fn preflight_codex_frame(frame: &[u8]) -> Result<(), RejectionCategory> {
    if frame.len() > pulse_link_core::MAX_HOOK_INGRESS_PAYLOAD_BYTES {
        return Err(RejectionCategory::Oversized);
    }
    for forbidden in [
        b"transcript".as_slice(),
        b"api_key".as_slice(),
        b"secret".as_slice(),
        b"password".as_slice(),
        b"credential".as_slice(),
        b"bearer".as_slice(),
    ] {
        if frame.windows(forbidden.len()).any(|window| {
            window
                .iter()
                .zip(forbidden)
                .all(|(actual, expected)| actual.to_ascii_lowercase() == *expected)
        }) {
            return Err(RejectionCategory::ForbiddenField);
        }
    }
    Ok(())
}

/// Build a content-free header plus bounded payload from one Codex Hook object.
#[cfg(target_env = "msvc")]
pub fn codex_hook_frame(
    input: &[u8],
    occurred_at: TimestampMs,
    request_id: u64,
) -> Result<(LinkFrameHeader, Vec<u8>), RejectionCategory> {
    let envelope = sanitize_codex_hook(input, occurred_at)?;
    let event = admit(envelope)?;
    let payload = encode_ingress_payload(&event)?;
    let header = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id,
        payload_length: u32::try_from(payload.len()).map_err(|_| RejectionCategory::Oversized)?,
    };
    Ok((header, payload))
}

/// Sanitize and deliver one Codex Hook to an existing Link ingress pipe.
#[cfg(target_env = "msvc")]
pub fn send_codex_hook_to_link(
    names: &LinkLocalObjectNames,
    input: &[u8],
    occurred_at: TimestampMs,
    request_id: u64,
) -> Result<(), RejectionCategory> {
    let (header, payload) = codex_hook_frame(input, occurred_at, request_id)?;
    send_ingress_message_and_wait_ack(names, &header, &payload)
        .map_err(|_| RejectionCategory::Malformed)
}

/// Content-free delivery attempt metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShimDeliveryAttempt {
    /// Validated input byte length.
    pub byte_len: usize,
}

/// Abstract delivery seam for existing/future Link transport.
pub trait ShimDelivery {
    /// Attempt delivery. `false` still results in fail-open shim success.
    fn deliver(&mut self, attempt: ShimDeliveryAttempt) -> bool;
}

/// Delivery adapter for an already-running Windows Link ingress pipe.
///
/// Raw Hook input has already passed bounded preflight when this adapter is called. Only a
/// content-free W3 frame header crosses the pipe; provider-specific sanitization and payload
/// transport remain adapter work.
#[cfg(target_env = "msvc")]
pub struct ExistingLinkIngressDelivery {
    names: LinkLocalObjectNames,
    next_request_id: u64,
}

#[cfg(target_env = "msvc")]
impl ExistingLinkIngressDelivery {
    /// Create an existing-Link delivery adapter for one scoped local transport name set.
    pub fn new(names: LinkLocalObjectNames) -> Self {
        Self {
            names,
            next_request_id: 1,
        }
    }
}

#[cfg(target_env = "msvc")]
impl ShimDelivery for ExistingLinkIngressDelivery {
    fn deliver(&mut self, _attempt: ShimDeliveryAttempt) -> bool {
        let header = LinkFrameHeader {
            message_kind: LinkMessageKind::HookEnvelope,
            request_id: self.next_request_id,
            payload_length: 0,
        };
        self.next_request_id = self.next_request_id.saturating_add(1);
        pulse_win32_link::send_ingress_frame_and_wait_ack(&self.names, &header.encode()).is_ok()
    }
}

/// Content-free shim run report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShimRunReport {
    /// Fail-open process exit category.
    pub exit_status: ShimExitStatus,
    /// Whether Link acknowledged delivery.
    pub forwarded: bool,
    /// Safe rejection category, when preflight rejected the input.
    pub rejection: Option<RejectionCategory>,
}

/// Native transport observation available to the short-lived Shim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeShimObservation {
    /// A scoped Link is already reachable and acknowledges ingress.
    ExistingLinkAcceptsIngress,
    /// No existing Link accepted ingress, so Shim may initiate first wake.
    NoExistingLink,
}

/// Content-free native transport result for one Shim run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeShimTransportReport {
    /// Whether Shim attempted the existing-Link ingress path first.
    pub existing_ingress_attempted: bool,
    /// Whether Shim would request a Link process launch after handoff setup.
    pub link_launch_requested: bool,
    /// Whether an inherited anonymous handoff pipe was created.
    pub handoff_pipe_created: bool,
    /// Whether the validated payload leaked into command-line arguments.
    pub command_line_payload_leaked: bool,
    /// Whether the validated payload leaked into environment variables.
    pub environment_payload_leaked: bool,
    /// Whether the validated payload leaked into temporary filenames.
    pub temp_filename_payload_leaked: bool,
    /// Number of native transport setup commands completed.
    pub native_transport_commands: usize,
}

/// Content-free report for a Shim preflight plus optional native transport attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeShimRunReport {
    /// Bounded input preflight and fail-open result.
    pub preflight: ShimRunReport,
    /// Native transport result, absent when input was rejected or Safe Mode bypassed delivery.
    pub transport: Option<NativeShimTransportReport>,
}

/// Run bounded shim preflight and optional delivery without retaining raw input.
pub fn run_shim_preflight(
    input: &[u8],
    safe_mode_enabled: bool,
    delivery: &mut impl ShimDelivery,
) -> ShimRunReport {
    let decision = shim_ingress_decision(safe_mode_enabled);
    if !decision.wake_link || !decision.forward_ingress {
        return ShimRunReport {
            exit_status: decision.exit_status,
            forwarded: false,
            rejection: None,
        };
    }
    if input.len() > SHIM_INPUT_LIMIT_BYTES {
        return rejected(RejectionCategory::Oversized);
    }
    if let Err(rejection) = preflight_frame(input) {
        return rejected(rejection);
    }
    let forwarded = delivery.deliver(ShimDeliveryAttempt {
        byte_len: input.len(),
    });
    ShimRunReport {
        exit_status: ShimExitStatus::Success,
        forwarded,
        rejection: None,
    }
}

/// Run bounded Shim preflight and the W3 native transport seam.
pub fn run_shim_native_transport<A>(
    input: &[u8],
    safe_mode_enabled: bool,
    _names: LinkLocalObjectNames,
    observation: NativeShimObservation,
    api: A,
) -> NativeShimRunReport
where
    A: LinkTransportNativeApi,
{
    let mut delivery = NativeShimDelivery { observation };
    let preflight = run_shim_preflight(input, safe_mode_enabled, &mut delivery);
    if preflight.rejection.is_some()
        || safe_mode_enabled
        || preflight.exit_status != ShimExitStatus::Success
    {
        return NativeShimRunReport {
            preflight,
            transport: None,
        };
    }

    if preflight.forwarded {
        return NativeShimRunReport {
            preflight,
            transport: Some(NativeShimTransportReport {
                existing_ingress_attempted: true,
                link_launch_requested: false,
                handoff_pipe_created: false,
                command_line_payload_leaked: false,
                environment_payload_leaked: false,
                temp_filename_payload_leaked: false,
                native_transport_commands: 0,
            }),
        };
    }

    let mut backend = LinkTransportNativeBackend::new(api);
    let handoff_created = backend
        .apply_command(LinkTransportCommand::CreateInheritedHandoffPipe)
        .is_ok();

    NativeShimRunReport {
        preflight,
        transport: Some(NativeShimTransportReport {
            existing_ingress_attempted: true,
            link_launch_requested: handoff_created,
            handoff_pipe_created: handoff_created,
            command_line_payload_leaked: false,
            environment_payload_leaked: false,
            temp_filename_payload_leaked: false,
            native_transport_commands: usize::from(handoff_created),
        }),
    }
}

fn rejected(rejection: RejectionCategory) -> ShimRunReport {
    ShimRunReport {
        exit_status: ShimExitStatus::Success,
        forwarded: false,
        rejection: Some(rejection),
    }
}

struct NativeShimDelivery {
    observation: NativeShimObservation,
}

impl ShimDelivery for NativeShimDelivery {
    fn deliver(&mut self, _attempt: ShimDeliveryAttempt) -> bool {
        self.observation == NativeShimObservation::ExistingLinkAcceptsIngress
    }
}
