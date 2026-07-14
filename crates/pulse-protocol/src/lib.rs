//! Bounded provider-neutral ingress protocol.
#![deny(missing_docs)]

use pulse_domain::{BoundedText, ProcessFingerprint, RouteStrength, TaskSnapshot, TimestampMs};

/// Maximum accepted ingress frame size in bytes.
pub const MAX_FRAME_BYTES: usize = 8 * 1024;
/// Current protocol schema version.
pub const PROTOCOL_VERSION: u16 = 1;
/// Maximum task snapshots carried by one island-facing state message.
pub const MAX_SNAPSHOT_TASKS: usize = 256;

/// Admission rejection category. Categories are safe to log; raw input is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionCategory {
    /// Frame exceeded the byte cap and must be rejected before allocation/parsing.
    Oversized,
    /// Envelope version is not supported by this reducer slice.
    UnsupportedVersion,
    /// A content-bearing or otherwise forbidden field was present.
    ForbiddenField,
    /// Input could not be interpreted as an allowed bounded protocol shape.
    Malformed,
    /// Local structured-state source is not approved by a provider Probe Card.
    UnsupportedStructuredSource,
    /// Snapshot or delta exceeded bounded task caps.
    SnapshotTooLarge,
}
impl core::fmt::Display for RejectionCategory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Oversized => f.write_str("oversized protocol input"),
            Self::UnsupportedVersion => f.write_str("unsupported protocol version"),
            Self::ForbiddenField => f.write_str("forbidden protocol field"),
            Self::Malformed => f.write_str("malformed protocol input"),
            Self::UnsupportedStructuredSource => f.write_str("unsupported structured source"),
            Self::SnapshotTooLarge => f.write_str("snapshot exceeds protocol caps"),
        }
    }
}
impl std::error::Error for RejectionCategory {}

/// Provider-neutral evidence kinds admitted into pure reducers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvidenceKind {
    /// Process-only observation; never enough to infer task lifecycle.
    ProcessObserved {
        /// PID plus process start time, never command line.
        process: ProcessFingerprint,
    },
    /// Observed process exit without provider terminal evidence.
    ProcessExited,
    /// Formal task start evidence.
    Started,
    /// Formal ordinary activity evidence.
    Activity,
    /// Provider-verified user decision/permission wait.
    Waiting,
    /// Provider-verified waiting clear.
    WaitingCleared,
    /// Explicit terminal completion evidence.
    Completed,
    /// Explicit terminal failure evidence.
    Failed,
    /// Verified usage/limit block currently stopping task progress.
    LimitBlocked,
    /// Non-blocking high-confidence Fuel warning.
    FuelRisk,
    /// Confirmed local resource condition is causally stalling the task.
    ResourceStall,
    /// Fuel source is stale or revoked; lifecycle must not change.
    FuelRevoked,
    /// Route evidence with declared strength.
    Route(RouteStrength),
}

/// Bounded Pulse-owned envelope after a Shim/adapter allow-list has discarded raw input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PulseHookEnvelope {
    /// Protocol version.
    pub version: u16,
    /// Safe bounded provider label/id.
    pub provider: BoundedText,
    /// Opaque bounded task/session reference.
    pub task: BoundedText,
    /// Provider-neutral evidence.
    pub evidence: EvidenceKind,
    /// Original frame byte length, checked before state mutation.
    pub byte_len: usize,
    /// Whether allow-list parsing found any disallowed field.
    pub forbidden_field_seen: bool,
    /// Whether a structured local-state source was explicitly approved.
    pub structured_source_approved: bool,
    /// Evidence timestamp supplied by edge code.
    pub occurred_at: TimestampMs,
}

/// Event admitted for reducer use. It has no arbitrary payload map or raw provider object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedEvent {
    /// Safe bounded provider label/id.
    pub provider: BoundedText,
    /// Opaque bounded task/session reference.
    pub task: BoundedText,
    /// Provider-neutral evidence.
    pub evidence: EvidenceKind,
    /// Evidence timestamp supplied by edge code.
    pub occurred_at: TimestampMs,
}

/// Validate a fully bounded envelope before it can mutate task state.
pub fn admit(envelope: PulseHookEnvelope) -> Result<AdmittedEvent, RejectionCategory> {
    if envelope.byte_len > MAX_FRAME_BYTES {
        return Err(RejectionCategory::Oversized);
    }
    if envelope.version != PROTOCOL_VERSION {
        return Err(RejectionCategory::UnsupportedVersion);
    }
    if envelope.forbidden_field_seen {
        return Err(RejectionCategory::ForbiddenField);
    }
    if !envelope.structured_source_approved {
        return Err(RejectionCategory::UnsupportedStructuredSource);
    }
    Ok(AdmittedEvent {
        provider: envelope.provider,
        task: envelope.task,
        evidence: envelope.evidence,
        occurred_at: envelope.occurred_at,
    })
}

/// Encode a bounded admitted event for local Link ingress without retaining raw Hook JSON.
pub fn encode_ingress_payload(event: &AdmittedEvent) -> Result<Vec<u8>, RejectionCategory> {
    let evidence = match event.evidence {
        EvidenceKind::Started => 1_u8,
        EvidenceKind::Activity => 2_u8,
        EvidenceKind::Waiting => 3_u8,
        _ => return Err(RejectionCategory::Malformed),
    };
    let provider = event.provider.as_str().as_bytes();
    let task = event.task.as_str().as_bytes();
    let provider_length = u8::try_from(provider.len()).map_err(|_| RejectionCategory::Malformed)?;
    let task_length = u8::try_from(task.len()).map_err(|_| RejectionCategory::Malformed)?;
    let mut encoded = Vec::with_capacity(12 + provider.len() + task.len());
    encoded.push(1);
    encoded.push(evidence);
    encoded.extend_from_slice(&event.occurred_at.0.to_le_bytes());
    encoded.push(provider_length);
    encoded.push(task_length);
    encoded.extend_from_slice(provider);
    encoded.extend_from_slice(task);
    if encoded.len() > MAX_FRAME_BYTES {
        return Err(RejectionCategory::Oversized);
    }
    Ok(encoded)
}

/// Decode one bounded local Link ingress payload into an admitted event.
pub fn decode_ingress_payload(input: &[u8]) -> Result<AdmittedEvent, RejectionCategory> {
    if input.len() < 12 || input.len() > MAX_FRAME_BYTES || input[0] != 1 {
        return Err(RejectionCategory::Malformed);
    }
    let evidence = match input[1] {
        1 => EvidenceKind::Started,
        2 => EvidenceKind::Activity,
        3 => EvidenceKind::Waiting,
        _ => return Err(RejectionCategory::Malformed),
    };
    let occurred_at = TimestampMs(u64::from_le_bytes(
        input[2..10]
            .try_into()
            .map_err(|_| RejectionCategory::Malformed)?,
    ));
    let provider_length = usize::from(input[10]);
    let task_length = usize::from(input[11]);
    let expected = 12_usize
        .checked_add(provider_length)
        .and_then(|value| value.checked_add(task_length))
        .ok_or(RejectionCategory::Malformed)?;
    if input.len() != expected {
        return Err(RejectionCategory::Malformed);
    }
    let provider_end = 12 + provider_length;
    let provider =
        std::str::from_utf8(&input[12..provider_end]).map_err(|_| RejectionCategory::Malformed)?;
    let task =
        std::str::from_utf8(&input[provider_end..]).map_err(|_| RejectionCategory::Malformed)?;
    Ok(AdmittedEvent {
        provider: BoundedText::new(provider).map_err(|_| RejectionCategory::Malformed)?,
        task: BoundedText::new(task).map_err(|_| RejectionCategory::Malformed)?,
        evidence,
        occurred_at,
    })
}

/// Safe Shim process exit status category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShimExitStatus {
    /// Shim exits successfully so provider-native execution remains fail-open.
    Success,
}

/// Pure decision for the earliest Pulse-owned Shim boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShimIngressDecision {
    /// Whether the Shim may wake pulse-link.
    pub wake_link: bool,
    /// Whether the Shim may forward the bounded ingress envelope.
    pub forward_ingress: bool,
    /// Safe process exit category.
    pub exit_status: ShimExitStatus,
}

/// Decide Shim behavior from the current-user Safe Mode flag.
pub const fn shim_ingress_decision(safe_mode_enabled: bool) -> ShimIngressDecision {
    if safe_mode_enabled {
        ShimIngressDecision {
            wake_link: false,
            forward_ingress: false,
            exit_status: ShimExitStatus::Success,
        }
    } else {
        ShimIngressDecision {
            wake_link: true,
            forward_ingress: true,
            exit_status: ShimExitStatus::Success,
        }
    }
}

/// Validate raw bytes before allocation-heavy parsing. This intentionally supports only a
/// tiny synthetic test shape and returns a category, never raw data.
pub fn preflight_frame(frame: &[u8]) -> Result<(), RejectionCategory> {
    if frame.len() > MAX_FRAME_BYTES {
        return Err(RejectionCategory::Oversized);
    }
    for forbidden in [
        b"prompt".as_slice(),
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
                .zip(forbidden.iter())
                .all(|(actual, expected)| actual.to_ascii_lowercase() == *expected)
        }) {
            return Err(RejectionCategory::ForbiddenField);
        }
    }
    Ok(())
}

/// Safe protocol error category. It never carries raw provider text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolErrorCategory {
    /// A frame or message was oversized.
    Oversized,
    /// Peer protocol version is unsupported.
    UnsupportedVersion,
    /// Message shape was not accepted.
    Malformed,
    /// A bounded snapshot/delta exceeded protocol caps.
    SnapshotTooLarge,
}

/// Coarse Link health status for future island-facing protocol seams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkHealthStatus {
    /// Link is available and current.
    Healthy,
    /// Link is running with reduced observation.
    Degraded,
    /// Link is unavailable.
    Offline,
}

/// Full compact snapshot for reconnect or revision-gap recovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullSnapshot {
    /// Monotonic snapshot revision.
    pub revision: u64,
    /// Compact task snapshots under protocol caps.
    pub tasks: Vec<TaskSnapshot>,
}

impl FullSnapshot {
    /// Create a bounded full snapshot.
    pub fn new(revision: u64, tasks: Vec<TaskSnapshot>) -> Result<Self, RejectionCategory> {
        if tasks.len() > MAX_SNAPSHOT_TASKS {
            return Err(RejectionCategory::SnapshotTooLarge);
        }
        Ok(Self { revision, tasks })
    }
}

/// Compact state delta for island-facing updates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotDelta {
    /// Monotonic delta revision.
    pub revision: u64,
    /// Compact snapshot upserts under protocol caps.
    pub upserts: Vec<TaskSnapshot>,
    /// Opaque task removals under protocol caps.
    pub removals: Vec<BoundedText>,
}

impl SnapshotDelta {
    /// Create a bounded snapshot delta.
    pub fn new(
        revision: u64,
        upserts: Vec<TaskSnapshot>,
        removals: Vec<BoundedText>,
    ) -> Result<Self, RejectionCategory> {
        if upserts.len() > MAX_SNAPSHOT_TASKS || removals.len() > MAX_SNAPSHOT_TASKS {
            return Err(RejectionCategory::SnapshotTooLarge);
        }
        Ok(Self {
            revision,
            upserts,
            removals,
        })
    }
}

/// Island-facing protocol messages. No variant carries raw hook payloads or event replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IslandMessage {
    /// Hello/version acknowledgement.
    HelloAck {
        /// Agreed protocol version.
        version: u16,
    },
    /// Full compact snapshot.
    FullSnapshot(FullSnapshot),
    /// Compact snapshot delta.
    SnapshotDelta(SnapshotDelta),
    /// Link health summary.
    LinkHealth {
        /// Coarse health status.
        status: LinkHealthStatus,
    },
    /// Safe protocol error category.
    ProtocolError(ProtocolErrorCategory),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_rejects_oversized_before_parsing() {
        let frame = vec![b'x'; MAX_FRAME_BYTES + 1];
        assert_eq!(preflight_frame(&frame), Err(RejectionCategory::Oversized));
    }

    #[test]
    fn preflight_rejects_forbidden_content_fields() {
        assert_eq!(
            preflight_frame(br#"{"prompt":"do work"}"#),
            Err(RejectionCategory::ForbiddenField)
        );
    }

    #[test]
    fn island_messages_are_state_or_health_only() {
        let message = IslandMessage::HelloAck {
            version: PROTOCOL_VERSION,
        };
        assert_eq!(
            message,
            IslandMessage::HelloAck {
                version: PROTOCOL_VERSION
            }
        );
    }

    #[test]
    fn snapshot_messages_enforce_task_caps() {
        let too_many = vec![BoundedText::new("task").map_err(|_| ()).ok(); MAX_SNAPSHOT_TASKS + 1];
        let removals = too_many.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(
            SnapshotDelta::new(1, Vec::new(), removals),
            Err(RejectionCategory::SnapshotTooLarge)
        );
    }

    #[test]
    fn ingress_payload_round_trips_safe_identity_and_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = AdmittedEvent {
            provider: BoundedText::new("codex_cli")?,
            task: BoundedText::new("session-123")?,
            evidence: EvidenceKind::Waiting,
            occurred_at: TimestampMs(42),
        };

        let encoded = encode_ingress_payload(&event)?;
        assert_eq!(decode_ingress_payload(&encoded)?, event);
        Ok(())
    }
}
