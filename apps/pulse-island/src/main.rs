//! Minimal production Island host entry point.

use std::path::PathBuf;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--diagnostic") {
        emit_diagnostic(state_path(&arguments));
        return;
    }
    if arguments.iter().any(|argument| argument == "--snapshot") {
        emit_snapshot(state_path(&arguments));
        return;
    }
    if arguments
        .iter()
        .any(|argument| argument == "--native-smoke")
    {
        native_smoke();
        return;
    }
    if arguments
        .iter()
        .any(|argument| argument == "--connect-once")
    {
        #[cfg(target_env = "msvc")]
        connect_once(&arguments);
    }
    if arguments
        .iter()
        .any(|argument| argument == "--connect-sequence")
    {
        #[cfg(target_env = "msvc")]
        connect_sequence(&arguments);
    }
}

#[cfg(target_env = "msvc")]
fn native_smoke() {
    use pulse_win32::{
        CompactWindowStylePolicy, GlobalHotkeyPolicy, HitTestLayout, ImmersiveState,
        NativeWindowAdapterDriver, NativeWindowAdapterInput, NativeWindowAdapterPlan, PointPx,
        SizePx, WindowPlacement,
    };
    use pulse_win32_hwnd::{
        CompactWindowClassSpec, HwndMessagePump, HwndMessagePumpBudget, HwndNativeBackend,
        WindowsSysCompactWindowFactory, WindowsSysHwndApi, WindowsSysMessagePump,
    };

    let input = NativeWindowAdapterInput {
        compact_visible: true,
        immersive_state: ImmersiveState::Normal,
        placement: WindowPlacement {
            origin: PointPx { x: 16, y: 16 },
            size: SizePx {
                width: 320,
                height: 56,
            },
        },
        hit_test: HitTestLayout {
            width_px: 320,
            height_px: 56,
            transparent_margin_px: 8,
            drag_grip_width_px: 24,
        },
        style_policy: CompactWindowStylePolicy::default(),
        hotkey_policy: GlobalHotkeyPolicy {
            enabled: false,
            id: 0,
            modifiers: 0,
            virtual_key: 0,
        },
    };
    let plan = NativeWindowAdapterPlan::from_input(input);
    let mut driver = NativeWindowAdapterDriver::default();
    let mut backend = HwndNativeBackend::new(
        WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default()),
        WindowsSysHwndApi,
    );
    let Ok(applied) = driver.apply_plan_commands(plan, &mut backend) else {
        println!("native_smoke=failed stage=apply");
        return;
    };
    let Some(hwnd) = backend.state().compact_hwnd else {
        println!("native_smoke=failed stage=create");
        return;
    };
    let mut pump = WindowsSysMessagePump;
    let pump_report =
        HwndMessagePumpBudget::new(8).map(|budget| pump.drain_pending_for_window(hwnd, budget));
    let destroyed = backend
        .apply_native_command(pulse_win32::NativeWindowAdapterCommand::DestroyCompactWindow)
        .is_ok();
    println!(
        "native_smoke={} applied_commands={} pump_removed={} destroyed={}",
        if destroyed { "passed" } else { "failed" },
        applied.len(),
        pump_report.map_or(0, |report| report.removed_messages),
        destroyed
    );
}

#[cfg(not(target_env = "msvc"))]
fn native_smoke() {
    println!("native_smoke=unsupported_target target_env=msvc_required");
}

fn emit_diagnostic(path: PathBuf) {
    let snapshot = load_snapshot(&path);
    println!(
        "island_host_ready=true transport=windows_named_pipe ui_mode=compact_observe_only state_root_configured={} snapshot_health={} snapshot_active={} snapshot_recent_terminal={}",
        !path.as_os_str().is_empty(),
        snapshot.health,
        snapshot.active,
        snapshot.recent_terminal
    );
}

fn emit_snapshot(path: PathBuf) {
    let snapshot = load_snapshot(&path);
    println!(
        "snapshot_health={} snapshot_active={} snapshot_recent_terminal={}",
        snapshot.health, snapshot.active, snapshot.recent_terminal
    );
}

struct SnapshotSummary {
    health: &'static str,
    active: usize,
    recent_terminal: usize,
}

fn load_snapshot(path: &std::path::Path) -> SnapshotSummary {
    if !path.exists() {
        return SnapshotSummary {
            health: "empty",
            active: 0,
            recent_terminal: 0,
        };
    }
    let store = pulse_persistence::FileBreadcrumbStore::new(path.to_path_buf());
    match pulse_persistence::BreadcrumbStore::load(&store) {
        Ok(snapshot) => SnapshotSummary {
            health: "available",
            active: snapshot.active_tasks.len(),
            recent_terminal: snapshot.recent_terminal_tasks.len(),
        },
        Err(_) => SnapshotSummary {
            health: "unavailable",
            active: 0,
            recent_terminal: 0,
        },
    }
}

fn state_path(arguments: &[String]) -> PathBuf {
    if let Some(path) = arguments
        .windows(2)
        .find(|pair| pair[0] == "--state")
        .map(|pair| PathBuf::from(&pair[1]))
        .filter(|path| !path.as_os_str().is_empty())
    {
        return path;
    }
    if let Some(root) = std::env::var_os("PULSE_LINK_STATE_ROOT").filter(|root| !root.is_empty()) {
        return PathBuf::from(root).join("breadcrumbs.snapshot");
    }
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("PulseIsland").join("breadcrumbs.snapshot"))
        .unwrap_or_else(|| PathBuf::from("PulseIsland/breadcrumbs.snapshot"))
}

#[cfg(target_env = "msvc")]
fn connect_once(arguments: &[String]) {
    let Some(scope) = arguments
        .windows(2)
        .find(|pair| pair[0] == "--scope")
        .map(|pair| pair[1].as_str())
        .filter(|scope| !scope.is_empty())
    else {
        return;
    };
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    if let Ok(response) = pulse_win32_link::send_island_request(&names, &[1], 3) {
        if response.len() == 3 && response[0] == 1 {
            println!(
                "island_host_connected=true snapshot_active={} snapshot_recent_terminal={}",
                response[1], response[2]
            );
        }
    }
}

#[cfg(target_env = "msvc")]
fn connect_sequence(arguments: &[String]) {
    let Some(scope) = arguments
        .windows(2)
        .find(|pair| pair[0] == "--scope")
        .map(|pair| pair[1].as_str())
        .filter(|scope| !scope.is_empty())
    else {
        return;
    };
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    let mut frames = 0_u8;
    for request in [[1_u8], [2_u8], [3_u8]] {
        if let Ok(response) = pulse_win32_link::send_island_request(&names, &request, 3) {
            if response.len() == 3 && response[0] == 1 {
                frames = frames.saturating_add(1);
            }
        }
    }
    println!("island_host_sequence_frames={frames}");
}
