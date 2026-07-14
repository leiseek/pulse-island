//! Fake Island client for W3 Link protocol spike.
#![deny(missing_docs)]

use pulse_link_core::{FakeIslandSession, IslandControlRequest, IslandDelivery};
use pulse_protocol::IslandMessage;
use pulse_win32::LinkLocalObjectNames;
#[cfg(target_env = "msvc")]
use pulse_win32_link::LinkTransportNativeBackendError;
use pulse_win32_link::{LinkTransportNativeApi, RawLinkHandle};

/// Summary of the fake Island attach flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttachReport {
    /// Whether Hello returned a protocol acknowledgement.
    pub hello_acknowledged: bool,
    /// Full snapshot revision returned during attach.
    pub snapshot_revision: u64,
    /// Whether Subscribe returned healthy Link state.
    pub subscribed: bool,
}

/// Report for connecting the fake Island client to the Island pipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PipeConnectReport {
    /// Whether the Island pipe connection opened.
    pub connected: bool,
    /// Content-free native handle value for tests/diagnostics.
    pub handle_value: Option<isize>,
}

/// Content-free report for an OS-backed fake Island protocol loop.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowsSysIslandProtocolLoopReport {
    /// Number of request/response byte round trips completed on the Island pipe.
    pub request_response_round_trips: u32,
    /// Whether Hello returned a protocol acknowledgement.
    pub hello_acknowledged: bool,
    /// Full snapshot revision returned during attach.
    pub snapshot_revision: u64,
    /// Whether Subscribe returned healthy Link state.
    pub subscribed: bool,
    /// Accepted delta revision, if the next delta was delivered in order.
    pub delta_accepted_revision: Option<u64>,
    /// Expected revision for the synthetic gap response.
    pub gap_expected_revision: Option<u64>,
    /// Received out-of-order revision for the synthetic gap response.
    pub gap_received_revision: Option<u64>,
    /// Number of native transport handles retained after cleanup.
    pub handles_remaining: u32,
}

/// Result of receiving a synthetic snapshot delta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaReceipt {
    /// Delta revision was accepted in order.
    Accepted {
        /// Accepted revision.
        revision: u64,
    },
    /// Delta revision was not delivered because the client is not subscribed.
    NotSubscribed {
        /// Undelivered revision.
        revision: u64,
    },
    /// A revision gap requires requesting a fresh full snapshot.
    NeedsFullSnapshot {
        /// Next expected revision.
        expected_revision: u64,
        /// Received out-of-order revision.
        received_revision: u64,
    },
    /// The protocol returned an unexpected safe message.
    UnexpectedMessage,
}

/// Error returned when starting the fake Island pipe message loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipeMessageLoopStartError {
    /// The native Island pipe client seam has not connected yet.
    PipeNotConnected,
}

/// Fake Island client that speaks the pure W3 session protocol.
#[derive(Clone, Debug)]
pub struct SpikeIslandClient {
    session: FakeIslandSession,
}

/// Pipe-backed fake Island message loop over the pure W3 session protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IslandPipeMessageLoop {
    session: FakeIslandSession,
}

/// Fake Island client wrapper with a native Island pipe connection seam.
#[derive(Clone, Debug)]
pub struct SpikeIslandPipeClient<A> {
    names: LinkLocalObjectNames,
    api: A,
    handle: Option<RawLinkHandle>,
}

impl<A> SpikeIslandPipeClient<A>
where
    A: LinkTransportNativeApi,
{
    /// Create a fake Island pipe client for one scoped Link namespace.
    pub fn new(names: LinkLocalObjectNames, api: A) -> Self {
        Self {
            names,
            api,
            handle: None,
        }
    }

    /// Borrow the native API adapter for diagnostics/tests.
    pub const fn api(&self) -> &A {
        &self.api
    }

    /// Connect to the scoped Island pipe before running the fake protocol session.
    pub fn connect_pipe(&mut self) -> PipeConnectReport {
        self.handle = self.api.connect_island_client(&self.names);
        PipeConnectReport {
            connected: self.handle.is_some(),
            handle_value: self.handle.map(RawLinkHandle::value),
        }
    }

    /// Run the existing fake session attach flow after pipe connection setup.
    pub fn attach_fake_session(&mut self, current_revision: u64) -> AttachReport {
        let mut client = SpikeIslandClient::connect(current_revision);
        client.attach()
    }

    /// Start a fake pipe message loop after the Island pipe connection exists.
    pub fn start_message_loop(
        &mut self,
        current_revision: u64,
    ) -> Result<IslandPipeMessageLoop, PipeMessageLoopStartError> {
        if self.handle.is_none() {
            return Err(PipeMessageLoopStartError::PipeNotConnected);
        }
        Ok(IslandPipeMessageLoop {
            session: FakeIslandSession::new(current_revision),
        })
    }
}

impl SpikeIslandClient {
    /// Connect to a fake Link session at the provided current revision.
    pub const fn connect(current_revision: u64) -> Self {
        Self {
            session: FakeIslandSession::new(current_revision),
        }
    }

    /// Execute Hello, GetSnapshot, and Subscribe.
    pub fn attach(&mut self) -> AttachReport {
        let hello_acknowledged = matches!(
            self.session.handle(IslandControlRequest::Hello),
            IslandDelivery::Message(IslandMessage::HelloAck { .. })
        );
        let snapshot_revision = match self.session.handle(IslandControlRequest::GetSnapshot) {
            IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)) => snapshot.revision,
            _ => 0,
        };
        let subscribed = matches!(
            self.session.handle(IslandControlRequest::Subscribe),
            IslandDelivery::Message(IslandMessage::LinkHealth { .. })
        );
        AttachReport {
            hello_acknowledged,
            snapshot_revision,
            subscribed,
        }
    }

    /// Receive a synthetic delta by revision.
    pub fn receive_delta(&mut self, revision: u64) -> DeltaReceipt {
        match self.session.publish_delta(revision) {
            IslandDelivery::Message(IslandMessage::SnapshotDelta(delta)) => {
                DeltaReceipt::Accepted {
                    revision: delta.revision,
                }
            }
            IslandDelivery::NotSubscribed { revision } => DeltaReceipt::NotSubscribed { revision },
            IslandDelivery::NeedsFullSnapshot {
                expected_revision,
                received_revision,
            } => DeltaReceipt::NeedsFullSnapshot {
                expected_revision,
                received_revision,
            },
            IslandDelivery::Message(_) => DeltaReceipt::UnexpectedMessage,
        }
    }

    /// Mutable access to the raw fake session for protocol edge tests.
    pub fn raw_session_mut(&mut self) -> &mut FakeIslandSession {
        &mut self.session
    }
}

impl IslandPipeMessageLoop {
    /// Handle one fake Island control request received through the pipe seam.
    pub fn handle_control(&mut self, request: IslandControlRequest) -> IslandDelivery {
        self.session.handle(request)
    }

    /// Publish one compact synthetic delta through the pipe seam.
    pub fn publish_delta(&mut self, revision: u64) -> IslandDelivery {
        self.session.publish_delta(revision)
    }
}

/// Run the MSVC `windows-sys` Island pipe loop against the fake Island protocol sequence.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_island_protocol_loop_harness(
    names: LinkLocalObjectNames,
    current_revision: u64,
) -> Result<WindowsSysIslandProtocolLoopReport, LinkTransportNativeBackendError> {
    let mut session = FakeIslandSession::new(current_revision);
    let hello = session.handle(IslandControlRequest::Hello);
    let snapshot = session.handle(IslandControlRequest::GetSnapshot);
    let subscribe = session.handle(IslandControlRequest::Subscribe);
    let delta = session.publish_delta(current_revision.saturating_add(1));
    let gap_revision = current_revision.saturating_add(3);
    let gap = session.publish_delta(gap_revision);

    let responses = [
        island_delivery_response_bytes(&hello),
        island_delivery_response_bytes(&snapshot),
        island_delivery_response_bytes(&subscribe),
        island_delivery_response_bytes(&delta),
        island_delivery_response_bytes(&gap),
    ];
    let Some(responses) = collect_response_bytes(&responses) else {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "EncodeIslandProtocolResponse",
        ));
    };
    let requests = island_protocol_request_bytes(current_revision);
    let request_slices = requests
        .iter()
        .map(|request| request.as_slice())
        .collect::<Vec<_>>();
    let response_slices = responses
        .iter()
        .map(|response| response.as_slice())
        .collect::<Vec<_>>();
    let os_report = pulse_win32_link::run_windows_sys_island_request_response_loop_harness(
        names,
        &request_slices,
        &response_slices,
    )?;

    Ok(WindowsSysIslandProtocolLoopReport {
        request_response_round_trips: os_report.round_trip_count,
        hello_acknowledged: matches!(
            hello,
            IslandDelivery::Message(IslandMessage::HelloAck { .. })
        ),
        snapshot_revision: snapshot_revision(&snapshot),
        subscribed: matches!(
            subscribe,
            IslandDelivery::Message(IslandMessage::LinkHealth { .. })
        ),
        delta_accepted_revision: accepted_delta_revision(&delta),
        gap_expected_revision: gap_revisions(&gap).map(|(expected, _)| expected),
        gap_received_revision: gap_revisions(&gap).map(|(_, received)| received),
        handles_remaining: os_report.shutdown.handles_remaining,
    })
}

#[cfg(target_env = "msvc")]
fn island_protocol_request_bytes(current_revision: u64) -> [Vec<u8>; 5] {
    [
        vec![1],
        vec![2],
        vec![3],
        revision_request_bytes(4, current_revision.saturating_add(1)),
        revision_request_bytes(4, current_revision.saturating_add(3)),
    ]
}

#[cfg(target_env = "msvc")]
fn revision_request_bytes(kind: u8, revision: u64) -> Vec<u8> {
    let mut bytes = vec![kind];
    bytes.extend_from_slice(&revision.to_le_bytes());
    bytes
}

#[cfg(target_env = "msvc")]
fn collect_response_bytes(responses: &[Option<Vec<u8>>; 5]) -> Option<[Vec<u8>; 5]> {
    Some([
        responses[0].clone()?,
        responses[1].clone()?,
        responses[2].clone()?,
        responses[3].clone()?,
        responses[4].clone()?,
    ])
}

#[cfg(target_env = "msvc")]
fn island_delivery_response_bytes(delivery: &IslandDelivery) -> Option<Vec<u8>> {
    match delivery {
        IslandDelivery::Message(IslandMessage::HelloAck { version }) => {
            let mut bytes = vec![0x11];
            bytes.extend_from_slice(&version.to_le_bytes());
            Some(bytes)
        }
        IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)) => {
            let mut bytes = vec![0x12];
            bytes.extend_from_slice(&snapshot.revision.to_le_bytes());
            Some(bytes)
        }
        IslandDelivery::Message(IslandMessage::LinkHealth { .. }) => Some(vec![0x13]),
        IslandDelivery::Message(IslandMessage::SnapshotDelta(delta)) => {
            let mut bytes = vec![0x14];
            bytes.extend_from_slice(&delta.revision.to_le_bytes());
            Some(bytes)
        }
        IslandDelivery::NeedsFullSnapshot {
            expected_revision,
            received_revision,
        } => {
            let mut bytes = vec![0x15];
            bytes.extend_from_slice(&expected_revision.to_le_bytes());
            bytes.extend_from_slice(&received_revision.to_le_bytes());
            Some(bytes)
        }
        IslandDelivery::NotSubscribed { .. } | IslandDelivery::Message(_) => None,
    }
}

#[cfg(target_env = "msvc")]
fn snapshot_revision(delivery: &IslandDelivery) -> u64 {
    match delivery {
        IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)) => snapshot.revision,
        _ => 0,
    }
}

#[cfg(target_env = "msvc")]
fn accepted_delta_revision(delivery: &IslandDelivery) -> Option<u64> {
    match delivery {
        IslandDelivery::Message(IslandMessage::SnapshotDelta(delta)) => Some(delta.revision),
        _ => None,
    }
}

#[cfg(target_env = "msvc")]
fn gap_revisions(delivery: &IslandDelivery) -> Option<(u64, u64)> {
    match delivery {
        IslandDelivery::NeedsFullSnapshot {
            expected_revision,
            received_revision,
        } => Some((*expected_revision, *received_revision)),
        _ => None,
    }
}
