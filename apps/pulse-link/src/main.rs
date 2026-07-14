//! Pulse Link executable entry point.

use std::io::Read;
use std::path::PathBuf;

use pulse_persistence::BreadcrumbStore;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--snapshot") {
        emit_snapshot(breadcrumb_argument(&arguments));
        return;
    }
    if arguments
        .iter()
        .any(|argument| argument == "--handoff-stdin")
    {
        reduce_handoff_stdin(
            arguments.iter().any(|argument| argument == "--diagnostic"),
            breadcrumb_argument(&arguments).or_else(|| {
                arguments
                    .iter()
                    .any(|argument| argument == "--persistent")
                    .then(persistent_breadcrumb_path)
                    .flatten()
            }),
        );
        return;
    }
    if arguments.iter().any(|argument| argument == "--serve-one") {
        #[cfg(target_env = "msvc")]
        serve_one(&arguments);
        return;
    }
    if arguments.iter().any(|argument| argument == "--serve-loop") {
        #[cfg(target_env = "msvc")]
        serve_loop(&arguments);
        return;
    }
    if arguments.iter().any(|argument| argument == "--island-once") {
        #[cfg(target_env = "msvc")]
        serve_island_once(&arguments);
        return;
    }
    if arguments
        .iter()
        .any(|argument| argument == "--island-sequence")
    {
        #[cfg(target_env = "msvc")]
        serve_island_sequence(&arguments);
        return;
    }

    if !arguments
        .iter()
        .any(|argument| argument == "--diagnostic-c1")
    {
        return;
    }

    let report = pulse_link::run_link_scenario(pulse_link::LinkScenario::C1FirstHookWakesLink);
    println!(
        "link_lifecycle={} active_breadcrumbs={} island_attached={} provider_affected={}",
        lifecycle_label(report.lifecycle_state),
        report.active_tasks,
        report.island_attached,
        report.provider_affected
    );
}

fn reduce_handoff_stdin(diagnostic: bool, breadcrumb_path: Option<PathBuf>) {
    let mut frame = [0_u8; pulse_link_core::FRAME_HEADER_BYTES];
    if std::io::stdin().read_exact(&mut frame).is_err() {
        return;
    }
    let Some(header) = pulse_link_core::LinkFrameHeader::decode(&frame).ok() else {
        return;
    };
    let mut payload = vec![0_u8; header.payload_length as usize];
    if !payload.is_empty() && std::io::stdin().read_exact(&mut payload).is_err() {
        return;
    }
    let reduced = match breadcrumb_path {
        Some(path) => {
            let store = pulse_persistence::FileBreadcrumbStore::new(path);
            let mut runtime = pulse_link::LinkRuntime::with_store(store);
            if let Ok(snapshot) = runtime.load_breadcrumbs() {
                let _ = runtime.recover_degraded_from_breadcrumbs(snapshot);
            }
            reduce_payload(&mut runtime, header, &payload)
        }
        None => {
            let mut runtime = pulse_link::LinkRuntime::new();
            reduce_payload(&mut runtime, header, &payload)
        }
    };
    let checkpoint_written = reduced
        .as_ref()
        .is_some_and(|report| report.checkpoint_written);
    let active_breadcrumbs = reduced.map_or(0, |report| report.active_tasks);
    if diagnostic {
        println!("ingress_accepted={checkpoint_written} active_breadcrumbs={active_breadcrumbs}");
    }
}

fn reduce_payload<S: pulse_persistence::BreadcrumbStore>(
    runtime: &mut pulse_link::LinkRuntime<S>,
    header: pulse_link_core::LinkFrameHeader,
    payload: &[u8],
) -> Option<pulse_link::LinkRuntimeReport> {
    if payload.is_empty() {
        return pulse_link::apply_header_only_ingress(runtime, header).ok();
    }
    if header.message_kind != pulse_link_core::LinkMessageKind::HookEnvelope {
        return None;
    }
    let event = pulse_protocol::decode_ingress_payload(payload).ok()?;
    runtime
        .apply_event(event, pulse_domain::PrivacyProfile::Minimal)
        .ok()
}

fn breadcrumb_argument(arguments: &[String]) -> Option<PathBuf> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--breadcrumb")
        .map(|pair| PathBuf::from(&pair[1]))
        .filter(|path| !path.as_os_str().is_empty())
}

fn emit_snapshot(breadcrumb_path: Option<PathBuf>) {
    let Some(path) = breadcrumb_path else {
        return;
    };
    let store = pulse_persistence::FileBreadcrumbStore::new(path);
    let Ok(snapshot) = store.load() else {
        println!("snapshot_active=0 snapshot_recent_terminal=0 snapshot_health=unavailable");
        return;
    };
    println!(
        "snapshot_active={} snapshot_recent_terminal={} snapshot_health=available",
        snapshot.active_tasks.len(),
        snapshot.recent_terminal_tasks.len()
    );
}

fn persistent_breadcrumb_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("PULSE_LINK_STATE_ROOT").filter(|root| !root.is_empty()) {
        return Some(PathBuf::from(root).join("breadcrumbs.snapshot"));
    }
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("PulseIsland").join("breadcrumbs.snapshot"))
}

#[cfg(target_env = "msvc")]
fn serve_one(arguments: &[String]) {
    let Some(scope) = scope_argument(arguments) else {
        return;
    };
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    let result = pulse_win32_link::serve_one_ingress_message(names);
    if let Ok((header, payload)) = result {
        let reduced = if payload.is_empty() {
            let mut runtime = pulse_link::LinkRuntime::new();
            pulse_link::apply_header_only_ingress(&mut runtime, header).ok()
        } else {
            pulse_protocol::decode_ingress_payload(&payload)
                .ok()
                .and_then(|event| {
                    let mut runtime = pulse_link::LinkRuntime::new();
                    runtime
                        .apply_event(event, pulse_domain::PrivacyProfile::Minimal)
                        .ok()
                })
        };
        let checkpoint_written = reduced
            .as_ref()
            .is_some_and(|report| report.checkpoint_written);
        let active_breadcrumbs = reduced.map_or(0, |report| report.active_tasks);
        println!(
            "ingress_accepted={checkpoint_written} active_breadcrumbs={active_breadcrumbs} payload_bytes={}",
            payload.len()
        );
    }
}

#[cfg(target_env = "msvc")]
fn scope_argument(arguments: &[String]) -> Option<&str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--scope")
        .map(|pair| pair[1].as_str())
        .filter(|scope| !scope.is_empty())
}

#[cfg(target_env = "msvc")]
fn serve_island_once(arguments: &[String]) {
    let Some(scope) = scope_argument(arguments) else {
        return;
    };
    let path = breadcrumb_argument(arguments).or_else(persistent_breadcrumb_path);
    let (active, recent) = path
        .and_then(|path| {
            let store = pulse_persistence::FileBreadcrumbStore::new(path);
            store.load().ok()
        })
        .map_or((0_u8, 0_u8), |snapshot| {
            (
                u8::try_from(snapshot.active_tasks.len()).unwrap_or(u8::MAX),
                u8::try_from(snapshot.recent_terminal_tasks.len()).unwrap_or(u8::MAX),
            )
        });
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    let response = [1_u8, active, recent];
    if let Ok(request) = pulse_win32_link::serve_one_island_request_response(names, 1, &response) {
        println!(
            "island_request_accepted={} snapshot_active={} snapshot_recent_terminal={}",
            request == [1],
            active,
            recent
        );
    }
}

#[cfg(target_env = "msvc")]
fn serve_island_sequence(arguments: &[String]) {
    let Some(scope) = scope_argument(arguments) else {
        return;
    };
    let path = breadcrumb_argument(arguments).or_else(persistent_breadcrumb_path);
    let (active, recent) = path
        .and_then(|p| pulse_persistence::FileBreadcrumbStore::new(p).load().ok())
        .map_or((0_u8, 0_u8), |s| {
            (
                u8::try_from(s.active_tasks.len()).unwrap_or(u8::MAX),
                u8::try_from(s.recent_terminal_tasks.len()).unwrap_or(u8::MAX),
            )
        });
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    let mut handled = 0_u32;
    for _ in 0..3 {
        let response = [1_u8, active, recent];
        let Ok(request) =
            pulse_win32_link::serve_one_island_request_response(names.clone(), 1, &response)
        else {
            break;
        };
        if request == [1] || request == [2] || request == [3] {
            handled = handled.saturating_add(1);
        }
    }
    println!("island_sequence_frames={handled} snapshot_active={active} snapshot_recent_terminal={recent}");
}

#[cfg(target_env = "msvc")]
fn frames_argument(arguments: &[String]) -> u32 {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--frames")
        .and_then(|pair| pair[1].parse::<u32>().ok())
        .filter(|frames| *frames > 0 && *frames <= 256)
        .unwrap_or(1)
}

#[cfg(target_env = "msvc")]
fn serve_loop(arguments: &[String]) {
    let Some(scope) = scope_argument(arguments) else {
        return;
    };
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    let mut processed = 0_u32;
    let mut active = 0_usize;
    let persistent = arguments.iter().any(|argument| argument == "--persistent");
    for _ in 0..frames_argument(arguments) {
        let Ok((header, payload)) = pulse_win32_link::serve_one_ingress_message(names.clone())
        else {
            break;
        };
        let reduced = if persistent {
            let Some(path) = persistent_breadcrumb_path() else {
                break;
            };
            let store = pulse_persistence::FileBreadcrumbStore::new(path);
            let mut runtime = pulse_link::LinkRuntime::with_store(store);
            if let Ok(snapshot) = runtime.load_breadcrumbs() {
                let _ = runtime.recover_degraded_from_breadcrumbs(snapshot);
            }
            reduce_payload(&mut runtime, header, &payload)
        } else if payload.is_empty() {
            let mut runtime = pulse_link::LinkRuntime::new();
            pulse_link::apply_header_only_ingress(&mut runtime, header).ok()
        } else {
            let mut runtime = pulse_link::LinkRuntime::new();
            reduce_payload(&mut runtime, header, &payload)
        };
        if let Some(report) = reduced {
            processed = processed.saturating_add(1);
            active = report.active_tasks;
        }
    }
    println!("loop_frames={} active_breadcrumbs={active}", processed);
}

fn lifecycle_label(state: pulse_link_core::LinkLifecycleState) -> &'static str {
    match state {
        pulse_link_core::LinkLifecycleState::NotRunning => "not_running",
        pulse_link_core::LinkLifecycleState::Starting => "starting",
        pulse_link_core::LinkLifecycleState::Warm => "warm",
        pulse_link_core::LinkLifecycleState::Active => "active",
        pulse_link_core::LinkLifecycleState::IslandActive => "island_active",
        pulse_link_core::LinkLifecycleState::DropMode => "drop_mode",
        pulse_link_core::LinkLifecycleState::GracePeriod => "grace_period",
        pulse_link_core::LinkLifecycleState::CheckpointAndExit => "checkpoint_and_exit",
    }
}
