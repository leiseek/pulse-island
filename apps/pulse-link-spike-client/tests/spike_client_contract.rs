//! W3 fake Island spike client contract tests.

use pulse_link_core::{IslandControlRequest, IslandDelivery};
#[cfg(target_env = "msvc")]
use pulse_link_spike_client::{
    run_windows_sys_island_protocol_loop_harness, WindowsSysIslandProtocolLoopReport,
};
use pulse_link_spike_client::{
    DeltaReceipt, PipeMessageLoopStartError, SpikeIslandClient, SpikeIslandPipeClient,
};
use pulse_protocol::{IslandMessage, LinkHealthStatus, PROTOCOL_VERSION};
use pulse_win32::LinkLocalObjectNames;
use pulse_win32_link::{InheritedHandoffPipe, LinkTransportNativeApi, RawLinkHandle};

#[test]
fn spike_client_attach_performs_hello_snapshot_and_subscribe() {
    let mut client = SpikeIslandClient::connect(4);

    let report = client.attach();

    assert!(report.hello_acknowledged);
    assert_eq!(report.snapshot_revision, 4);
    assert!(report.subscribed);
}

#[test]
fn spike_client_accepts_next_delta_and_flags_revision_gap() {
    let mut client = SpikeIslandClient::connect(10);
    client.attach();

    assert_eq!(
        client.receive_delta(11),
        DeltaReceipt::Accepted { revision: 11 }
    );
    assert_eq!(
        client.receive_delta(13),
        DeltaReceipt::NeedsFullSnapshot {
            expected_revision: 12,
            received_revision: 13,
        }
    );
}

#[test]
fn spike_client_reports_unsubscribed_delivery_before_attach() {
    let mut client = SpikeIslandClient::connect(2);

    assert_eq!(
        client.raw_session_mut().publish_delta(3),
        IslandDelivery::NotSubscribed { revision: 3 }
    );
}

#[test]
fn spike_pipe_client_connects_to_island_pipe_before_attach(
) -> Result<(), Box<dyn std::error::Error>> {
    let names = LinkLocalObjectNames::derive("install", "sid", "session", 1);
    let mut pipe_client = SpikeIslandPipeClient::new(names.clone(), FakePipeApi::default());

    let connect = pipe_client.connect_pipe();
    let attach = pipe_client.attach_fake_session(7);

    assert!(connect.connected);
    assert_eq!(connect.handle_value, Some(1));
    assert_eq!(pipe_client.api().connected_names, vec![names.island_pipe]);
    assert!(attach.hello_acknowledged);
    assert_eq!(attach.snapshot_revision, 7);
    assert!(attach.subscribed);
    Ok(())
}

#[test]
fn spike_pipe_message_loop_requires_pipe_connection() {
    let names = LinkLocalObjectNames::derive("install", "sid", "session", 1);
    let mut pipe_client = SpikeIslandPipeClient::new(names.clone(), FakePipeApi::default());

    assert_eq!(
        pipe_client.start_message_loop(3),
        Err(PipeMessageLoopStartError::PipeNotConnected)
    );
}

#[test]
fn spike_pipe_message_loop_handles_attach_delta_and_gap() -> Result<(), Box<dyn std::error::Error>>
{
    let names = LinkLocalObjectNames::derive("install", "sid", "session", 1);
    let mut pipe_client = SpikeIslandPipeClient::new(names.clone(), FakePipeApi::default());

    let connect = pipe_client.connect_pipe();
    let mut message_loop = match pipe_client.start_message_loop(3) {
        Ok(message_loop) => message_loop,
        Err(error) => return Err(format!("fake loop startup failed: {error:?}").into()),
    };

    assert!(connect.connected);
    assert_eq!(
        message_loop.handle_control(IslandControlRequest::Hello),
        IslandDelivery::Message(IslandMessage::HelloAck {
            version: PROTOCOL_VERSION,
        })
    );
    assert!(matches!(
        message_loop.handle_control(IslandControlRequest::GetSnapshot),
        IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)) if snapshot.revision == 3
    ));
    assert_eq!(
        message_loop.handle_control(IslandControlRequest::Subscribe),
        IslandDelivery::Message(IslandMessage::LinkHealth {
            status: LinkHealthStatus::Healthy,
        })
    );
    assert!(matches!(
        message_loop.publish_delta(4),
        IslandDelivery::Message(IslandMessage::SnapshotDelta(delta)) if delta.revision == 4
    ));
    assert_eq!(
        message_loop.publish_delta(6),
        IslandDelivery::NeedsFullSnapshot {
            expected_revision: 5,
            received_revision: 6,
        }
    );
    assert_eq!(pipe_client.api().connected_names, vec![names.island_pipe]);
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_island_pipe_protocol_loop_handles_attach_delta_and_gap(
) -> Result<(), Box<dyn std::error::Error>> {
    let report = run_windows_sys_island_protocol_loop_harness(unique_test_names(), 3)
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert_eq!(
        report,
        WindowsSysIslandProtocolLoopReport {
            request_response_round_trips: 5,
            hello_acknowledged: true,
            snapshot_revision: 3,
            subscribed: true,
            delta_accepted_revision: Some(4),
            gap_expected_revision: Some(5),
            gap_received_revision: Some(6),
            handles_remaining: 0,
        }
    );
    Ok(())
}

#[cfg(target_env = "msvc")]
fn unique_test_names() -> LinkLocalObjectNames {
    let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    LinkLocalObjectNames::derive(
        &format!("pulse-island-client-{}-{timestamp}", std::process::id()),
        "sid",
        "session",
        1,
    )
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FakePipeApi {
    next_handle: isize,
    connected_names: Vec<String>,
}

impl FakePipeApi {
    fn next_handle(&mut self) -> Option<RawLinkHandle> {
        self.next_handle += 1;
        RawLinkHandle::new(self.next_handle)
    }
}

impl LinkTransportNativeApi for FakePipeApi {
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
        let read = self.next_handle()?;
        let write = self.next_handle()?;
        Some(InheritedHandoffPipe { read, write })
    }

    fn connect_island_client(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.connected_names.push(names.island_pipe.clone());
        self.next_handle()
    }

    fn close_handle(&mut self, _handle: RawLinkHandle) -> bool {
        true
    }
}
