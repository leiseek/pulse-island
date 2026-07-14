//! Contract tests for the future native HWND backend boundary.

use pulse_win32::{
    CompactWindowStylePolicy, GlobalHotkeyPolicy, HitTestLayout, ImmersiveState,
    NativeWindowAdapterCommand, NativeWindowAdapterDriver, NativeWindowAdapterInput,
    NativeWindowAdapterPlan, PointPx, SizePx, Win32HitTestCode, WindowPlacement,
};
use pulse_win32_hwnd::{
    CompactWindowClassSpec, HwndCommandPreflightError, HwndCommandPreflightSink,
    HwndCompactWindowFactory, HwndHitTestBridge, HwndMessagePumpBudget, HwndMessagePumpReport,
    HwndMouseInputBridge, HwndMouseInputEvent, HwndNativeApi, HwndNativeBackend,
    HwndNativeBackendError, HwndPaintBridge, HwndRenderEvent, RawHwnd, UNSAFE_BOUNDARY_CRATE,
};

fn visible_plan() -> NativeWindowAdapterPlan {
    NativeWindowAdapterPlan::from_input(NativeWindowAdapterInput {
        compact_visible: true,
        immersive_state: ImmersiveState::Normal,
        placement: WindowPlacement {
            origin: PointPx { x: 100, y: 80 },
            size: SizePx {
                width: 260,
                height: 64,
            },
        },
        hit_test: HitTestLayout {
            width_px: 260,
            height_px: 64,
            transparent_margin_px: 8,
            drag_grip_width_px: 32,
        },
        style_policy: CompactWindowStylePolicy::default(),
        hotkey_policy: GlobalHotkeyPolicy::palette_default(),
    })
}

#[test]
fn hwnd_preflight_sink_accepts_payload_commands_without_advancing_on_hidden_failures(
) -> Result<(), String> {
    assert_eq!(UNSAFE_BOUNDARY_CRATE, "pulse-win32-hwnd");
    let plan = visible_plan();
    let mut driver = NativeWindowAdapterDriver::default();
    let mut sink = HwndCommandPreflightSink::default();

    let applied = driver
        .apply_plan_commands(plan, &mut sink)
        .map_err(|error| format!("{error:?}"))?;

    assert_eq!(applied, sink.applied_commands());
    assert_eq!(sink.created_windows(), 1);
    assert!(sink.hotkey_registered());
    assert_eq!(driver.state().window_generation, 1);
    Ok(())
}

#[test]
fn hwnd_preflight_sink_rejects_move_resize_before_window_creation() {
    let mut sink = HwndCommandPreflightSink::default();
    let placement = visible_plan().placement;

    assert_eq!(
        sink.validate_command(NativeWindowAdapterCommand::MoveResize(placement)),
        Err(HwndCommandPreflightError::WindowMissing)
    );
    assert!(sink.applied_commands().is_empty());
}

#[test]
fn hwnd_preflight_sink_rejects_invalid_hotkey_payload() {
    let mut sink = HwndCommandPreflightSink::default();

    assert_eq!(
        sink.validate_command(NativeWindowAdapterCommand::RegisterHotkey(
            pulse_win32::Win32HotkeyChord {
                id: 0,
                modifiers: 0,
                virtual_key: 0,
            },
        )),
        Err(HwndCommandPreflightError::InvalidHotkey)
    );
    assert!(!sink.hotkey_registered());
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FakeCompactWindowFactory {
    next_hwnd: Option<RawHwnd>,
    destroyed: Vec<RawHwnd>,
}

impl HwndCompactWindowFactory for FakeCompactWindowFactory {
    fn create_compact_window(&mut self) -> Option<RawHwnd> {
        self.next_hwnd.take()
    }

    fn destroy_compact_window(&mut self, hwnd: RawHwnd) -> bool {
        self.destroyed.push(hwnd);
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FakeNativeCall {
    ApplyWindowStyles(RawHwnd),
    UpdateHitTestLayout(RawHwnd),
    MoveResize(RawHwnd),
    RegisterHotkey(RawHwnd, i32),
    ShowNoActivate(RawHwnd),
    SetTopmost(RawHwnd),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FakeNativeApi {
    calls: Vec<FakeNativeCall>,
}

impl HwndNativeApi for FakeNativeApi {
    fn apply_window_styles(
        &mut self,
        hwnd: RawHwnd,
        _style_bits: pulse_win32::Win32StyleBits,
    ) -> bool {
        self.calls.push(FakeNativeCall::ApplyWindowStyles(hwnd));
        true
    }

    fn update_hit_test_layout(
        &mut self,
        hwnd: RawHwnd,
        _hit_test: pulse_win32::HitTestLayout,
    ) -> bool {
        self.calls.push(FakeNativeCall::UpdateHitTestLayout(hwnd));
        true
    }

    fn move_resize(&mut self, hwnd: RawHwnd, _placement: WindowPlacement) -> bool {
        self.calls.push(FakeNativeCall::MoveResize(hwnd));
        true
    }

    fn show_no_activate(&mut self, hwnd: RawHwnd) -> bool {
        self.calls.push(FakeNativeCall::ShowNoActivate(hwnd));
        true
    }

    fn hide_compact_window(&mut self, _hwnd: RawHwnd) -> bool {
        true
    }

    fn set_topmost(&mut self, hwnd: RawHwnd) -> bool {
        self.calls.push(FakeNativeCall::SetTopmost(hwnd));
        true
    }

    fn clear_topmost(&mut self, _hwnd: RawHwnd) -> bool {
        true
    }

    fn register_hotkey(&mut self, hwnd: RawHwnd, hotkey: pulse_win32::Win32HotkeyChord) -> bool {
        self.calls
            .push(FakeNativeCall::RegisterHotkey(hwnd, hotkey.id));
        true
    }

    fn unregister_hotkey(&mut self, _hwnd: RawHwnd, _hotkey_id: i32) -> bool {
        true
    }
}

fn raw_hwnd(value: isize) -> Result<RawHwnd, String> {
    RawHwnd::new(value).ok_or_else(|| "raw HWND test value must be non-zero".to_owned())
}

#[test]
fn compact_window_class_spec_is_content_free_and_null_terminated() {
    let spec = CompactWindowClassSpec::default();

    assert_eq!(spec.class_name(), "PulseIslandCompactWindow");
    assert_eq!(spec.window_title(), "Pulse Island");
    assert!(spec.class_name_wide().ends_with(&[0]));
    assert!(spec.window_title_wide().ends_with(&[0]));
    assert!(!spec.class_name_wide()[..spec.class_name_wide().len() - 1].contains(&0));
    assert!(!spec.window_title_wide()[..spec.window_title_wide().len() - 1].contains(&0));
}

#[test]
fn hwnd_message_pump_budget_and_report_are_bounded_and_content_free() -> Result<(), String> {
    let budget =
        HwndMessagePumpBudget::new(16).ok_or_else(|| "expected non-zero budget".to_owned())?;

    assert_eq!(budget.max_messages(), 16);
    assert_eq!(HwndMessagePumpBudget::new(0), None);
    assert_eq!(
        HwndMessagePumpReport::default()
            .record_removed_message()
            .record_dispatch_call(),
        HwndMessagePumpReport {
            removed_messages: 1,
            dispatch_calls: 1,
        }
    );
    Ok(())
}

#[test]
fn hwnd_hit_test_bridge_maps_client_points_to_win32_codes() {
    let mut bridge = HwndHitTestBridge::default();
    assert_eq!(
        bridge.nchittest_for_client_point(PointPx { x: 16, y: 20 }),
        None
    );

    bridge.update_layout(visible_plan().hit_test);

    assert_eq!(
        bridge.nchittest_for_client_point(PointPx { x: 2, y: 20 }),
        Some(Win32HitTestCode::Transparent.value())
    );
    assert_eq!(
        bridge.nchittest_for_client_point(PointPx { x: 16, y: 20 }),
        Some(Win32HitTestCode::Caption.value())
    );
    assert_eq!(
        bridge.nchittest_for_client_point(PointPx { x: 80, y: 20 }),
        Some(Win32HitTestCode::Client.value())
    );
}

#[test]
fn hwnd_mouse_input_bridge_dispatches_only_content_free_client_clicks() {
    let mut bridge = HwndMouseInputBridge::default();
    assert_eq!(
        bridge.event_for_left_button_up(PointPx { x: 80, y: 20 }),
        None
    );

    bridge.update_layout(visible_plan().hit_test);

    assert_eq!(
        bridge.event_for_left_button_up(PointPx { x: 80, y: 20 }),
        Some(HwndMouseInputEvent::CompactPrimaryClick)
    );
    assert_eq!(
        bridge.event_for_left_button_up(PointPx { x: 2, y: 20 }),
        None
    );
    assert_eq!(
        bridge.event_for_left_button_up(PointPx { x: 16, y: 20 }),
        None
    );
    assert_eq!(
        bridge.event_for_left_button_up(PointPx { x: 300, y: 20 }),
        None
    );
}

#[test]
fn hwnd_paint_bridge_emits_content_free_repaint_request() {
    let bridge = HwndPaintBridge;

    assert_eq!(
        bridge.event_for_paint(),
        HwndRenderEvent::CompactRepaintRequested
    );
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_compact_factory_creates_and_destroys_hidden_compact_hwnd() -> Result<(), String> {
    use pulse_win32_hwnd::WindowsSysCompactWindowFactory;

    let mut factory = WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default());
    let hwnd = factory
        .create_compact_window()
        .ok_or_else(|| "failed to create compact HWND".to_owned())?;

    assert_ne!(hwnd.value(), 0);
    assert!(factory.destroy_compact_window(hwnd));
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_wndproc_returns_hit_test_codes_from_cached_layout() -> Result<(), String> {
    use pulse_win32_hwnd::{WindowsSysCompactWindowFactory, WindowsSysHwndApi};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_NCHITTEST};

    let mut factory = WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default());
    let hwnd = factory
        .create_compact_window()
        .ok_or_else(|| "failed to create compact HWND".to_owned())?;
    let mut api = WindowsSysHwndApi;
    assert!(api.update_hit_test_layout(hwnd, visible_plan().hit_test));
    assert!(api.move_resize(hwnd, visible_plan().placement));

    let drag = unsafe {
        SendMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_NCHITTEST,
            0,
            screen_lparam(116, 100),
        )
    };
    let margin = unsafe {
        SendMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_NCHITTEST,
            0,
            screen_lparam(102, 100),
        )
    };

    assert_eq!(drag, Win32HitTestCode::Caption.value() as isize);
    assert_eq!(margin, Win32HitTestCode::Transparent.value() as isize);
    assert!(factory.destroy_compact_window(hwnd));
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_wndproc_rejects_mouse_activation_for_compact_hwnd() -> Result<(), String> {
    use pulse_win32_hwnd::WindowsSysCompactWindowFactory;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageW, MA_NOACTIVATE, WM_LBUTTONDOWN, WM_MOUSEACTIVATE,
    };

    let mut factory = WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default());
    let hwnd = factory
        .create_compact_window()
        .ok_or_else(|| "failed to create compact HWND".to_owned())?;

    let mouse_activate = unsafe {
        SendMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_MOUSEACTIVATE,
            0,
            WM_LBUTTONDOWN as isize,
        )
    };

    assert_eq!(mouse_activate, MA_NOACTIVATE as isize);
    assert!(factory.destroy_compact_window(hwnd));
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_wndproc_dispatches_content_free_paint_request() -> Result<(), String> {
    use pulse_win32_hwnd::{WindowsSysCompactWindowFactory, WindowsSysHwndRenderQueue};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_PAINT};

    let mut factory = WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default());
    let hwnd = factory
        .create_compact_window()
        .ok_or_else(|| "failed to create compact HWND".to_owned())?;

    let paint = unsafe {
        SendMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_PAINT,
            0,
            0,
        )
    };

    assert_eq!(paint, 0);
    assert_eq!(
        WindowsSysHwndRenderQueue::drain(hwnd),
        vec![HwndRenderEvent::CompactRepaintRequested]
    );
    assert!(factory.destroy_compact_window(hwnd));
    Ok(())
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_wndproc_dispatches_content_free_client_mouse_click() -> Result<(), String> {
    use pulse_win32_hwnd::{
        WindowsSysCompactWindowFactory, WindowsSysHwndApi, WindowsSysHwndInputQueue,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_LBUTTONUP};

    let mut factory = WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default());
    let hwnd = factory
        .create_compact_window()
        .ok_or_else(|| "failed to create compact HWND".to_owned())?;
    let mut api = WindowsSysHwndApi;
    assert!(api.update_hit_test_layout(hwnd, visible_plan().hit_test));
    assert!(api.move_resize(hwnd, visible_plan().placement));

    let client_click = unsafe {
        SendMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_LBUTTONUP,
            0,
            client_lparam(80, 20),
        )
    };
    let transparent_click = unsafe {
        SendMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_LBUTTONUP,
            0,
            client_lparam(2, 20),
        )
    };

    assert_eq!(client_click, 0);
    assert_eq!(transparent_click, 0);
    assert_eq!(
        WindowsSysHwndInputQueue::drain(hwnd),
        vec![HwndMouseInputEvent::CompactPrimaryClick]
    );
    assert!(factory.destroy_compact_window(hwnd));
    Ok(())
}

#[cfg(target_env = "msvc")]
fn screen_lparam(x: i16, y: i16) -> isize {
    let x = u16::from_ne_bytes(x.to_ne_bytes()) as u32;
    let y = u16::from_ne_bytes(y.to_ne_bytes()) as u32;
    ((y << 16) | x) as isize
}

#[cfg(target_env = "msvc")]
fn client_lparam(x: i16, y: i16) -> isize {
    screen_lparam(x, y)
}

#[cfg(target_env = "msvc")]
#[test]
fn windows_sys_message_pump_drains_posted_window_message_without_blocking() -> Result<(), String> {
    use pulse_win32_hwnd::{
        HwndMessagePump, WindowsSysCompactWindowFactory, WindowsSysMessagePump,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_NULL};

    let mut factory = WindowsSysCompactWindowFactory::new(CompactWindowClassSpec::default());
    let hwnd = factory
        .create_compact_window()
        .ok_or_else(|| "failed to create compact HWND".to_owned())?;
    let posted = unsafe {
        PostMessageW(
            hwnd.value() as windows_sys::Win32::Foundation::HWND,
            WM_NULL,
            0,
            0,
        )
    };
    assert_ne!(posted, 0);

    let mut pump = WindowsSysMessagePump::default();
    let report = pump.drain_pending_for_window(
        hwnd,
        HwndMessagePumpBudget::new(8).ok_or_else(|| "missing pump budget".to_owned())?,
    );

    assert!(report.removed_messages >= 1);
    assert!(report.dispatch_calls >= 1);
    assert!(factory.destroy_compact_window(hwnd));
    Ok(())
}

#[test]
fn hwnd_native_backend_calls_native_api_after_preflight_with_created_hwnd() -> Result<(), String> {
    let hwnd = raw_hwnd(42)?;
    let plan = visible_plan();
    let mut driver = NativeWindowAdapterDriver::default();
    let factory = FakeCompactWindowFactory {
        next_hwnd: Some(hwnd),
        destroyed: Vec::new(),
    };
    let mut backend = HwndNativeBackend::new(factory, FakeNativeApi::default());

    let applied = driver
        .apply_plan_commands(plan, &mut backend)
        .map_err(|error| format!("{error:?}"))?;

    assert_eq!(
        applied,
        vec![
            NativeWindowAdapterCommand::CreateCompactWindow,
            NativeWindowAdapterCommand::ApplyWindowStyles(plan.style_bits),
            NativeWindowAdapterCommand::UpdateHitTestLayout(plan.hit_test),
            NativeWindowAdapterCommand::MoveResize(plan.placement),
            NativeWindowAdapterCommand::RegisterHotkey(
                GlobalHotkeyPolicy::palette_default()
                    .registration_chord()
                    .ok_or_else(|| "missing default hotkey".to_owned())?
            ),
            NativeWindowAdapterCommand::ShowNoActivate,
            NativeWindowAdapterCommand::SetTopmost,
        ]
    );
    assert_eq!(backend.state().compact_hwnd, Some(hwnd));
    assert_eq!(
        backend.api().calls,
        vec![
            FakeNativeCall::ApplyWindowStyles(hwnd),
            FakeNativeCall::UpdateHitTestLayout(hwnd),
            FakeNativeCall::MoveResize(hwnd),
            FakeNativeCall::RegisterHotkey(hwnd, 1),
            FakeNativeCall::ShowNoActivate(hwnd),
            FakeNativeCall::SetTopmost(hwnd),
        ]
    );
    assert_eq!(driver.state().window_generation, 1);
    Ok(())
}

#[test]
fn hwnd_native_backend_rejects_native_command_before_window_without_calling_api(
) -> Result<(), String> {
    let mut backend = HwndNativeBackend::new(
        FakeCompactWindowFactory {
            next_hwnd: Some(raw_hwnd(9)?),
            destroyed: Vec::new(),
        },
        FakeNativeApi::default(),
    );

    assert_eq!(
        backend.apply_native_command(NativeWindowAdapterCommand::ShowNoActivate),
        Err(HwndNativeBackendError::Preflight(
            HwndCommandPreflightError::WindowMissing
        ))
    );
    assert!(backend.api().calls.is_empty());
    assert_eq!(backend.state().compact_hwnd, None);
    Ok(())
}
