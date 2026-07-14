//! Provider-neutral W3 Link/Shim core.
#![deny(missing_docs)]

use pulse_protocol::{
    FullSnapshot, IslandMessage, LinkHealthStatus, ProtocolErrorCategory, SnapshotDelta,
    PROTOCOL_VERSION,
};

/// Local Link frame magic. This is safe diagnostic metadata, not payload content.
pub const FRAME_MAGIC: [u8; 4] = *b"PILK";
/// Current Link frame major protocol version.
pub const LINK_PROTOCOL_MAJOR: u16 = 1;
/// Current Link frame minor protocol version.
pub const LINK_PROTOCOL_MINOR: u16 = 0;
/// Fixed byte size of the Spike C frame header.
pub const FRAME_HEADER_BYTES: usize = 28;
/// Maximum Hook ingress payload accepted by W3 Link/Shim.
pub const MAX_HOOK_INGRESS_PAYLOAD_BYTES: usize = 8 * 1024;
/// Maximum Island control/delta/health payload accepted by W3 Link/Shim.
pub const MAX_ISLAND_PAYLOAD_BYTES: usize = 8 * 1024;
/// Maximum full snapshot payload accepted by W3 Link/Shim.
pub const MAX_FULL_SNAPSHOT_PAYLOAD_BYTES: usize = 128 * 1024;

/// Spike C local frame message kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkMessageKind {
    /// Synthetic Hook ingress envelope.
    HookEnvelope,
    /// Island control request or bounded state response.
    IslandControl,
    /// Full snapshot response.
    FullSnapshot,
}

impl LinkMessageKind {
    /// Stable wire value.
    pub const fn wire_value(self) -> u16 {
        match self {
            Self::HookEnvelope => 1,
            Self::IslandControl => 2,
            Self::FullSnapshot => 3,
        }
    }

    /// Payload cap for this message kind.
    pub const fn max_payload_bytes(self) -> usize {
        match self {
            Self::HookEnvelope => MAX_HOOK_INGRESS_PAYLOAD_BYTES,
            Self::IslandControl => MAX_ISLAND_PAYLOAD_BYTES,
            Self::FullSnapshot => MAX_FULL_SNAPSHOT_PAYLOAD_BYTES,
        }
    }

    fn from_wire(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::HookEnvelope),
            2 => Some(Self::IslandControl),
            3 => Some(Self::FullSnapshot),
            _ => None,
        }
    }
}

/// Content-free Link frame header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkFrameHeader {
    /// Message kind.
    pub message_kind: LinkMessageKind,
    /// Monotonic caller request identifier.
    pub request_id: u64,
    /// Payload length already validated before payload parsing.
    pub payload_length: u32,
}

impl LinkFrameHeader {
    /// Encode the header in little-endian local frame format.
    pub fn encode(self) -> [u8; FRAME_HEADER_BYTES] {
        let mut output = [0_u8; FRAME_HEADER_BYTES];
        output[0..4].copy_from_slice(&FRAME_MAGIC);
        write_u16(&mut output, 4, LINK_PROTOCOL_MAJOR);
        write_u16(&mut output, 6, LINK_PROTOCOL_MINOR);
        write_u16(&mut output, 8, self.message_kind.wire_value());
        write_u16(&mut output, 10, 0);
        write_u64(&mut output, 12, self.request_id);
        write_u32(&mut output, 20, self.payload_length);
        write_u32(&mut output, 24, 0);
        output
    }

    /// Decode and validate a content-free frame header before payload parsing.
    pub fn decode(input: &[u8]) -> Result<Self, LinkFrameError> {
        if input.len() != FRAME_HEADER_BYTES {
            return Err(LinkFrameError::InvalidHeaderLength);
        }
        if input[0..4] != FRAME_MAGIC {
            return Err(LinkFrameError::BadMagic);
        }
        if read_u16(input, 4) != LINK_PROTOCOL_MAJOR {
            return Err(LinkFrameError::UnsupportedVersion);
        }
        let Some(message_kind) = LinkMessageKind::from_wire(read_u16(input, 8)) else {
            return Err(LinkFrameError::UnknownMessageKind);
        };
        let payload_length = read_u32(input, 20);
        if payload_length as usize > message_kind.max_payload_bytes() {
            return Err(LinkFrameError::PayloadTooLarge);
        }
        Ok(Self {
            message_kind,
            request_id: read_u64(input, 12),
            payload_length,
        })
    }
}

/// Safe frame rejection categories. No variant contains payload bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkFrameError {
    /// Header length is not the fixed frame header size.
    InvalidHeaderLength,
    /// Magic bytes do not match Pulse Link.
    BadMagic,
    /// Protocol major version is unsupported.
    UnsupportedVersion,
    /// Message kind is unknown.
    UnknownMessageKind,
    /// Payload length exceeds the cap for the message kind.
    PayloadTooLarge,
}

/// Process launch handoff plan for the first validated event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitialHandoffPlan {
    /// Safe fixed command-line arguments. No event payload is included.
    pub argv: [&'static str; 2],
    /// Whether the validated frame is sent through inherited standard input.
    pub inherited_handoff_stdin: bool,
    /// Environment additions. Must remain empty for initial event handoff.
    pub environment: Vec<(String, String)>,
    /// Temporary file name. Must remain absent for initial event handoff.
    pub temp_file_name: Option<String>,
    /// Validated content-free frame header carried through the inherited pipe.
    pub frame_header: LinkFrameHeader,
}

impl InitialHandoffPlan {
    /// Build a launch handoff plan after validating frame length caps.
    pub fn new(frame_header: LinkFrameHeader) -> Result<Self, LinkFrameError> {
        if frame_header.payload_length as usize > frame_header.message_kind.max_payload_bytes() {
            return Err(LinkFrameError::PayloadTooLarge);
        }
        Ok(Self {
            argv: ["--wake-if-needed", "--handoff-stdin"],
            inherited_handoff_stdin: true,
            environment: Vec::new(),
            temp_file_name: None,
            frame_header,
        })
    }
}

/// Spike C Pulse Link lifecycle states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkLifecycleState {
    /// No Link runtime is running for this user/logon-session.
    NotRunning,
    /// Link is acquiring ownership and opening local endpoints.
    Starting,
    /// Link can accept bounded input, but has no active task yet.
    Warm,
    /// Link has at least one active task and no Island subscriber.
    Active,
    /// Link has active state and an Island subscriber.
    IslandActive,
    /// Link keeps compact breadcrumbs while no Island subscriber is attached.
    DropMode,
    /// Link is waiting briefly after the last active task ends.
    GracePeriod,
    /// Link is writing a final checkpoint and closing local resources.
    CheckpointAndExit,
}

/// Content-free lifecycle events accepted by the W3 state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkLifecycleEvent {
    /// Shim or Island requested a Link wake.
    WakeRequested,
    /// The starting Link owns the instance and local endpoints are ready.
    RuntimeReady,
    /// A valid bounded event indicates at least one active task.
    ValidActiveTaskEvent,
    /// Island client attached to receive snapshots/deltas.
    IslandAttached,
    /// Island client detached or unsubscribed.
    IslandDetached,
    /// The last active task reached a terminal lifecycle.
    LastActiveTaskTerminal,
    /// The no-active-task grace period expired.
    GraceExpired,
    /// Final checkpoint and resource close completed.
    CheckpointComplete,
}

/// Pure Spike C lifecycle reducer for Pulse Link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkLifecycle {
    state: LinkLifecycleState,
}

impl LinkLifecycle {
    /// Create a lifecycle in the stopped state.
    pub const fn new() -> Self {
        Self {
            state: LinkLifecycleState::NotRunning,
        }
    }

    /// Current lifecycle state.
    pub const fn state(self) -> LinkLifecycleState {
        self.state
    }

    /// Whether Link is fully stopped after checkpoint/exit.
    pub const fn is_terminal(self) -> bool {
        matches!(self.state, LinkLifecycleState::NotRunning)
    }

    /// Apply one content-free lifecycle event.
    pub const fn apply(self, event: LinkLifecycleEvent) -> Self {
        Self {
            state: next_lifecycle_state(self.state, event),
        }
    }
}

impl Default for LinkLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

/// Fake Island control requests used by W3 synthetic protocol tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IslandControlRequest {
    /// Version handshake.
    Hello,
    /// Request a complete bounded snapshot.
    GetSnapshot,
    /// Subscribe to future compact deltas.
    Subscribe,
    /// Stop receiving compact deltas.
    Unsubscribe,
    /// Health check.
    Ping,
    /// Request Link wake without provider control.
    RequestLinkWake,
}

/// Result of a fake Island protocol step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IslandDelivery {
    /// A bounded Island protocol message is available.
    Message(IslandMessage),
    /// No delta is delivered because the Island has not subscribed.
    NotSubscribed {
        /// Delta revision that was not delivered.
        revision: u64,
    },
    /// A revision gap was detected; Island must request a full snapshot.
    NeedsFullSnapshot {
        /// Next expected revision.
        expected_revision: u64,
        /// Revision that arrived.
        received_revision: u64,
    },
}

/// In-memory fake Island session for W3 protocol proof before named pipes exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FakeIslandSession {
    last_revision: u64,
    subscribed: bool,
}

impl FakeIslandSession {
    /// Create a fake Island session at the current Link snapshot revision.
    pub const fn new(current_revision: u64) -> Self {
        Self {
            last_revision: current_revision,
            subscribed: false,
        }
    }

    /// Handle one fake Island control request.
    pub fn handle(&mut self, request: IslandControlRequest) -> IslandDelivery {
        match request {
            IslandControlRequest::Hello => IslandDelivery::Message(IslandMessage::HelloAck {
                version: PROTOCOL_VERSION,
            }),
            IslandControlRequest::GetSnapshot => {
                match FullSnapshot::new(self.last_revision, Vec::new()) {
                    Ok(snapshot) => IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)),
                    Err(_) => IslandDelivery::Message(IslandMessage::ProtocolError(
                        ProtocolErrorCategory::SnapshotTooLarge,
                    )),
                }
            }
            IslandControlRequest::Subscribe => {
                self.subscribed = true;
                IslandDelivery::Message(IslandMessage::LinkHealth {
                    status: LinkHealthStatus::Healthy,
                })
            }
            IslandControlRequest::Unsubscribe => {
                self.subscribed = false;
                IslandDelivery::Message(IslandMessage::LinkHealth {
                    status: LinkHealthStatus::Healthy,
                })
            }
            IslandControlRequest::Ping | IslandControlRequest::RequestLinkWake => {
                IslandDelivery::Message(IslandMessage::LinkHealth {
                    status: LinkHealthStatus::Healthy,
                })
            }
        }
    }

    /// Publish a compact delta by revision, or request full snapshot recovery on gap.
    pub fn publish_delta(&mut self, revision: u64) -> IslandDelivery {
        if !self.subscribed {
            return IslandDelivery::NotSubscribed { revision };
        }
        let expected_revision = self.last_revision.saturating_add(1);
        if revision != expected_revision {
            return IslandDelivery::NeedsFullSnapshot {
                expected_revision,
                received_revision: revision,
            };
        }
        self.last_revision = revision;
        match SnapshotDelta::new(revision, Vec::new(), Vec::new()) {
            Ok(delta) => IslandDelivery::Message(IslandMessage::SnapshotDelta(delta)),
            Err(_) => IslandDelivery::Message(IslandMessage::ProtocolError(
                ProtocolErrorCategory::SnapshotTooLarge,
            )),
        }
    }
}

const fn next_lifecycle_state(
    state: LinkLifecycleState,
    event: LinkLifecycleEvent,
) -> LinkLifecycleState {
    match (state, event) {
        (LinkLifecycleState::NotRunning, LinkLifecycleEvent::WakeRequested) => {
            LinkLifecycleState::Starting
        }
        (LinkLifecycleState::Starting, LinkLifecycleEvent::RuntimeReady) => {
            LinkLifecycleState::Warm
        }
        (
            LinkLifecycleState::Warm
            | LinkLifecycleState::Active
            | LinkLifecycleState::DropMode
            | LinkLifecycleState::GracePeriod,
            LinkLifecycleEvent::ValidActiveTaskEvent,
        ) => LinkLifecycleState::Active,
        (
            LinkLifecycleState::Warm | LinkLifecycleState::Active | LinkLifecycleState::DropMode,
            LinkLifecycleEvent::IslandAttached,
        ) => LinkLifecycleState::IslandActive,
        (LinkLifecycleState::IslandActive, LinkLifecycleEvent::IslandDetached) => {
            LinkLifecycleState::DropMode
        }
        (
            LinkLifecycleState::Warm
            | LinkLifecycleState::Active
            | LinkLifecycleState::IslandActive
            | LinkLifecycleState::DropMode,
            LinkLifecycleEvent::LastActiveTaskTerminal,
        ) => LinkLifecycleState::GracePeriod,
        (LinkLifecycleState::GracePeriod, LinkLifecycleEvent::GraceExpired) => {
            LinkLifecycleState::CheckpointAndExit
        }
        (LinkLifecycleState::CheckpointAndExit, LinkLifecycleEvent::CheckpointComplete) => {
            LinkLifecycleState::NotRunning
        }
        _ => state,
    }
}

fn write_u16(output: &mut [u8; FRAME_HEADER_BYTES], offset: usize, value: u16) {
    output[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(output: &mut [u8; FRAME_HEADER_BYTES], offset: usize, value: u32) {
    output[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(output: &mut [u8; FRAME_HEADER_BYTES], offset: usize, value: u64) {
    output[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u16(input: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([input[offset], input[offset + 1]])
}

fn read_u32(input: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
    ])
}

fn read_u64(input: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
        input[offset + 4],
        input[offset + 5],
        input[offset + 6],
        input[offset + 7],
    ])
}
