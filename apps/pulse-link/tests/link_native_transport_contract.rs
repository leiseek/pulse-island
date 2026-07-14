//! W3 Link native transport startup contract tests.

use pulse_link::{
    prepare_native_link_transport, LinkNativeStartupReport, LinkNativeTransportRuntime,
};
#[cfg(target_env = "msvc")]
use pulse_link::{
    run_windows_sys_c0_c9_harness, run_windows_sys_grace_exit_residue_harness,
    run_windows_sys_ingress_reducer_ack_harness, run_windows_sys_ingress_reducer_ack_loop_harness,
    WindowsSysGraceExitResidueReport, WindowsSysIngressReducerAckLoopReport,
    WindowsSysIngressReducerAckReport, WindowsSysSpikeC0C9HarnessReport,
};
#[cfg(target_env = "msvc")]
use pulse_link_core::{LinkFrameHeader, LinkLifecycleState, LinkMessageKind};
use pulse_win32::LinkLocalObjectNames;
use pulse_win32_link::{
    InheritedHandoffPipe, LinkTransportNativeApi, LinkTransportShutdownReport, RawLinkHandle,
};

#[test]
fn link_startup_creates_mutex_ingress_and_island_pipe_servers() {
    assert_eq!(
        prepare_native_link_transport(test_names(), FakeLinkTransportApi::default()),
        Ok(LinkNativeStartupReport {
            mutex_created: true,
            ingress_pipe_created: true,
            island_pipe_created: true,
            handoff_pipe_created: false,
        })
    );
}

#[test]
fn link_startup_does_not_report_ready_after_ingress_pipe_failure() {
    let result = prepare_native_link_transport(
        test_names(),
        FakeLinkTransportApi {
            fail_ingress_pipe: true,
            ..FakeLinkTransportApi::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn link_native_runtime_shutdown_closes_transport_handles_after_checkpoint(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime =
        LinkNativeTransportRuntime::start(test_names(), FakeLinkTransportApi::default())
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    let report = runtime.shutdown_after_checkpoint();

    assert_eq!(
        report,
        LinkTransportShutdownReport {
            close_attempts: 3,
            closed_handles: 3,
            failed_closes: 0,
            handles_remaining: 0,
        }
    );
    assert_eq!(runtime.api().closed_handles, vec![3, 2, 1]);
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_ingress_frame_ack_drives_reducer_and_checkpoint(
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 91,
        payload_length: 0,
    };

    let report = run_windows_sys_ingress_reducer_ack_harness(unique_test_names(), frame.encode())
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert_eq!(
        report,
        WindowsSysIngressReducerAckReport {
            frame_accepted: true,
            reducer_checkpoint_written: true,
            lifecycle_state: LinkLifecycleState::Active,
            active_tasks: 1,
            recent_terminal_tasks: 0,
            ack_round_tripped: true,
            handles_remaining: 0,
        }
    );
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_ingress_frame_ack_loop_rejects_bad_frame_without_stopping_reducer(
) -> Result<(), Box<dyn std::error::Error>> {
    let first = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 101,
        payload_length: 0,
    }
    .encode();
    let mut bad = first;
    bad[0] = b'X';
    let second = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 102,
        payload_length: 0,
    }
    .encode();

    let frames = [first, bad, second];
    let report = run_windows_sys_ingress_reducer_ack_loop_harness(unique_test_names(), &frames)
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert_eq!(
        report,
        WindowsSysIngressReducerAckLoopReport {
            frames_seen: 3,
            frames_accepted: 2,
            frames_rejected: 1,
            reducer_checkpoint_writes: 2,
            lifecycle_state: LinkLifecycleState::Active,
            active_tasks: 2,
            recent_terminal_tasks: 0,
            ack_round_trips: 3,
            handles_remaining: 0,
        }
    );
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_grace_exit_closes_transport_and_leaves_no_child_residue(
) -> Result<(), Box<dyn std::error::Error>> {
    let report = run_windows_sys_grace_exit_residue_harness(unique_test_names())
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert_eq!(
        report,
        WindowsSysGraceExitResidueReport {
            final_checkpoint_written: true,
            stopped_link: true,
            lifecycle_state: LinkLifecycleState::NotRunning,
            active_tasks: 0,
            recent_terminal_tasks: 1,
            shutdown_close_attempts: 3,
            shutdown_closed_handles: 3,
            transport_handles_remaining: 0,
            child_process_started: true,
            child_exit_observed: true,
            child_processes_remaining: 0,
        }
    );
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_c0_c9_harness_covers_all_spike_c_scenarios() -> Result<(), Box<dyn std::error::Error>>
{
    let report = run_windows_sys_c0_c9_harness(unique_test_names())
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?;

    assert_eq!(
        report,
        WindowsSysSpikeC0C9HarnessReport {
            scenario_count: 10,
            os_transport_ready: true,
            c0_existing_link_delivery: true,
            c1_first_hook_handoff: true,
            c2_parallel_race_single_link: true,
            c3_link_unavailable_fail_open: true,
            c4_malformed_rejected_before_mutation: true,
            c5_drop_mode_breadcrumb_bounded: true,
            c6_island_attach_detach_reattach: true,
            c7_restart_recovery_degraded: true,
            c8_grace_exit_residue_clean: true,
            c9_event_storm_bounded: true,
            provider_affected: false,
            raw_payload_persisted: false,
            handles_remaining: 0,
        }
    );
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
        &format!("pulse-link-{}-{timestamp}", std::process::id()),
        "sid",
        "session",
        1,
    )
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FakeLinkTransportApi {
    created_handles: isize,
    fail_ingress_pipe: bool,
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
        self.closed_handles.push(handle.value());
        true
    }
}
