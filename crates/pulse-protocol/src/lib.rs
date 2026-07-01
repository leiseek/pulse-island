//! Bounded provider-neutral ingress protocol.
#![deny(missing_docs)]

use pulse_domain::{BoundedText, ProcessFingerprint, RouteStrength, TaskSnapshot, TimestampMs};

/// Maximum accepted ingress frame size in bytes.
pub const MAX_FRAME_BYTES: usize = 4096;
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
}
