//! Native Windows Link transport boundary for Pulse Island.
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::num::NonZeroIsize;

use pulse_win32::LinkLocalObjectNames;

/// The only crate intended to contain future unsafe Link transport FFI calls.
pub const LINK_TRANSPORT_UNSAFE_BOUNDARY_CRATE: &str = "pulse-win32-link";

/// Non-null raw Windows handle value kept out of provider-neutral crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawLinkHandle(NonZeroIsize);

impl RawLinkHandle {
    /// Create a raw handle wrapper from a non-zero platform handle value.
    pub const fn new(value: isize) -> Option<Self> {
        match NonZeroIsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Return the stored platform handle value.
    pub const fn value(self) -> isize {
        self.0.get()
    }
}

/// Link transport setup commands that may reach the native adapter after preflight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTransportCommand {
    /// Create or acquire the scoped single-instance mutex.
    CreateMutex(LinkLocalObjectNames),
    /// Create the Shim-to-Link named pipe server.
    CreateIngressPipe(LinkLocalObjectNames),
    /// Create the Island named pipe server.
    CreateIslandPipe(LinkLocalObjectNames),
    /// Create the inherited anonymous pipe used for first-event handoff.
    CreateInheritedHandoffPipe,
    /// Connect a fake Island client to the scoped Island pipe.
    ConnectIslandClient(LinkLocalObjectNames),
}

/// Safe preflight state for Link transport setup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkTransportState {
    /// Whether the scoped mutex was created/acquired.
    pub mutex_created: bool,
    /// Whether the ingress pipe server was created.
    pub ingress_pipe_created: bool,
    /// Whether the Island pipe server was created.
    pub island_pipe_created: bool,
    /// Whether the anonymous handoff pipe was created.
    pub handoff_pipe_created: bool,
    /// Whether an Island client connection was opened.
    pub island_client_connected: bool,
}

/// Safe preflight errors before native Link transport FFI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkTransportPreflightError {
    /// A pipe server command was attempted before mutex ownership.
    MutexMissing,
    /// Mutex creation/acquisition was requested twice.
    MutexAlreadyCreated,
    /// Ingress pipe creation was requested twice.
    IngressPipeAlreadyCreated,
    /// Island pipe creation was requested twice.
    IslandPipeAlreadyCreated,
    /// Anonymous handoff pipe creation was requested twice.
    HandoffAlreadyCreated,
    /// Island client connection was attempted before the Island pipe server exists.
    IslandPipeMissing,
    /// Island client connection was requested twice.
    IslandClientAlreadyConnected,
}

/// Safe preflight sink that gates native Link transport commands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkTransportPreflightSink {
    state: LinkTransportState,
    commands: Vec<LinkTransportCommand>,
}

impl LinkTransportPreflightSink {
    /// Validate and record a command before it can reach native transport FFI.
    pub fn validate_command(
        &mut self,
        command: LinkTransportCommand,
    ) -> Result<(), LinkTransportPreflightError> {
        self.validate(&command)?;
        self.apply_validated(command);
        Ok(())
    }

    /// Current preflight state.
    pub const fn state(&self) -> LinkTransportState {
        self.state
    }

    /// Commands accepted by this preflight sink.
    pub fn accepted_commands(&self) -> Vec<LinkTransportCommand> {
        self.commands.clone()
    }

    fn validate(&self, command: &LinkTransportCommand) -> Result<(), LinkTransportPreflightError> {
        match command {
            LinkTransportCommand::CreateMutex(_) => {
                if self.state.mutex_created {
                    Err(LinkTransportPreflightError::MutexAlreadyCreated)
                } else {
                    Ok(())
                }
            }
            LinkTransportCommand::CreateIngressPipe(_) => {
                self.require_mutex()?;
                if self.state.ingress_pipe_created {
                    Err(LinkTransportPreflightError::IngressPipeAlreadyCreated)
                } else {
                    Ok(())
                }
            }
            LinkTransportCommand::CreateIslandPipe(_) => {
                self.require_mutex()?;
                if self.state.island_pipe_created {
                    Err(LinkTransportPreflightError::IslandPipeAlreadyCreated)
                } else {
                    Ok(())
                }
            }
            LinkTransportCommand::CreateInheritedHandoffPipe => {
                if self.state.handoff_pipe_created {
                    Err(LinkTransportPreflightError::HandoffAlreadyCreated)
                } else {
                    Ok(())
                }
            }
            LinkTransportCommand::ConnectIslandClient(_) => {
                if !self.state.island_pipe_created {
                    Err(LinkTransportPreflightError::IslandPipeMissing)
                } else if self.state.island_client_connected {
                    Err(LinkTransportPreflightError::IslandClientAlreadyConnected)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn require_mutex(&self) -> Result<(), LinkTransportPreflightError> {
        if self.state.mutex_created {
            Ok(())
        } else {
            Err(LinkTransportPreflightError::MutexMissing)
        }
    }

    fn apply_validated(&mut self, command: LinkTransportCommand) {
        match command {
            LinkTransportCommand::CreateMutex(_) => {
                self.state.mutex_created = true;
            }
            LinkTransportCommand::CreateIngressPipe(_) => {
                self.state.ingress_pipe_created = true;
            }
            LinkTransportCommand::CreateIslandPipe(_) => {
                self.state.island_pipe_created = true;
            }
            LinkTransportCommand::CreateInheritedHandoffPipe => {
                self.state.handoff_pipe_created = true;
            }
            LinkTransportCommand::ConnectIslandClient(_) => {
                self.state.island_client_connected = true;
            }
        }
        self.commands.push(command);
    }
}

/// Native Link transport API called only after safe preflight.
pub trait LinkTransportNativeApi {
    /// Create/acquire a scoped named mutex.
    fn create_mutex(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle>;

    /// Create the ingress named pipe server.
    fn create_ingress_pipe(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle>;

    /// Create the Island named pipe server.
    fn create_island_pipe(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle>;

    /// Create inheritable read/write handles for first-event handoff.
    fn create_inherited_handoff_pipe(&mut self) -> Option<InheritedHandoffPipe>;

    /// Connect a fake Island client to the Island pipe.
    fn connect_island_client(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle>;

    /// Close a raw transport handle.
    fn close_handle(&mut self, handle: RawLinkHandle) -> bool;
}

/// Anonymous handoff pipe handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InheritedHandoffPipe {
    /// Inherited read handle passed to Link as handoff stdin.
    pub read: RawLinkHandle,
    /// Shim-owned write handle used to send the validated frame.
    pub write: RawLinkHandle,
}

/// Native transport handles owned by the Link transport boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkTransportNativeState {
    /// Scoped mutex handle.
    pub mutex_handle: Option<RawLinkHandle>,
    /// Ingress named pipe server handle.
    pub ingress_pipe_handle: Option<RawLinkHandle>,
    /// Island named pipe server handle.
    pub island_pipe_handle: Option<RawLinkHandle>,
    /// Inherited first-event handoff pipe.
    pub handoff_pipe: Option<InheritedHandoffPipe>,
    /// Inherited first-event handoff read handle retained for independent cleanup.
    pub handoff_read_handle: Option<RawLinkHandle>,
    /// Shim-owned first-event handoff write handle retained for independent cleanup.
    pub handoff_write_handle: Option<RawLinkHandle>,
    /// Connected fake Island client handle.
    pub island_client_handle: Option<RawLinkHandle>,
}

impl LinkTransportNativeState {
    /// Count currently retained native handles.
    pub const fn handle_count(self) -> u32 {
        let mut count = 0;
        if self.mutex_handle.is_some() {
            count += 1;
        }
        if self.ingress_pipe_handle.is_some() {
            count += 1;
        }
        if self.island_pipe_handle.is_some() {
            count += 1;
        }
        if self.handoff_read_handle.is_some() {
            count += 1;
        }
        if self.handoff_write_handle.is_some() {
            count += 1;
        }
        if self.island_client_handle.is_some() {
            count += 1;
        }
        count
    }
}

/// Content-free report for native Link transport shutdown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkTransportShutdownReport {
    /// Number of close attempts made.
    pub close_attempts: u32,
    /// Number of handles successfully closed.
    pub closed_handles: u32,
    /// Number of handles that failed to close and remain retained.
    pub failed_closes: u32,
    /// Number of handles still retained for retry.
    pub handles_remaining: u32,
}

/// Content-free report for an OS-backed W3 Link transport harness run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysOsTransportHarnessReport {
    /// Whether the scoped mutex was created/acquired.
    pub mutex_created: bool,
    /// Whether the ingress pipe server was created.
    pub ingress_pipe_created: bool,
    /// Whether the Island pipe server was created.
    pub island_pipe_created: bool,
    /// Whether the inherited first-event handoff pipe was created.
    pub handoff_pipe_created: bool,
    /// Whether a fake Island client connected to the Island pipe.
    pub island_client_connected: bool,
    /// Shutdown cleanup report after closing all owned handles.
    pub shutdown: LinkTransportShutdownReport,
}

/// Content-free report for a real ingress pipe frame/ack round trip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysIngressFrameAckReport {
    /// Number of frame bytes written by the client.
    pub frame_bytes_written: u32,
    /// Number of frame bytes read by the server.
    pub frame_bytes_read: u32,
    /// Number of acknowledgement bytes written by the server.
    pub ack_bytes_written: u32,
    /// Number of acknowledgement bytes read by the client.
    pub ack_bytes_read: u32,
    /// Whether the server read the exact frame bytes written by the client.
    pub frame_round_tripped: bool,
    /// Whether the client read the exact acknowledgement byte written by the server.
    pub ack_round_tripped: bool,
    /// Shutdown cleanup report after closing client, pipe server, and mutex handles.
    pub shutdown: LinkTransportShutdownReport,
}

/// Content-free report for a real ingress pipe multi-frame/ack loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysIngressFrameAckLoopReport {
    /// Number of frames attempted by the client.
    pub frame_count: u32,
    /// Total frame bytes written by the client.
    pub frame_bytes_written: u32,
    /// Total frame bytes read by the server.
    pub frame_bytes_read: u32,
    /// Total acknowledgement bytes written by the server.
    pub ack_bytes_written: u32,
    /// Total acknowledgement bytes read by the client.
    pub ack_bytes_read: u32,
    /// Whether every frame read exactly matched the corresponding write.
    pub frames_round_tripped: bool,
    /// Whether every frame received one acknowledgement byte.
    pub acks_round_tripped: bool,
    /// Shutdown cleanup report after closing client, pipe server, and mutex handles.
    pub shutdown: LinkTransportShutdownReport,
}

/// Content-free report for a real Island pipe request/response loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysIslandRequestResponseLoopReport {
    /// Number of request/response pairs attempted.
    pub round_trip_count: u32,
    /// Total request bytes written by the client.
    pub request_bytes_written: u32,
    /// Total request bytes read by the server.
    pub request_bytes_read: u32,
    /// Total response bytes written by the server.
    pub response_bytes_written: u32,
    /// Total response bytes read by the client.
    pub response_bytes_read: u32,
    /// Whether every request read exactly matched the corresponding write.
    pub requests_round_tripped: bool,
    /// Whether every response read exactly matched the corresponding write.
    pub responses_round_tripped: bool,
    /// Shutdown cleanup report after closing client, pipe server, and mutex handles.
    pub shutdown: LinkTransportShutdownReport,
}

/// Error returned by the native Link transport backend boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTransportNativeBackendError {
    /// Preflight rejected the command before any native call.
    Preflight(LinkTransportPreflightError),
    /// Native API returned no handle or failed.
    NativeCallFailed(&'static str),
}

/// Safe executor that gates native Link transport calls behind preflight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkTransportNativeBackend<A> {
    api: A,
    preflight: LinkTransportPreflightSink,
    state: LinkTransportNativeState,
}

/// Serve one fixed-size ingress frame through a current-session Link pipe.
///
/// This runtime seam is intentionally limited to the Spike C frame boundary: it creates the
/// scoped mutex and ingress server, acknowledges exactly one bounded frame, then closes all
/// owned handles. The caller is responsible for decoding and reducing the returned bytes.
#[cfg(target_env = "msvc")]
pub fn serve_one_ingress_frame(
    names: LinkLocalObjectNames,
    frame_bytes: usize,
) -> Result<Vec<u8>, LinkTransportNativeBackendError> {
    if frame_bytes == 0 || frame_bytes > 8 * 1024 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressFrameSize",
        ));
    }

    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names))?;
    let server = backend.state().ingress_pipe_handle.ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("CreateIngressPipe"),
    )?;

    let mut frame = vec![0_u8; frame_bytes];
    let result = (|| {
        connect_named_pipe_server(server)?;
        read_exact_os(server, &mut frame, "ReadIngressFrame")?;
        write_all_os(server, &[0xAC], "WriteIngressAck")?;
        Ok(frame)
    })();
    let shutdown = backend.close_all();
    if shutdown.handles_remaining != 0 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressShutdownResidue",
        ));
    }
    result
}

/// Send one bounded ingress frame to an already-running Link and require its acknowledgement.
#[cfg(target_env = "msvc")]
pub fn send_ingress_frame_and_wait_ack(
    names: &LinkLocalObjectNames,
    frame: &[u8],
) -> Result<(), LinkTransportNativeBackendError> {
    if frame.is_empty() || frame.len() > 8 * 1024 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressFrameSize",
        ));
    }

    let client = connect_ingress_client_with_retry(&names.ingress_pipe).ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("ConnectIngressClient"),
    )?;
    let result = (|| {
        write_all_os(client, frame, "WriteIngressFrame")?;
        let mut acknowledgement = [0_u8; 1];
        read_exact_os(client, &mut acknowledgement, "ReadIngressAck")?;
        if acknowledgement != [0xAC] {
            return Err(LinkTransportNativeBackendError::NativeCallFailed(
                "IngressAckMismatch",
            ));
        }
        Ok(())
    })();
    if !WindowsSysLinkTransportApi.close_handle(client) {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CloseIngressClient",
        ));
    }
    result
}

/// Serve one framed ingress message and return its validated header plus bounded payload.
#[cfg(target_env = "msvc")]
pub fn serve_one_ingress_message(
    names: LinkLocalObjectNames,
) -> Result<(pulse_link_core::LinkFrameHeader, Vec<u8>), LinkTransportNativeBackendError> {
    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names))?;
    let server = backend.state().ingress_pipe_handle.ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("CreateIngressPipe"),
    )?;
    let result = (|| {
        connect_named_pipe_server(server)?;
        let mut encoded_header = [0_u8; pulse_link_core::FRAME_HEADER_BYTES];
        read_exact_os(server, &mut encoded_header, "ReadIngressHeader")?;
        let header = pulse_link_core::LinkFrameHeader::decode(&encoded_header).map_err(|_| {
            LinkTransportNativeBackendError::NativeCallFailed("DecodeIngressHeader")
        })?;
        let mut payload = vec![0_u8; header.payload_length as usize];
        if !payload.is_empty() {
            read_exact_os(server, &mut payload, "ReadIngressPayload")?;
        }
        write_all_os(server, &[0xAC], "WriteIngressAck")?;
        Ok((header, payload))
    })();
    let shutdown = backend.close_all();
    if shutdown.handles_remaining != 0 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressShutdownResidue",
        ));
    }
    result
}

/// Send one framed ingress message and wait for the Link acknowledgement.
#[cfg(target_env = "msvc")]
pub fn send_ingress_message_and_wait_ack(
    names: &LinkLocalObjectNames,
    header: &pulse_link_core::LinkFrameHeader,
    payload: &[u8],
) -> Result<(), LinkTransportNativeBackendError> {
    if payload.len() != header.payload_length as usize {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressPayloadLength",
        ));
    }
    let client = connect_ingress_client_with_retry(&names.ingress_pipe).ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("ConnectIngressClient"),
    )?;
    let result = (|| {
        write_all_os(client, &header.encode(), "WriteIngressHeader")?;
        if !payload.is_empty() {
            write_all_os(client, payload, "WriteIngressPayload")?;
        }
        let mut acknowledgement = [0_u8; 1];
        read_exact_os(client, &mut acknowledgement, "ReadIngressAck")?;
        if acknowledgement != [0xAC] {
            return Err(LinkTransportNativeBackendError::NativeCallFailed(
                "IngressAckMismatch",
            ));
        }
        Ok(())
    })();
    if !WindowsSysLinkTransportApi.close_handle(client) {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CloseIngressClient",
        ));
    }
    result
}

/// Serve one bounded Island request and return the request bytes after writing a response.
#[cfg(target_env = "msvc")]
pub fn serve_one_island_request_response(
    names: LinkLocalObjectNames,
    request_bytes: usize,
    response: &[u8],
) -> Result<Vec<u8>, LinkTransportNativeBackendError> {
    if request_bytes == 0 || request_bytes > 8 * 1024 || response.len() > 8 * 1024 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IslandMessageSize",
        ));
    }
    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names))?;
    let server = backend.state().island_pipe_handle.ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("CreateIslandPipe"),
    )?;
    let result = (|| {
        connect_named_pipe_server(server)?;
        let mut request = vec![0_u8; request_bytes];
        read_exact_os(server, &mut request, "ReadIslandRequest")?;
        write_all_os(server, response, "WriteIslandResponse")?;
        Ok(request)
    })();
    let shutdown = backend.close_all();
    if shutdown.handles_remaining != 0 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IslandShutdownResidue",
        ));
    }
    result
}

/// Send one bounded Island request and read its bounded response.
#[cfg(target_env = "msvc")]
pub fn send_island_request(
    names: &LinkLocalObjectNames,
    request: &[u8],
    response_bytes: usize,
) -> Result<Vec<u8>, LinkTransportNativeBackendError> {
    if request.is_empty() || request.len() > 8 * 1024 || response_bytes > 8 * 1024 {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IslandMessageSize",
        ));
    }
    let client = connect_ingress_client_with_retry(&names.island_pipe).ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("ConnectIslandClient"),
    )?;
    let result = (|| {
        write_all_os(client, request, "WriteIslandRequest")?;
        let mut response = vec![0_u8; response_bytes];
        read_exact_os(client, &mut response, "ReadIslandResponse")?;
        Ok(response)
    })();
    if !WindowsSysLinkTransportApi.close_handle(client) {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CloseIslandClient",
        ));
    }
    result
}

/// Run the MSVC `windows-sys` Link transport harness against real OS handles.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_os_transport_harness(
    names: LinkLocalObjectNames,
) -> Result<WindowsSysOsTransportHarnessReport, LinkTransportNativeBackendError> {
    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateInheritedHandoffPipe)?;
    backend.apply_command(LinkTransportCommand::ConnectIslandClient(names))?;

    let state = *backend.state();
    let shutdown = backend.close_all();
    Ok(WindowsSysOsTransportHarnessReport {
        mutex_created: state.mutex_handle.is_some(),
        ingress_pipe_created: state.ingress_pipe_handle.is_some(),
        island_pipe_created: state.island_pipe_handle.is_some(),
        handoff_pipe_created: state.handoff_pipe.is_some(),
        island_client_connected: state.island_client_handle.is_some(),
        shutdown,
    })
}

/// Run a real ingress named-pipe frame write/read plus acknowledgement round trip.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_ingress_frame_ack_harness(
    names: LinkLocalObjectNames,
    frame: &[u8],
) -> Result<WindowsSysIngressFrameAckReport, LinkTransportNativeBackendError> {
    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;

    let Some(server) = backend.state().ingress_pipe_handle else {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CreateIngressPipe",
        ));
    };
    let client = connect_named_pipe_client(&names.ingress_pipe).ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("ConnectIngressClient"),
    )?;

    let mut shutdown = LinkTransportShutdownReport::default();
    let round_trip = run_ingress_frame_ack_round_trip(server, client, frame);
    close_harness_handle(client, &mut shutdown);
    merge_shutdown_reports(&mut shutdown, backend.close_all());

    let (frame_bytes_written, frame_bytes_read, ack_bytes_written, ack_bytes_read) = round_trip?;
    Ok(WindowsSysIngressFrameAckReport {
        frame_bytes_written,
        frame_bytes_read,
        ack_bytes_written,
        ack_bytes_read,
        frame_round_tripped: frame_bytes_written == frame_bytes_read,
        ack_round_tripped: ack_bytes_written == 1 && ack_bytes_read == 1,
        shutdown,
    })
}

/// Run a real ingress named-pipe multi-frame write/read plus acknowledgement loop.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_ingress_frame_ack_loop_harness(
    names: LinkLocalObjectNames,
    frames: &[&[u8]],
) -> Result<WindowsSysIngressFrameAckLoopReport, LinkTransportNativeBackendError> {
    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIngressPipe(names.clone()))?;

    let Some(server) = backend.state().ingress_pipe_handle else {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CreateIngressPipe",
        ));
    };
    let client = connect_named_pipe_client(&names.ingress_pipe).ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("ConnectIngressClient"),
    )?;

    let mut shutdown = LinkTransportShutdownReport::default();
    let loop_report = run_ingress_frame_ack_round_trip_loop(server, client, frames);
    close_harness_handle(client, &mut shutdown);
    merge_shutdown_reports(&mut shutdown, backend.close_all());

    let (frame_count, frame_bytes_written, frame_bytes_read, ack_bytes_written, ack_bytes_read) =
        loop_report?;
    Ok(WindowsSysIngressFrameAckLoopReport {
        frame_count,
        frame_bytes_written,
        frame_bytes_read,
        ack_bytes_written,
        ack_bytes_read,
        frames_round_tripped: frame_bytes_written == frame_bytes_read,
        acks_round_tripped: ack_bytes_written == frame_count && ack_bytes_read == frame_count,
        shutdown,
    })
}

/// Run a real Island named-pipe request/response byte loop.
#[cfg(target_env = "msvc")]
pub fn run_windows_sys_island_request_response_loop_harness(
    names: LinkLocalObjectNames,
    requests: &[&[u8]],
    responses: &[&[u8]],
) -> Result<WindowsSysIslandRequestResponseLoopReport, LinkTransportNativeBackendError> {
    let mut backend = LinkTransportNativeBackend::new(WindowsSysLinkTransportApi);
    backend.apply_command(LinkTransportCommand::CreateMutex(names.clone()))?;
    backend.apply_command(LinkTransportCommand::CreateIslandPipe(names.clone()))?;

    let Some(server) = backend.state().island_pipe_handle else {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "CreateIslandPipe",
        ));
    };
    let client = connect_named_pipe_client(&names.island_pipe).ok_or(
        LinkTransportNativeBackendError::NativeCallFailed("ConnectIslandClient"),
    )?;

    let mut shutdown = LinkTransportShutdownReport::default();
    let loop_report = run_request_response_round_trip_loop(server, client, requests, responses);
    close_harness_handle(client, &mut shutdown);
    merge_shutdown_reports(&mut shutdown, backend.close_all());

    let (
        round_trip_count,
        request_bytes_written,
        request_bytes_read,
        response_bytes_written,
        response_bytes_read,
    ) = loop_report?;
    Ok(WindowsSysIslandRequestResponseLoopReport {
        round_trip_count,
        request_bytes_written,
        request_bytes_read,
        response_bytes_written,
        response_bytes_read,
        requests_round_tripped: request_bytes_written == request_bytes_read,
        responses_round_tripped: response_bytes_written == response_bytes_read,
        shutdown,
    })
}

impl<A> LinkTransportNativeBackend<A>
where
    A: LinkTransportNativeApi,
{
    /// Create a backend from a native API implementation.
    pub fn new(api: A) -> Self {
        Self {
            api,
            preflight: LinkTransportPreflightSink::default(),
            state: LinkTransportNativeState::default(),
        }
    }

    /// Current native handle state.
    pub const fn state(&self) -> &LinkTransportNativeState {
        &self.state
    }

    /// Borrow the underlying API adapter for diagnostics/tests.
    pub const fn api(&self) -> &A {
        &self.api
    }

    /// Validate and apply one Link transport command.
    pub fn apply_command(
        &mut self,
        command: LinkTransportCommand,
    ) -> Result<(), LinkTransportNativeBackendError> {
        let previous_preflight = self.preflight.clone();
        self.preflight
            .validate_command(command.clone())
            .map_err(LinkTransportNativeBackendError::Preflight)?;

        if let Err(error) = self.apply_preflighted(command) {
            self.preflight = previous_preflight;
            return Err(error);
        }
        Ok(())
    }

    /// Close all owned native handles in shutdown order, retaining failures for retry.
    pub fn close_all(&mut self) -> LinkTransportShutdownReport {
        let mut report = LinkTransportShutdownReport::default();

        if let Some(handle) = self.state.handoff_write_handle {
            if self.close_one(handle, &mut report) {
                self.state.handoff_write_handle = None;
            }
        }
        if let Some(handle) = self.state.handoff_read_handle {
            if self.close_one(handle, &mut report) {
                self.state.handoff_read_handle = None;
            }
        }
        if self.state.handoff_read_handle.is_none() && self.state.handoff_write_handle.is_none() {
            self.state.handoff_pipe = None;
        }
        if let Some(handle) = self.state.island_client_handle {
            if self.close_one(handle, &mut report) {
                self.state.island_client_handle = None;
            }
        }
        if let Some(handle) = self.state.island_pipe_handle {
            if self.close_one(handle, &mut report) {
                self.state.island_pipe_handle = None;
            }
        }
        if let Some(handle) = self.state.ingress_pipe_handle {
            if self.close_one(handle, &mut report) {
                self.state.ingress_pipe_handle = None;
            }
        }
        if let Some(handle) = self.state.mutex_handle {
            if self.close_one(handle, &mut report) {
                self.state.mutex_handle = None;
            }
        }

        report.handles_remaining = self.state.handle_count();
        report
    }

    fn close_one(
        &mut self,
        handle: RawLinkHandle,
        report: &mut LinkTransportShutdownReport,
    ) -> bool {
        report.close_attempts += 1;
        if self.api.close_handle(handle) {
            report.closed_handles += 1;
            true
        } else {
            report.failed_closes += 1;
            false
        }
    }

    fn apply_preflighted(
        &mut self,
        command: LinkTransportCommand,
    ) -> Result<(), LinkTransportNativeBackendError> {
        match command {
            LinkTransportCommand::CreateMutex(names) => {
                self.state.mutex_handle = Some(self.api.create_mutex(&names).ok_or(
                    LinkTransportNativeBackendError::NativeCallFailed("CreateMutex"),
                )?);
            }
            LinkTransportCommand::CreateIngressPipe(names) => {
                self.state.ingress_pipe_handle = Some(self.api.create_ingress_pipe(&names).ok_or(
                    LinkTransportNativeBackendError::NativeCallFailed("CreateIngressPipe"),
                )?);
            }
            LinkTransportCommand::CreateIslandPipe(names) => {
                self.state.island_pipe_handle = Some(self.api.create_island_pipe(&names).ok_or(
                    LinkTransportNativeBackendError::NativeCallFailed("CreateIslandPipe"),
                )?);
            }
            LinkTransportCommand::CreateInheritedHandoffPipe => {
                let pipe = self.api.create_inherited_handoff_pipe().ok_or(
                    LinkTransportNativeBackendError::NativeCallFailed("CreateInheritedHandoffPipe"),
                )?;
                self.state.handoff_read_handle = Some(pipe.read);
                self.state.handoff_write_handle = Some(pipe.write);
                self.state.handoff_pipe = Some(pipe);
            }
            LinkTransportCommand::ConnectIslandClient(names) => {
                self.state.island_client_handle =
                    Some(self.api.connect_island_client(&names).ok_or(
                        LinkTransportNativeBackendError::NativeCallFailed("ConnectIslandClient"),
                    )?);
            }
        }
        Ok(())
    }
}

/// Native adapter backed by `windows-sys` for MSVC Windows builds.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysLinkTransportApi;

#[cfg(target_env = "msvc")]
impl LinkTransportNativeApi for WindowsSysLinkTransportApi {
    fn create_mutex(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        let name = wide_null_terminated(&names.mutex);
        let handle = unsafe {
            windows_sys::Win32::System::Threading::CreateMutexW(std::ptr::null(), 0, name.as_ptr())
        };
        raw_handle(handle)
    }

    fn create_ingress_pipe(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        create_named_pipe(&names.ingress_pipe)
    }

    fn create_island_pipe(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        create_named_pipe(&names.island_pipe)
    }

    fn create_inherited_handoff_pipe(&mut self) -> Option<InheritedHandoffPipe> {
        let mut security_attributes = windows_sys::Win32::Security::SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<windows_sys::Win32::Security::SECURITY_ATTRIBUTES>()
                as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: 1,
        };
        let mut read = std::ptr::null_mut();
        let mut write = std::ptr::null_mut();
        let created = unsafe {
            windows_sys::Win32::System::Pipes::CreatePipe(
                &mut read,
                &mut write,
                &mut security_attributes,
                0,
            )
        };
        if created == 0 {
            return None;
        }
        Some(InheritedHandoffPipe {
            read: raw_handle(read)?,
            write: raw_handle(write)?,
        })
    }

    fn connect_island_client(&mut self, names: &LinkLocalObjectNames) -> Option<RawLinkHandle> {
        connect_named_pipe_client(&names.island_pipe)
    }

    fn close_handle(&mut self, handle: RawLinkHandle) -> bool {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(windows_handle(handle)) != 0 }
    }
}

#[cfg(target_env = "msvc")]
fn create_named_pipe(name: &str) -> Option<RawLinkHandle> {
    use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
    use windows_sys::Win32::System::Pipes::{
        CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT,
    };

    let name = wide_null_terminated(name);
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1,
            8 * 1024,
            8 * 1024,
            400,
            std::ptr::null(),
        )
    };
    raw_handle(handle)
}

#[cfg(target_env = "msvc")]
fn connect_named_pipe_client(name: &str) -> Option<RawLinkHandle> {
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING,
    };

    let name = wide_null_terminated(name);
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    raw_handle(handle)
}

#[cfg(target_env = "msvc")]
fn connect_ingress_client_with_retry(name: &str) -> Option<RawLinkHandle> {
    for _ in 0..20 {
        if let Some(handle) = connect_named_pipe_client(name) {
            return Some(handle);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    None
}

#[cfg(target_env = "msvc")]
fn connect_named_pipe_server(server: RawLinkHandle) -> Result<(), LinkTransportNativeBackendError> {
    let connected = unsafe {
        windows_sys::Win32::System::Pipes::ConnectNamedPipe(
            windows_handle(server),
            std::ptr::null_mut(),
        )
    };
    if connected != 0
        || unsafe { windows_sys::Win32::Foundation::GetLastError() }
            == windows_sys::Win32::Foundation::ERROR_PIPE_CONNECTED
    {
        return Ok(());
    }
    Err(LinkTransportNativeBackendError::NativeCallFailed(
        "ConnectIngressServer",
    ))
}

#[cfg(target_env = "msvc")]
fn run_ingress_frame_ack_round_trip(
    server: RawLinkHandle,
    client: RawLinkHandle,
    frame: &[u8],
) -> Result<(u32, u32, u32, u32), LinkTransportNativeBackendError> {
    const ACK: [u8; 1] = [0xAC];

    let frame_bytes_written = write_all_os(client, frame, "WriteIngressFrame")?;
    let mut frame_buffer = vec![0_u8; frame.len()];
    let frame_bytes_read = read_exact_os(server, &mut frame_buffer, "ReadIngressFrame")?;
    let ack_bytes_written = write_all_os(server, &ACK, "WriteIngressAck")?;
    let mut ack_buffer = [0_u8; 1];
    let ack_bytes_read = read_exact_os(client, &mut ack_buffer, "ReadIngressAck")?;

    if frame_buffer != frame || ack_buffer != ACK {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "IngressRoundTripMismatch",
        ));
    }
    Ok((
        frame_bytes_written,
        frame_bytes_read,
        ack_bytes_written,
        ack_bytes_read,
    ))
}

#[cfg(target_env = "msvc")]
fn run_ingress_frame_ack_round_trip_loop(
    server: RawLinkHandle,
    client: RawLinkHandle,
    frames: &[&[u8]],
) -> Result<(u32, u32, u32, u32, u32), LinkTransportNativeBackendError> {
    let frame_count = u32::try_from(frames.len())
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("FrameCountOverflow"))?;
    let mut total_frame_bytes_written = 0_u32;
    let mut total_frame_bytes_read = 0_u32;
    let mut total_ack_bytes_written = 0_u32;
    let mut total_ack_bytes_read = 0_u32;

    for frame in frames {
        let (frame_bytes_written, frame_bytes_read, ack_bytes_written, ack_bytes_read) =
            run_ingress_frame_ack_round_trip(server, client, frame)?;
        total_frame_bytes_written = total_frame_bytes_written.saturating_add(frame_bytes_written);
        total_frame_bytes_read = total_frame_bytes_read.saturating_add(frame_bytes_read);
        total_ack_bytes_written = total_ack_bytes_written.saturating_add(ack_bytes_written);
        total_ack_bytes_read = total_ack_bytes_read.saturating_add(ack_bytes_read);
    }

    Ok((
        frame_count,
        total_frame_bytes_written,
        total_frame_bytes_read,
        total_ack_bytes_written,
        total_ack_bytes_read,
    ))
}

#[cfg(target_env = "msvc")]
fn run_request_response_round_trip_loop(
    server: RawLinkHandle,
    client: RawLinkHandle,
    requests: &[&[u8]],
    responses: &[&[u8]],
) -> Result<(u32, u32, u32, u32, u32), LinkTransportNativeBackendError> {
    if requests.len() != responses.len() {
        return Err(LinkTransportNativeBackendError::NativeCallFailed(
            "RequestResponseCountMismatch",
        ));
    }
    let round_trip_count = u32::try_from(requests.len())
        .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("FrameCountOverflow"))?;
    let mut total_request_bytes_written = 0_u32;
    let mut total_request_bytes_read = 0_u32;
    let mut total_response_bytes_written = 0_u32;
    let mut total_response_bytes_read = 0_u32;

    for (request, response) in requests.iter().zip(responses.iter()) {
        let request_bytes_written = write_all_os(client, request, "WriteIslandRequest")?;
        let mut request_buffer = vec![0_u8; request.len()];
        let request_bytes_read = read_exact_os(server, &mut request_buffer, "ReadIslandRequest")?;
        let response_bytes_written = write_all_os(server, response, "WriteIslandResponse")?;
        let mut response_buffer = vec![0_u8; response.len()];
        let response_bytes_read =
            read_exact_os(client, &mut response_buffer, "ReadIslandResponse")?;

        if request_buffer != *request || response_buffer != *response {
            return Err(LinkTransportNativeBackendError::NativeCallFailed(
                "IslandRoundTripMismatch",
            ));
        }

        total_request_bytes_written =
            total_request_bytes_written.saturating_add(request_bytes_written);
        total_request_bytes_read = total_request_bytes_read.saturating_add(request_bytes_read);
        total_response_bytes_written =
            total_response_bytes_written.saturating_add(response_bytes_written);
        total_response_bytes_read = total_response_bytes_read.saturating_add(response_bytes_read);
    }

    Ok((
        round_trip_count,
        total_request_bytes_written,
        total_request_bytes_read,
        total_response_bytes_written,
        total_response_bytes_read,
    ))
}

#[cfg(target_env = "msvc")]
fn write_all_os(
    handle: RawLinkHandle,
    bytes: &[u8],
    error_label: &'static str,
) -> Result<u32, LinkTransportNativeBackendError> {
    let mut total = 0_u32;
    while usize::try_from(total).map_or(true, |written| written < bytes.len()) {
        let offset = usize::try_from(total)
            .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ByteCountOverflow"))?;
        let remaining = &bytes[offset..];
        let chunk_len = u32::try_from(remaining.len())
            .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ByteCountOverflow"))?;
        let mut written = 0_u32;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::WriteFile(
                windows_handle(handle),
                remaining.as_ptr(),
                chunk_len,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 || written == 0 {
            return Err(LinkTransportNativeBackendError::NativeCallFailed(
                error_label,
            ));
        }
        total = total.saturating_add(written);
    }
    Ok(total)
}

#[cfg(target_env = "msvc")]
fn read_exact_os(
    handle: RawLinkHandle,
    buffer: &mut [u8],
    error_label: &'static str,
) -> Result<u32, LinkTransportNativeBackendError> {
    let mut total = 0_u32;
    while usize::try_from(total).map_or(true, |read| read < buffer.len()) {
        let offset = usize::try_from(total)
            .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ByteCountOverflow"))?;
        let remaining = &mut buffer[offset..];
        let chunk_len = u32::try_from(remaining.len())
            .map_err(|_| LinkTransportNativeBackendError::NativeCallFailed("ByteCountOverflow"))?;
        let mut read = 0_u32;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::ReadFile(
                windows_handle(handle),
                remaining.as_mut_ptr(),
                chunk_len,
                &mut read,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 || read == 0 {
            return Err(LinkTransportNativeBackendError::NativeCallFailed(
                error_label,
            ));
        }
        total = total.saturating_add(read);
    }
    Ok(total)
}

#[cfg(target_env = "msvc")]
fn close_harness_handle(handle: RawLinkHandle, report: &mut LinkTransportShutdownReport) {
    report.close_attempts += 1;
    if WindowsSysLinkTransportApi.close_handle(handle) {
        report.closed_handles += 1;
    } else {
        report.failed_closes += 1;
        report.handles_remaining += 1;
    }
}

#[cfg(target_env = "msvc")]
fn merge_shutdown_reports(
    target: &mut LinkTransportShutdownReport,
    source: LinkTransportShutdownReport,
) {
    target.close_attempts = target.close_attempts.saturating_add(source.close_attempts);
    target.closed_handles = target.closed_handles.saturating_add(source.closed_handles);
    target.failed_closes = target.failed_closes.saturating_add(source.failed_closes);
    target.handles_remaining = target
        .handles_remaining
        .saturating_add(source.handles_remaining);
}

#[cfg(target_env = "msvc")]
fn wide_null_terminated(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_env = "msvc")]
fn raw_handle(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<RawLinkHandle> {
    if handle.is_null() || handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        None
    } else {
        RawLinkHandle::new(handle as isize)
    }
}

#[cfg(target_env = "msvc")]
fn windows_handle(handle: RawLinkHandle) -> windows_sys::Win32::Foundation::HANDLE {
    handle.value() as windows_sys::Win32::Foundation::HANDLE
}
