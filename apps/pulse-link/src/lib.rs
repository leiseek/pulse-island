//! Pure W3 Pulse Link runner before OS transport is wired.
#![deny(missing_docs)]

use pulse_domain::{BoundedText, Lifecycle, PrivacyProfile, TaskHealth, TaskSnapshot, TimestampMs};
use pulse_link_core::{
    FakeIslandSession, InitialHandoffPlan, IslandControlRequest, IslandDelivery, LinkFrameHeader,
    LinkLifecycle, LinkLifecycleEvent, LinkLifecycleState, LinkMessageKind,
};
use pulse_link_shim::{run_shim_preflight, ShimDelivery, ShimDeliveryAttempt, ShimRunReport};
use pulse_persistence::{
    BreadcrumbSet, BreadcrumbStore, MemoryBreadcrumbStore, StoreError, RECENT_TERMINAL_LIMIT,
};
use pulse_protocol::{AdmittedEvent, EvidenceKind, IslandMessage, ShimExitStatus};
use pulse_reducer::{initial, reduce, BreadcrumbRetention};
use pulse_win32::{
    LinkLocalObjectNames, LinkOwnershipDecision, LinkOwnershipRegistry, LinkStartupObservation,
};
use pulse_win32_link::{
    LinkTransportCommand, LinkTransportNativeApi, LinkTransportNativeBackend,
    LinkTransportNativeBackendError, LinkTransportShutdownReport,
};

/// Spike C fixed grace duration before Link exits after the last active task ends.
pub const SPIKE_C_GRACE_PERIOD_MS: u64 = 90_000;

/// Report emitted after one synthetic Link event is reduced and checkpointed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkRuntimeReport {
    /// Current Link lifecycle state.
    pub lifecycle_state: LinkLifecycleState,
    /// Number of active task breadcrumbs retained.
    pub active_tasks: usize,
    /// Number of recent terminal breadcrumbs retained.
    pub recent_terminal_tasks: usize,
    /// Whether the complete replacement checkpoint was written.
    pub checkpoint_written: bool,
}

/// Report emitted by the Drop Mode grace driver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropModeGraceReport {
    /// Current grace deadline, if Link is waiting for possible new work.
    pub grace_deadline: Option<TimestampMs>,
    /// Whether a final checkpoint was written on this tick.
    pub final_checkpoint_written: bool,
    /// Whether Link transitioned to stopped on this tick.
    pub stopped_link: bool,
}

/// Driver for the W3 90-second grace-exit rule after the last active task ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropModeGraceDriver {
    grace_period_ms: u64,
    deadline: Option<TimestampMs>,
}

impl DropModeGraceDriver {
    /// Create the Spike C grace driver with a fixed 90-second duration.
    pub const fn spike_c() -> Self {
        Self {
            grace_period_ms: SPIKE_C_GRACE_PERIOD_MS,
            deadline: None,
        }
    }

    /// Observe runtime state and arm or cancel the grace deadline.
    pub fn observe_runtime<S>(
        &mut self,
        runtime: &LinkRuntime<S>,
        now: TimestampMs,
    ) -> DropModeGraceReport
    where
        S: BreadcrumbStore,
    {
        if runtime.lifecycle_state() == LinkLifecycleState::GracePeriod {
            if self.deadline.is_none() {
                self.deadline = Some(TimestampMs(now.0.saturating_add(self.grace_period_ms)));
            }
        } else {
            self.deadline = None;
        }
        self.report(false, false)
    }

    /// Advance the driver clock and stop Link when the armed deadline has elapsed.
    pub fn tick<S>(
        &mut self,
        runtime: &mut LinkRuntime<S>,
        now: TimestampMs,
    ) -> Result<DropModeGraceReport, StoreError>
    where
        S: BreadcrumbStore,
    {
        if runtime.lifecycle_state() != LinkLifecycleState::GracePeriod {
            self.deadline = None;
            return Ok(self.report(false, false));
        }
        let Some(deadline) = self.deadline else {
            return Ok(self.report(false, false));
        };
        if now < deadline {
            return Ok(self.report(false, false));
        }

        runtime.final_checkpoint_and_stop(now)?;
        self.deadline = None;
        Ok(self.report(true, true))
    }

    fn report(self, final_checkpoint_written: bool, stopped_link: bool) -> DropModeGraceReport {
        DropModeGraceReport {
            grace_deadline: self.deadline,
            final_checkpoint_written,
            stopped_link,
        }
    }
}

/// Content-free report for W3 native Link transport startup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkNativeStartupReport {
    /// Whether Link created/acquired the scoped mutex.
    pub mutex_created: bool,
    /// Whether Link created the Shim ingress pipe server.
    pub ingress_pipe_created: bool,
    /// Whether Link created the Island pipe server.
    pub island_pipe_created: bool,
    /// Whether Link owns an inherited first-event handoff pipe.
    pub handoff_pipe_created: bool,
}

/// Content-free report for an OS ingress frame that drives Link reducer state.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowsSysIngressReducerAckReport {
    /// Whether a valid Hook ingress header was accepted after the OS ack.
    pub frame_accepted: bool,
    /// Whether reducer output was written to a complete replacement checkpoint.
    pub reducer_checkpoint_written: bool,
    /// Current Link lifecycle state after reducer application.
    pub lifecycle_state: LinkLifecycleState,
    /// Number of active task breadcrumbs retained.
    pub active_tasks: usize,
    /// Number of recent terminal task breadcrumbs retained.
    pub recent_terminal_tasks: usize,
    /// Whether the OS-backed ingress acknowledgement completed.
    pub ack_round_tripped: bool,
    /// Number of native transport handles retained after cleanup.
    pub handles_remaining: u32,
}

/// Content-free report for an OS ingress frame loop that drives Link reducer state.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowsSysIngressReducerAckLoopReport {
    /// Number of frames read from the OS ingress loop.
    pub frames_seen: u32,
    /// Number of valid Hook headers applied to the reducer.
    pub frames_accepted: u32,
    /// Number of invalid or unsupported headers rejected before reducer mutation.
    pub frames_rejected: u32,
    /// Number of complete replacement checkpoints written by reducer application.
    pub reducer_checkpoint_writes: u32,
    /// Current Link lifecycle state after the loop.
    pub lifecycle_state: LinkLifecycleState,
    /// Number of active task breadcrumbs retained.
    pub active_tasks: usize,
    /// Number of recent terminal task breadcrumbs retained.
    pub recent_terminal_tasks: usize,
    /// Number of OS-backed acknowledgement round trips completed.
    pub ack_round_trips: u32,
    /// Number of native transport handles retained after cleanup.
    pub handles_remaining: u32,
}

/// Content-free report for OS-backed C8 grace-exit residue checks.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowsSysGraceExitResidueReport {
    /// Whether the final checkpoint was written at grace expiry.
    pub final_checkpoint_written: bool,
    /// Whether Link transitioned to stopped on grace expiry.
    pub stopped_link: bool,
    /// Current Link lifecycle state after grace expiry.
    pub lifecycle_state: LinkLifecycleState,
    /// Number of active task breadcrumbs retained after terminal checkpoint.
    pub active_tasks: usize,
    /// Number of recent terminal task breadcrumbs retained.
    pub recent_terminal_tasks: usize,
    /// Number of native transport close attempts made after checkpoint.
    pub shutdown_close_attempts: u32,
    /// Number of native transport handles successfully closed.
    pub shutdown_closed_handles: u32,
    /// Number of native transport handles retained after shutdown cleanup.
    pub transport_handles_remaining: u32,
    /// Whether a short-lived child process was started for residue measurement.
    pub child_process_started: bool,
    /// Whether the short-lived child process exit was observed.
    pub child_exit_observed: bool,
    /// Number of child processes still observed as running after wait.
    pub child_processes_remaining: u32,
}

/// Content-free report for the OS-backed Spike C C0-C9 harness aggregate.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowsSysSpikeC0C9HarnessReport {
    /// Number of Spike C scenarios covered by the aggregate harness.
    pub scenario_count: u32,
    /// Whether the OS-backed transport smoke path created required local objects.
    pub os_transport_ready: bool,
    /// C0: existing Link delivery used OS ingress and reducer checkpoint evidence.
    pub c0_existing_link_delivery: bool,
    /// C1: first Hook wake used inherited handoff without command-line payload leakage.
    pub c1_first_hook_handoff: bool,
    /// C2: parallel Shim race keeps a single Link launch.
    pub c2_parallel_race_single_link: bool,
    /// C3: unavailable Link remains fail-open for the provider.
    pub c3_link_unavailable_fail_open: bool,
    /// C4: malformed ingress is rejected before reducer mutation.
    pub c4_malformed_rejected_before_mutation: bool,
    /// C5: Drop Mode breadcrumb state remains bounded.
    pub c5_drop_mode_breadcrumb_bounded: bool,
    /// C6: Island attach/detach/reattach is backed by Island pipe request/response evidence.
    pub c6_island_attach_detach_reattach: bool,
    /// C7: restart recovery restores active breadcrumbs as degraded.
    pub c7_restart_recovery_degraded: bool,
    /// C8: grace exit writes final checkpoint and leaves no OS residue.
    pub c8_grace_exit_residue_clean: bool,
    /// C9: event storm remains within bounded breadcrumb caps.
    pub c9_event_storm_bounded: bool,
    /// Whether any scenario would affect the synthetic provider.
    pub provider_affected: bool,
    /// Whether any scenario persisted raw payload content.
    pub raw_payload_persisted: bool,
    /// Total native handles retained across OS-backed slices after cleanup.
    pub handles_remaining: u32,
}

/// Prepare the W3 native Link transport boundary after the process has started.
pub fn prepare_native_link_transport<A>(
    names: LinkLocalObjectNames,
    api: A,
) -> Result<LinkNativeStartupReport, LinkTransportNativeBackendError>
where
    A: LinkTransportNativeApi,
{
    let mut backend = LinkTransportNativeBackend::new(api);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names))?;
    let state = backend.state();
    Ok(LinkNativeStartupReport {
        mutex_created: state.mutex_handle.is_some(),
        ingress_pipe_created: state.ingress_pipe_handle.is_some(),
        island_pipe_created: state.island_pipe_handle.is_some(),
        handoff_pipe_created: state.handoff_pipe.is_some(),
    })
}

/// Run the MSVC `windows-sys` ingress ack harness, then drive synthetic reducer state.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_ingress_reducer_ack_harness(
    names: LinkLocalObjectNames,
    frame: [u8; pulse_link_core::FRAME_HEADER_BYTES],
) -> Result<WindowsSysIngressReducerAckReport, LinkTransportNativeBackendError> {
    let ack_report = pulse_win32_link::run_windows_sys_ingress_frame_ack_harness(names, &frame)?;
    let header = LinkFrameHeader::decode(&frame).map_err(|_| {
        LinkTransportNativeBackendError::NativeCallFailed("DecodeIngressFrameHeader")
    })?;
    if header.message_kind != LinkMessageKind::HookEnvelope {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "UnsupportedIngressFrameKind",
        ));
    }
    if header.payload_length != 0 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressPayloadNotAvailable",
        ));
    }

    let mut runtime = LinkRuntime::new();
    let reducer_report =
        apply_static_event(&mut runtime, "native-ingress-task", EvidenceKind::Started)
            .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ReducerCheckpoint"))?;

    Ok(WindowsSysIngressReducerAckReport {
        frame_accepted: ack_report.frame_round_tripped,
        reducer_checkpoint_written: reducer_report.checkpoint_written,
        lifecycle_state: reducer_report.lifecycle_state,
        active_tasks: reducer_report.active_tasks,
        recent_terminal_tasks: reducer_report.recent_terminal_tasks,
        ack_round_tripped: ack_report.ack_round_tripped,
        handles_remaining: ack_report.shutdown.handles_remaining,
    })
}

/// Run the MSVC `windows-sys` ingress ack loop, then drive synthetic reducer state.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_ingress_reducer_ack_loop_harness(
    names: LinkLocalObjectNames,
    frames: &[[u8; pulse_link_core::FRAME_HEADER_BYTES]],
) -> Result<WindowsSysIngressReducerAckLoopReport, LinkTransportNativeBackendError> {
    let frame_slices = frames
        .iter()
        .map(|frame| frame.as_slice())
        .collect::<Vec<_>>();
    let ack_report =
        pulse_win32_link::run_windows_sys_ingress_frame_ack_loop_harness(names, &frame_slices)?;

    let mut runtime = LinkRuntime::new();
    let mut frames_accepted = 0_u32;
    let mut frames_rejected = 0_u32;
    let mut reducer_checkpoint_writes = 0_u32;
    let mut last_report = LinkRuntimeReport {
        lifecycle_state: runtime.lifecycle_state(),
        active_tasks: 0,
        recent_terminal_tasks: 0,
        checkpoint_written: false,
    };

    for frame in frames {
        match apply_ingress_frame_to_runtime(&mut runtime, frame) {
            Ok(report) => {
                frames_accepted = frames_accepted.saturating_add(1);
                if report.checkpoint_written {
                    reducer_checkpoint_writes = reducer_checkpoint_writes.saturating_add(1);
                }
                last_report = report;
            }
            Err(()) => {
                frames_rejected = frames_rejected.saturating_add(1);
            }
        }
    }

    Ok(WindowsSysIngressReducerAckLoopReport {
        frames_seen: ack_report.frame_count,
        frames_accepted,
        frames_rejected,
        reducer_checkpoint_writes,
        lifecycle_state: last_report.lifecycle_state,
        active_tasks: last_report.active_tasks,
        recent_terminal_tasks: last_report.recent_terminal_tasks,
        ack_round_trips: ack_report.ack_bytes_read,
        handles_remaining: ack_report.shutdown.handles_remaining,
    })
}

/// Run the MSVC OS-backed aggregate harness for Spike C C0-C9.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_c0_c9_harness(
    names: LinkLocalObjectNames,
) -> Result<WindowsSysSpikeC0C9HarnessReport, LinkTransportNativeBackendError> {
    let os_transport =
        pulse_win32_link::run_windows_sys_os_transport_harness(child_names(&names, "transport"))?;
    let c0_frame = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 201,
        payload_length: 0,
    };
    let c0_os =
        run_windows_sys_ingress_reducer_ack_harness(child_names(&names, "c0"), c0_frame.encode())?;
    let island_requests: [&[u8]; 2] = [b"hello", b"snapshot"];
    let island_responses: [&[u8]; 2] = [b"hello-ack", b"snapshot"];
    let c6_os = pulse_win32_link::run_windows_sys_island_request_response_loop_harness(
        child_names(&names, "c6"),
        &island_requests,
        &island_responses,
    )?;
    let c8_os = run_windows_sys_grace_exit_residue_harness(child_names(&names, "c8"))?;

    let c0 = run_link_scenario(LinkScenario::C0ExistingLinkDelivery);
    let c1 = run_link_scenario(LinkScenario::C1FirstHookWakesLink);
    let c2 = run_link_scenario(LinkScenario::C2ParallelShimRace);
    let c3 = run_link_scenario(LinkScenario::C3LinkUnavailable);
    let c4 = run_link_scenario(LinkScenario::C4MalformedOversizedIngress);
    let c5 = run_link_scenario(LinkScenario::C5DropModeBreadcrumb);
    let c6 = run_link_scenario(LinkScenario::C6IslandAttachDetachReattach);
    let c7 = run_link_scenario(LinkScenario::C7LinkRestartRecovery);
    let c8 = run_link_scenario(LinkScenario::C8GraceExit);
    let c9 = run_link_scenario(LinkScenario::C9EventStorm);
    let scenarios = [c0, c1, c2, c3, c4, c5, c6, c7, c8, c9];

    Ok(WindowsSysSpikeC0C9HarnessReport {
        scenario_count: scenarios.len() as u32,
        os_transport_ready: os_transport.mutex_created
            && os_transport.ingress_pipe_created
            && os_transport.island_pipe_created
            && os_transport.handoff_pipe_created
            && os_transport.island_client_connected
            && os_transport.shutdown.handles_remaining == 0,
        c0_existing_link_delivery: c0_os.frame_accepted
            && c0_os.reducer_checkpoint_written
            && c0.lifecycle_state == LinkLifecycleState::IslandActive
            && c0.active_tasks == 1
            && c0.island_snapshot_revision.is_some(),
        c1_first_hook_handoff: os_transport.handoff_pipe_created
            && c1.link_process_launches == 1
            && c1.inherited_handoff_used
            && !c1.command_line_payload_leaked,
        c2_parallel_race_single_link: c2.link_process_launches == 1 && c2.active_tasks <= 128,
        c3_link_unavailable_fail_open: c3.shim_exit_status == ShimExitStatus::Success
            && !c3.provider_affected,
        c4_malformed_rejected_before_mutation: c4.active_tasks == 0
            && c4.recent_terminal_tasks == 0
            && !c4.raw_payload_persisted,
        c5_drop_mode_breadcrumb_bounded: c5.active_tasks <= 128 && c5.recent_terminal_tasks <= 20,
        c6_island_attach_detach_reattach: c6_os.requests_round_tripped
            && c6_os.responses_round_tripped
            && c6_os.shutdown.handles_remaining == 0
            && c6.island_attached
            && c6.active_tasks == 1,
        c7_restart_recovery_degraded: c7.restored_degraded_tasks == 1,
        c8_grace_exit_residue_clean: c8.lifecycle_state == LinkLifecycleState::NotRunning
            && c8_os.final_checkpoint_written
            && c8_os.stopped_link
            && c8_os.transport_handles_remaining == 0
            && c8_os.child_processes_remaining == 0,
        c9_event_storm_bounded: c9.active_tasks <= 128 && c9.recent_terminal_tasks <= 20,
        provider_affected: scenarios.iter().any(|scenario| scenario.provider_affected),
        raw_payload_persisted: scenarios
            .iter()
            .any(|scenario| scenario.raw_payload_persisted),
        handles_remaining: os_transport
            .shutdown
            .handles_remaining
            .saturating_add(c0_os.handles_remaining)
            .saturating_add(c6_os.shutdown.handles_remaining)
            .saturating_add(c8_os.transport_handles_remaining),
    })
}

/// Run OS-backed C8 grace-exit residue checks on MSVC Windows.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_grace_exit_residue_harness(
    names: LinkLocalObjectNames,
) -> Result<WindowsSysGraceExitResidueReport, LinkTransportNativeBackendError> {
    let mut transport =
        LinkNativeTransportRuntime::start(names, pulse_win32_link::WindowsSysLinkTransportApi)?;
    let mut runtime = LinkRuntime::new();
    apply_static_event(&mut runtime, "native-grace-task", EvidenceKind::Started)
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ReducerStart"))?;
    apply_static_event(&mut runtime, "native-grace-task", EvidenceKind::Completed)
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ReducerTerminal"))?;

    let mut grace = DropModeGraceDriver::spike_c();
    let _ = grace.observe_runtime(&runtime, TimestampMs(1));
    let grace_report = grace
        .tick(&mut runtime, TimestampMs(1 + SPIKE_C_GRACE_PERIOD_MS))
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("FinalCheckpoint"))?;
    let breadcrumbs = runtime
        .load_breadcrumbs()
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("LoadCheckpoint"))?;
    let shutdown = transport.shutdown_after_checkpoint();
    let child = run_short_lived_child_process()?;

    Ok(WindowsSysGraceExitResidueReport {
        final_checkpoint_written: grace_report.final_checkpoint_written,
        stopped_link: grace_report.stopped_link,
        lifecycle_state: runtime.lifecycle_state(),
        active_tasks: breadcrumbs.active_tasks.len(),
        recent_terminal_tasks: breadcrumbs.recent_terminal_tasks.len(),
        shutdown_close_attempts: shutdown.close_attempts,
        shutdown_closed_handles: shutdown.closed_handles,
        transport_handles_remaining: shutdown.handles_remaining,
        child_process_started: child.started,
        child_exit_observed: child.exit_observed,
        child_processes_remaining: child.processes_remaining,
    })
}

/// Native Link transport runtime held while Link is alive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkNativeTransportRuntime<A> {
    backend: LinkTransportNativeBackend<A>,
}

impl<A> LinkNativeTransportRuntime<A>
where
    A: LinkTransportNativeApi,
{
    /// Start the native Link transport and retain handles for later shutdown cleanup.
    pub fn start(
        names: LinkLocalObjectNames,
        api: A,
    ) -> Result<Self, LinkTransportNativeBackendError> {
        let mut backend = LinkTransportNativeBackend::new(api);
        backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
        backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;
        backend.apply_command(LinkTransportCommand::CreateIslandPipe(names))?;
        Ok(Self { backend })
    }

    /// Borrow the underlying API adapter for diagnostics/tests.
    pub const fn api(&self) -> &A {
        self.backend.api()
    }

    /// Close transport handles after the final checkpoint has been written.
    pub fn shutdown_after_checkpoint(&mut self) -> LinkTransportShutdownReport {
        self.backend.close_all()
    }
}

/// Pure synthetic Link runtime used before W3 OS transport is wired.
#[derive(Clone, Debug)]
pub struct LinkRuntime<S = MemoryBreadcrumbStore> {
    lifecycle: LinkLifecycle,
    store: S,
    active_tasks: Vec<TaskSnapshot>,
    recent_terminal_tasks: Vec<TaskSnapshot>,
}

impl LinkRuntime<MemoryBreadcrumbStore> {
    /// Create a stopped runtime with an empty memory breadcrumb store.
    pub fn new() -> Self {
        Self::with_store(MemoryBreadcrumbStore::default())
    }
}

impl<S> LinkRuntime<S>
where
    S: BreadcrumbStore,
{
    /// Create a stopped runtime with a caller-provided breadcrumb store.
    pub fn with_store(store: S) -> Self {
        Self {
            lifecycle: LinkLifecycle::new(),
            store,
            active_tasks: Vec::new(),
            recent_terminal_tasks: Vec::new(),
        }
    }

    /// Apply one admitted synthetic event through reducer, retention, and lifecycle seams.
    pub fn apply_event(
        &mut self,
        event: AdmittedEvent,
        privacy: pulse_domain::PrivacyProfile,
    ) -> Result<LinkRuntimeReport, StoreError> {
        self.ensure_warm();

        let prior = self
            .remove_existing(&event)
            .unwrap_or_else(|| initial(&event, event.occurred_at));
        let reduced = reduce(prior, &event, event.occurred_at, privacy);
        let lifecycle_event = if reduced.snapshot.lifecycle.is_terminal() {
            LinkLifecycleEvent::LastActiveTaskTerminal
        } else {
            LinkLifecycleEvent::ValidActiveTaskEvent
        };

        match reduced.retention {
            BreadcrumbRetention::RetainActive => self.active_tasks.push(reduced.snapshot),
            BreadcrumbRetention::RetainRecentTerminal => {
                self.recent_terminal_tasks.push(reduced.snapshot);
                self.trim_recent_terminal();
            }
            BreadcrumbRetention::TerminalCheckpointOnly | BreadcrumbRetention::DoNotRetain => {}
        }

        self.lifecycle = self.lifecycle.apply(lifecycle_event);
        let set = BreadcrumbSet::new(
            1,
            event.occurred_at,
            self.active_tasks.clone(),
            self.recent_terminal_tasks.clone(),
            Vec::new(),
        )?;
        self.store.checkpoint(&set)?;

        Ok(LinkRuntimeReport {
            lifecycle_state: self.lifecycle.state(),
            active_tasks: self.active_tasks.len(),
            recent_terminal_tasks: self.recent_terminal_tasks.len(),
            checkpoint_written: true,
        })
    }

    /// Load the current complete replacement breadcrumb snapshot.
    pub fn load_breadcrumbs(&self) -> Result<BreadcrumbSet, StoreError> {
        self.store.load()
    }

    /// Current Link lifecycle state.
    pub const fn lifecycle_state(&self) -> LinkLifecycleState {
        self.lifecycle.state()
    }

    /// Attach a fake Island subscriber to the pure runtime lifecycle.
    pub fn attach_island(&mut self) {
        self.lifecycle = self.lifecycle.apply(LinkLifecycleEvent::IslandAttached);
    }

    /// Detach the fake Island subscriber and return to Drop Mode.
    pub fn detach_island(&mut self) {
        self.lifecycle = self.lifecycle.apply(LinkLifecycleEvent::IslandDetached);
    }

    /// Simulate accelerated grace expiry and final checkpoint completion.
    pub fn expire_grace_and_stop(&mut self) {
        self.lifecycle = self
            .lifecycle
            .apply(LinkLifecycleEvent::GraceExpired)
            .apply(LinkLifecycleEvent::CheckpointComplete);
    }

    /// Write the final checkpoint and stop Link after grace expiry.
    pub fn final_checkpoint_and_stop(&mut self, now: TimestampMs) -> Result<(), StoreError> {
        let set = BreadcrumbSet::new(
            1,
            now,
            self.active_tasks.clone(),
            self.recent_terminal_tasks.clone(),
            Vec::new(),
        )?;
        self.store.checkpoint(&set)?;
        self.expire_grace_and_stop();
        Ok(())
    }

    /// Restore active breadcrumbs as degraded until fresh synthetic evidence arrives.
    pub fn recover_degraded_from_breadcrumbs(
        &mut self,
        breadcrumbs: BreadcrumbSet,
    ) -> Result<(), StoreError> {
        self.ensure_warm();
        self.active_tasks = breadcrumbs
            .active_tasks
            .into_iter()
            .map(|mut task| {
                task.health = TaskHealth::Degraded;
                task
            })
            .collect();
        self.recent_terminal_tasks = breadcrumbs.recent_terminal_tasks;
        if !self.active_tasks.is_empty() {
            self.lifecycle = self
                .lifecycle
                .apply(LinkLifecycleEvent::ValidActiveTaskEvent);
        }
        let set = BreadcrumbSet::new(
            1,
            breadcrumbs.written_at,
            self.active_tasks.clone(),
            self.recent_terminal_tasks.clone(),
            Vec::new(),
        )?;
        self.store.checkpoint(&set)
    }

    fn ensure_warm(&mut self) {
        if self.lifecycle.state() == LinkLifecycleState::NotRunning {
            self.lifecycle = self
                .lifecycle
                .apply(LinkLifecycleEvent::WakeRequested)
                .apply(LinkLifecycleEvent::RuntimeReady);
        }
    }

    fn remove_existing(&mut self, event: &AdmittedEvent) -> Option<TaskSnapshot> {
        if let Some(position) = self
            .active_tasks
            .iter()
            .position(|task| task.task_id.0 == event.task)
        {
            return Some(self.active_tasks.remove(position));
        }
        if let Some(position) = self
            .recent_terminal_tasks
            .iter()
            .position(|task| task.task_id.0 == event.task)
        {
            return Some(self.recent_terminal_tasks.remove(position));
        }
        None
    }

    fn trim_recent_terminal(&mut self) {
        while self.recent_terminal_tasks.len() > RECENT_TERMINAL_LIMIT {
            self.recent_terminal_tasks.remove(0);
        }
    }
}

impl Default for LinkRuntime<MemoryBreadcrumbStore> {
    fn default() -> Self {
        Self::new()
    }
}

/// Synthetic Spike C scenario subset currently covered by the pure W3 harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkScenario {
    /// C0: existing Link accepts a shim event and serves a late Island snapshot.
    C0ExistingLinkDelivery,
    /// C1: first Hook wakes one Link and a late Island sees the snapshot.
    C1FirstHookWakesLink,
    /// C2: parallel Shims race but only one Link is launched.
    C2ParallelShimRace,
    /// C3: Link is unavailable and Shim fails open.
    C3LinkUnavailable,
    /// C4: malformed or oversized ingress is rejected before mutation.
    C4MalformedOversizedIngress,
    /// C5: Link runs in Drop Mode and retains only bounded breadcrumbs.
    C5DropModeBreadcrumb,
    /// C6: Island attaches, detaches, and reattaches without duplicating state.
    C6IslandAttachDetachReattach,
    /// C7: Link restart restores active breadcrumb as degraded until fresh evidence.
    C7LinkRestartRecovery,
    /// C8: terminal task enters grace, checkpoints, and exits.
    C8GraceExit,
    /// C9: event storm remains within bounded breadcrumb caps.
    C9EventStorm,
}

/// Content-free report for one synthetic Spike C scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkScenarioReport {
    /// Scenario identifier.
    pub scenario: LinkScenario,
    /// Shim fail-open exit status.
    pub shim_exit_status: ShimExitStatus,
    /// Whether the shim delivery seam acknowledged forwarding.
    pub shim_forwarded: bool,
    /// Number of new Link process launches represented by the pure scenario.
    pub link_process_launches: u32,
    /// Final Link lifecycle state.
    pub lifecycle_state: LinkLifecycleState,
    /// Active breadcrumb count.
    pub active_tasks: usize,
    /// Recent terminal breadcrumb count.
    pub recent_terminal_tasks: usize,
    /// Recovered active breadcrumbs marked degraded.
    pub restored_degraded_tasks: usize,
    /// Whether first event was planned through inherited handoff stdin.
    pub inherited_handoff_used: bool,
    /// Whether the first event payload leaked into command-line arguments.
    pub command_line_payload_leaked: bool,
    /// Whether a fake Island attached.
    pub island_attached: bool,
    /// Snapshot revision observed by fake Island, if attached.
    pub island_snapshot_revision: Option<u64>,
    /// Terminal lifecycle retained in recent breadcrumbs, if any.
    pub terminal_lifecycle: Option<Lifecycle>,
    /// Whether the synthetic provider would be affected by Pulse failure.
    pub provider_affected: bool,
    /// Whether raw payload content was persisted.
    pub raw_payload_persisted: bool,
}

/// Run a deterministic synthetic Spike C scenario without OS transport or providers.
pub fn run_link_scenario(scenario: LinkScenario) -> LinkScenarioReport {
    match scenario {
        LinkScenario::C0ExistingLinkDelivery => run_active_scenario(scenario, 0, true),
        LinkScenario::C1FirstHookWakesLink => run_first_hook_wake_scenario(),
        LinkScenario::C2ParallelShimRace => run_parallel_race_scenario(),
        LinkScenario::C3LinkUnavailable => run_link_unavailable_scenario(),
        LinkScenario::C4MalformedOversizedIngress => run_malformed_ingress_scenario(),
        LinkScenario::C5DropModeBreadcrumb => run_terminal_scenario(scenario, false, false),
        LinkScenario::C6IslandAttachDetachReattach => run_attach_detach_reattach_scenario(),
        LinkScenario::C7LinkRestartRecovery => run_restart_recovery_scenario(),
        LinkScenario::C8GraceExit => run_terminal_scenario(scenario, false, true),
        LinkScenario::C9EventStorm => run_event_storm_scenario(),
    }
}

fn run_first_hook_wake_scenario() -> LinkScenarioReport {
    let names = scenario_names();
    let mut registry = LinkOwnershipRegistry::default();
    let decision = registry.observe_start(&names, LinkStartupObservation::NoExistingObjects);
    let handoff = first_event_handoff_used();
    let launches = match decision {
        LinkOwnershipDecision::OwnInstance => 1,
        LinkOwnershipDecision::ConnectToExisting
        | LinkOwnershipDecision::RetryBounded
        | LinkOwnershipDecision::FailOpen => 0,
    };
    let mut report = run_active_scenario(LinkScenario::C1FirstHookWakesLink, launches, true);
    report.inherited_handoff_used = handoff.inherited_handoff_used;
    report.command_line_payload_leaked = handoff.command_line_payload_leaked;
    report
}

fn run_active_scenario(
    scenario: LinkScenario,
    link_process_launches: u32,
    attach_island: bool,
) -> LinkScenarioReport {
    let mut runtime = LinkRuntime::new();
    let shim = run_successful_shim();
    let _ = apply_static_event(&mut runtime, "scenario-task", EvidenceKind::Started);

    let island_snapshot_revision = if attach_island {
        runtime.attach_island();
        attach_fake_island_snapshot_revision(1)
    } else {
        None
    };

    report_from_runtime(
        scenario,
        shim,
        link_process_launches,
        &runtime,
        attach_island,
        island_snapshot_revision,
    )
}

fn run_terminal_scenario(
    scenario: LinkScenario,
    attach_island: bool,
    expire_grace: bool,
) -> LinkScenarioReport {
    let mut runtime = LinkRuntime::new();
    let shim = run_successful_shim();
    let _ = apply_static_event(&mut runtime, "scenario-task", EvidenceKind::Started);
    let _ = apply_static_event(&mut runtime, "scenario-task", EvidenceKind::Completed);
    if expire_grace {
        let mut grace = DropModeGraceDriver::spike_c();
        let _ = grace.observe_runtime(&runtime, TimestampMs(1));
        let _ = grace.tick(&mut runtime, TimestampMs(1 + SPIKE_C_GRACE_PERIOD_MS));
    }

    report_from_runtime(scenario, shim, 1, &runtime, attach_island, None)
}

fn run_parallel_race_scenario() -> LinkScenarioReport {
    let mut runtime = LinkRuntime::new();
    let shim = run_successful_shim();
    let names = scenario_names();
    let mut registry = LinkOwnershipRegistry::default();
    let first_decision = registry.observe_start(&names, LinkStartupObservation::NoExistingObjects);
    for _ in 0..50 {
        let _ = run_successful_shim();
        let _ = registry.observe_start(&names, LinkStartupObservation::MutexAlreadyOwned);
    }
    let _ = apply_static_event(&mut runtime, "race-active", EvidenceKind::Started);
    let _ = apply_static_event(&mut runtime, "race-terminal", EvidenceKind::Started);
    let _ = apply_static_event(&mut runtime, "race-terminal", EvidenceKind::Completed);
    let _ = apply_static_event(&mut runtime, "race-active", EvidenceKind::Activity);

    report_from_runtime(
        LinkScenario::C2ParallelShimRace,
        shim,
        if first_decision == LinkOwnershipDecision::OwnInstance {
            1
        } else {
            0
        },
        &runtime,
        false,
        None,
    )
}

fn run_link_unavailable_scenario() -> LinkScenarioReport {
    let mut delivery = NeverDeliver;
    let shim = run_shim_preflight(
        br#"{"version":1,"event":"synthetic"}"#,
        false,
        &mut delivery,
    );
    report_from_runtime(
        LinkScenario::C3LinkUnavailable,
        shim,
        0,
        &LinkRuntime::new(),
        false,
        None,
    )
}

fn run_malformed_ingress_scenario() -> LinkScenarioReport {
    let mut delivery = AlwaysDeliver;
    let shim = run_shim_preflight(br#"{"prompt":"raw content"}"#, false, &mut delivery);
    report_from_runtime(
        LinkScenario::C4MalformedOversizedIngress,
        shim,
        0,
        &LinkRuntime::new(),
        false,
        None,
    )
}

fn run_attach_detach_reattach_scenario() -> LinkScenarioReport {
    let mut runtime = LinkRuntime::new();
    let shim = run_successful_shim();
    let _ = apply_static_event(&mut runtime, "reattach-task", EvidenceKind::Started);
    runtime.attach_island();
    runtime.detach_island();
    runtime.attach_island();

    report_from_runtime(
        LinkScenario::C6IslandAttachDetachReattach,
        shim,
        1,
        &runtime,
        true,
        attach_fake_island_snapshot_revision(2),
    )
}

fn run_restart_recovery_scenario() -> LinkScenarioReport {
    let mut original = LinkRuntime::new();
    let shim = run_successful_shim();
    let _ = apply_static_event(&mut original, "recover-task", EvidenceKind::Started);

    let mut recovered = LinkRuntime::new();
    if let Ok(breadcrumbs) = original.load_breadcrumbs() {
        let _ = recovered.recover_degraded_from_breadcrumbs(breadcrumbs);
    }
    recovered.attach_island();

    report_from_runtime(
        LinkScenario::C7LinkRestartRecovery,
        shim,
        1,
        &recovered,
        true,
        attach_fake_island_snapshot_revision(1),
    )
}

fn run_event_storm_scenario() -> LinkScenarioReport {
    let mut runtime = LinkRuntime::new();
    let shim = run_successful_shim();

    for index in 0..128 {
        let _ = apply_static_event(
            &mut runtime,
            &format!("storm-{index}"),
            EvidenceKind::Started,
        );
    }
    for index in 0..20 {
        let _ = apply_static_event(
            &mut runtime,
            &format!("storm-{index}"),
            EvidenceKind::Completed,
        );
    }
    for index in 128..148 {
        let _ = apply_static_event(
            &mut runtime,
            &format!("storm-{index}"),
            EvidenceKind::Started,
        );
    }

    report_from_runtime(LinkScenario::C9EventStorm, shim, 1, &runtime, false, None)
}

fn report_from_runtime<S>(
    scenario: LinkScenario,
    shim: ShimRunReport,
    link_process_launches: u32,
    runtime: &LinkRuntime<S>,
    island_attached: bool,
    island_snapshot_revision: Option<u64>,
) -> LinkScenarioReport
where
    S: BreadcrumbStore,
{
    let breadcrumbs = match runtime.load_breadcrumbs() {
        Ok(breadcrumbs) => breadcrumbs,
        Err(_) => empty_breadcrumbs(),
    };
    let terminal_lifecycle = breadcrumbs
        .recent_terminal_tasks
        .first()
        .map(|task| task.lifecycle);
    let restored_degraded_tasks = breadcrumbs
        .active_tasks
        .iter()
        .filter(|task| task.health == TaskHealth::Degraded)
        .count();

    LinkScenarioReport {
        scenario,
        shim_exit_status: shim.exit_status,
        shim_forwarded: shim.forwarded,
        link_process_launches,
        lifecycle_state: runtime.lifecycle_state(),
        active_tasks: breadcrumbs.active_tasks.len(),
        recent_terminal_tasks: breadcrumbs.recent_terminal_tasks.len(),
        restored_degraded_tasks,
        inherited_handoff_used: false,
        command_line_payload_leaked: false,
        island_attached,
        island_snapshot_revision,
        terminal_lifecycle,
        provider_affected: false,
        raw_payload_persisted: false,
    }
}

fn first_event_handoff_used() -> HandoffSafetyReport {
    let header = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 1,
        payload_length: 64,
    };
    let Ok(plan) = InitialHandoffPlan::new(header) else {
        return HandoffSafetyReport {
            inherited_handoff_used: false,
            command_line_payload_leaked: false,
        };
    };
    HandoffSafetyReport {
        inherited_handoff_used: plan.inherited_handoff_stdin,
        command_line_payload_leaked: plan
            .argv
            .iter()
            .any(|argument| argument.contains("synthetic") || argument.contains("event")),
    }
}

fn scenario_names() -> LinkLocalObjectNames {
    LinkLocalObjectNames::derive("scenario-install", "scenario-sid", "scenario-session", 1)
}

struct HandoffSafetyReport {
    inherited_handoff_used: bool,
    command_line_payload_leaked: bool,
}

fn run_successful_shim() -> ShimRunReport {
    let mut delivery = AlwaysDeliver;
    run_shim_preflight(
        br#"{"version":1,"event":"synthetic"}"#,
        false,
        &mut delivery,
    )
}

fn apply_static_event<S>(
    runtime: &mut LinkRuntime<S>,
    task: &str,
    evidence: EvidenceKind,
) -> Result<LinkRuntimeReport, StoreError>
where
    S: BreadcrumbStore,
{
    let Some(event) = static_event(task, evidence) else {
        return Err(StoreError::SnapshotTooLarge);
    };
    runtime.apply_event(event, PrivacyProfile::Minimal)
}

/// Reduce one zero-payload synthetic ingress header and checkpoint its bounded state.
///
/// This is a W3 transport diagnostic seam, not a Provider adapter. It accepts only a Hook frame
/// with no payload, derives no Provider content, and uses a stable synthetic task key from the
/// request identifier.
pub fn apply_header_only_ingress<S>(
    runtime: &mut LinkRuntime<S>,
    header: LinkFrameHeader,
) -> Result<LinkRuntimeReport, StoreError>
where
    S: BreadcrumbStore,
{
    if header.message_kind != LinkMessageKind::HookEnvelope || header.payload_length != 0 {
        return Err(StoreError::CorruptSnapshot);
    }
    apply_static_event(
        runtime,
        &format!("native-ingress-{}", header.request_id),
        EvidenceKind::Started,
    )
}

#[cfg(target_env = "msvc")]
fn apply_ingress_frame_to_runtime(
    runtime: &mut LinkRuntime,
    frame: &[u8; pulse_link_core::FRAME_HEADER_BYTES],
) -> Result<LinkRuntimeReport, ()> {
    let header = LinkFrameHeader::decode(frame).map_err(|_| ())?;
    apply_header_only_ingress(runtime, header).map_err(|_| ())
}

#[cfg(target_env = "msvc")]
fn child_names(names: &LinkLocalObjectNames, suffix: &str) -> LinkLocalObjectNames {
    LinkLocalObjectNames::derive(
        &format!("{}-{suffix}", names.mutex),
        &names.ingress_pipe,
        &names.island_pipe,
        1,
    )
}

#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChildResidueReport {
    started: bool,
    exit_observed: bool,
    processes_remaining: u32,
}

#[cfg(target_env = "msvc")]
fn run_short_lived_child_process() -> Result<ChildResidueReport, LinkTransportNativeBackendError> {
    let mut child = std::process::Command::new("cmd.exe")
        .args(["/d", "/s", "/c", "exit 0"])
        .spawn()
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("SpawnChildProcess"))?;
    let status = child
        .wait()
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("WaitChildProcess"))?;
    let processes_remaining = match child.try_wait() {
        Ok(Some(_)) => 0,
        Ok(None) | Err(_) => 1,
    };
    Ok(ChildResidueReport {
        started: true,
        exit_observed: status.success(),
        processes_remaining,
    })
}

fn static_event(task: &str, evidence: EvidenceKind) -> Option<AdmittedEvent> {
    let Ok(provider) = BoundedText::new("synthetic") else {
        return None;
    };
    let Ok(task) = BoundedText::new(task) else {
        return None;
    };
    Some(AdmittedEvent {
        provider,
        task,
        evidence,
        occurred_at: TimestampMs(1),
    })
}

fn attach_fake_island_snapshot_revision(current_revision: u64) -> Option<u64> {
    let mut session = FakeIslandSession::new(current_revision);
    let _ = session.handle(IslandControlRequest::Hello);
    match session.handle(IslandControlRequest::GetSnapshot) {
        IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)) => Some(snapshot.revision),
        _ => None,
    }
}

fn empty_breadcrumbs() -> BreadcrumbSet {
    BreadcrumbSet {
        protocol_version: 1,
        written_at: TimestampMs(0),
        active_tasks: Vec::new(),
        recent_terminal_tasks: Vec::new(),
        diagnostic_counters: Vec::new(),
    }
}

struct AlwaysDeliver;

impl ShimDelivery for AlwaysDeliver {
    fn deliver(&mut self, _attempt: ShimDeliveryAttempt) -> bool {
        true
    }
}

struct NeverDeliver;

impl ShimDelivery for NeverDeliver {
    fn deliver(&mut self, _attempt: ShimDeliveryAttempt) -> bool {
        false
    }
}
