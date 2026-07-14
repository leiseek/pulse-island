//! W3 native Link transport boundary contract tests.

use pulse_win32::LinkLocalObjectNames;
use pulse_win32_link::{
    InheritedHandoffPipe, LinkTransportCommand, LinkTransportNativeApi, LinkTransportNativeBackend,
    LinkTransportNativeBackendError, LinkTransportPreflightError, LinkTransportPreflightSink,
    LinkTransportShutdownReport, LinkTransportState, RawLinkHandle,
};

#[cfg(target_env = "msvc")]
use pulse_link_core::{LinkFrameHeader, LinkMessageKind, FRAME_HEADER_BYTES};
#[cfg(target_env = "msvc")]
use pulse_win32_link::{
    run_windows_sys_ingress_frame_ack_harness, run_windows_sys_os_transport_harness,
};

#[test]
fn link_transport_preflight_accepts_mutex_then_pipe_servers_then_handoff() {
    let names = test_names();
    let mut sink = LinkTransportPreflightSink::default();

    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateMutex(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateIngressPipe(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateIslandPipe(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateInheritedHandoffPipe),
        Ok(())
    );

    assert_eq!(
        sink.state(),
        LinkTransportState {
            mutex_created: true,
            ingress_pipe_created: true,
            island_pipe_created: true,
            handoff_pipe_created: true,
            island_client_connected: false,
        }
    );
}

#[test]
fn link_transport_rejects_pipe_servers_before_mutex_ownership() {
    let names = test_names();
    let mut sink = LinkTransportPreflightSink::default();

    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateIngressPipe(names.clone())),
        Err(LinkTransportPreflightError::MutexMissing)
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateIslandPipe(names)),
        Err(LinkTransportPreflightError::MutexMissing)
    );
    assert_eq!(sink.state(), LinkTransportState::default());
}

#[test]
fn link_transport_rejects_duplicate_objects_without_advancing_state() {
    let names = test_names();
    let mut sink = LinkTransportPreflightSink::default();

    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateMutex(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateMutex(names)),
        Err(LinkTransportPreflightError::MutexAlreadyCreated)
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateInheritedHandoffPipe),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateInheritedHandoffPipe),
        Err(LinkTransportPreflightError::HandoffAlreadyCreated)
    );

    assert_eq!(
        sink.state(),
        LinkTransportState {
            mutex_created: true,
            ingress_pipe_created: false,
            island_pipe_created: false,
            handoff_pipe_created: true,
            island_client_connected: false,
        }
    );
}

#[test]
fn raw_link_handle_rejects_null_handles() {
    assert_eq!(RawLinkHandle::new(0), None);
    assert_eq!(RawLinkHandle::new(42).map(RawLinkHandle::value), Some(42));
}

#[test]
fn native_backend_applies_preflighted_transport_commands(
) -> Result<(), LinkTransportNativeBackendError> {
    let names = test_names();
    let mut backend = LinkTransportNativeBackend::new(FakeLinkTransportApi::default());

    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateInheritedHandoffPipe)?;
    backend.apply_command(LinkTransportCommand::ConnectIslandClient(names.clone()))?;

    let state = backend.state();
    assert!(state.mutex_handle.is_some());
    assert!(state.ingress_pipe_handle.is_some());
    assert!(state.island_pipe_handle.is_some());
    assert!(state.handoff_pipe.is_some());
    assert!(state.island_client_handle.is_some());
    assert_eq!(backend.api().created_handles, 6);
    Ok(())
}

#[test]
fn native_backend_does_not_advance_after_native_failure() {
    let names = test_names();
    let mut backend = LinkTransportNativeBackend::new(FakeLinkTransportApi {
        fail_ingress_pipe: true,
        ..FakeLinkTransportApi::default()
    });

    assert_eq!(
        backend.apply_command(LinkTransportCommand::CreateMutex(names.clone())),
        Ok(())
    );
    assert_eq!(
        backend.apply_command(LinkTransportCommand::CreateIngressPipe(names)),
        Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CreateIngressPipe"
        ))
    );

    assert!(backend.state().mutex_handle.is_some());
    assert!(backend.state().ingress_pipe_handle.is_none());
}

#[test]
fn native_backend_closes_all_owned_handles_and_clears_state(
) -> Result<(), LinkTransportNativeBackendError> {
    let names = test_names();
    let mut backend = LinkTransportNativeBackend::new(FakeLinkTransportApi::default());

    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateInheritedHandoffPipe)?;

    let report = backend.close_all();

    assert_eq!(
        report,
        LinkTransportShutdownReport {
            close_attempts: 5,
            closed_handles: 5,
            failed_closes: 0,
            handles_remaining: 0,
        }
    );
    assert_eq!(backend.api().closed_handles, vec![5, 4, 3, 2, 1]);
    assert_eq!(backend.state(), &Default::default());
    Ok(())
}

#[test]
fn native_backend_retains_failed_close_handles_for_retry(
) -> Result<(), LinkTransportNativeBackendError> {
    let names = test_names();
    let mut backend = LinkTransportNativeBackend::new(FakeLinkTransportApi {
        fail_close_handle: Some(3),
        ..FakeLinkTransportApi::default()
    });

    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names.clone()))?;

    let report = backend.close_all();

    assert_eq!(
        report,
        LinkTransportShutdownReport {
            close_attempts: 3,
            closed_handles: 2,
            failed_closes: 1,
            handles_remaining: 1,
        }
    );
    assert_eq!(
        backend.state().island_pipe_handle.map(RawLinkHandle::value),
        Some(3)
    );
    assert!(backend.state().mutex_handle.is_none());
    assert!(backend.state().ingress_pipe_handle.is_none());
    Ok(())
}

#[test]
fn link_transport_connects_island_client_only_after_island_pipe_exists() {
    let names = test_names();
    let mut sink = LinkTransportPreflightSink::default();

    assert_eq!(
        sink.validate_command(LinkTransportCommand::ConnectIslandClient(names.clone())),
        Err(LinkTransportPreflightError::IslandPipeMissing)
    );

    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateMutex(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::CreateIslandPipe(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::ConnectIslandClient(names.clone())),
        Ok(())
    );
    assert_eq!(
        sink.validate_command(LinkTransportCommand::ConnectIslandClient(names)),
        Err(LinkTransportPreflightError::IslandClientAlreadyConnected)
    );
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_os_transport_harness_creates_pipe_mutex_handoff_and_cleans_up(
) -> Result<(), Box<dyn std::error::Error>> {
    let report = run_windows_sys_os_transport_harness(unique_test_names())
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert!(report.mutex_created);
    assert!(report.ingress_pipe_created);
    assert!(report.island_pipe_created);
    assert!(report.handoff_pipe_created);
    assert!(report.island_client_connected);
    assert_eq!(report.shutdown.close_attempts, 6);
    assert_eq!(report.shutdown.closed_handles, 6);
    assert_eq!(report.shutdown.failed_closes, 0);
    assert_eq!(report.shutdown.handles_remaining, 0);
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_ingress_pipe_round_trips_frame_header_and_ack(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 77,
        payload_length: 0,
    }
    .encode();

    let report = run_windows_sys_ingress_frame_ack_harness(unique_test_names(), &frame)
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert_eq!(report.frame_bytes_written, FRAME_HEADER_BYTES as u32);
    assert_eq!(report.frame_bytes_read, FRAME_HEADER_BYTES as u32);
    assert_eq!(report.ack_bytes_written, 1);
    assert_eq!(report.ack_bytes_read, 1);
    assert!(report.frame_round_tripped);
    assert!(report.ack_round_tripped);
    assert_eq!(report.shutdown.close_attempts, 3);
    assert_eq!(report.shutdown.closed_handles, 3);
    assert_eq!(report.shutdown.failed_closes, 0);
    assert_eq!(report.shutdown.handles_remaining, 0);
    Ok(())
}

fn test_names() -> LinkLocalObjectNames {
    LinkLocalObjectNames::derive("install", "sid", "session", 1)
}

#[cfg(target_env = "msvc")]
fn unique_test_names() -> LinkLocalObjectNames {
    let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    LinkLocalObjectNames::derive(
        &format!("install-{}-{timestamp}", std::process::id()),
        "sid",
        "session",
        1,
    )
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FakeLinkTransportApi {
    created_handles: isize,
    fail_ingress_pipe: bool,
    fail_close_handle: Option<isize>,
    closed_handles: Vec<isize>,
}

impl FakeLinkTransportApi {
    fn next_handle(&mut self) -> Option<RawLinkHandle> {
        self.created_handles += 1;
        RawLinkHandle::new(self.created_handles)
    }
}

impl LinkTransportNativeApi for FakeLinkTransportApi {
    fn create_mutex(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn create_ingress_pipe(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        if self.fail_ingress_pipe {
            None
        } else {
            self.next_handle()
        }
    }

    fn create_island_pipe(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn create_inherited_handoff_pipe(&mut self) -> Option<InheritedHandoffPipe> {
        let read = self.next_handle()?;
        let write = self.next_handle()?;
        Some(InheritedHandoffPipe { read, write })
    }

    fn connect_island_client(&mut self, _names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        self.next_handle()
    }

    fn close_handle(&mut self, handle: RawLinkHandle) -> bool {
        if self.fail_close_handle == Some(handle.value()) {
            return false;
        }
        self.closed_handles.push(handle.value());
        true
    }
}
