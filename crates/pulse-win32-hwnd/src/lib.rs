//! Native HWND backend boundary for Pulse Island.
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::num::NonZeroIsize;

use pulse_win32::{
    HitTarget, HitTestLayout, NativeWindowAdapterCommand, NativeWindowCommandSink, PointPx, SizePx,
    Win32HitTestCode, Win32HotkeyChord, Win32StyleBits, WindowPlacement,
};

/// The only crate intended to contain future unsafe HWND FFI calls.
pub const UNSAFE_BOUNDARY_CRATE: &str = "pulse-win32-hwnd";

/// Content-free class/title metadata for the compact Island HWND.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactWindowClassSpec {
    class_name: &'static str,
    window_title: &'static str,
    class_name_wide: Vec<u16>,
    window_title_wide: Vec<u16>,
}

impl Default for CompactWindowClassSpec {
    fn default() -> Self {
        Self::new("PulseIslandCompactWindow", "Pulse Island")
    }
}

impl CompactWindowClassSpec {
    /// Create a class spec from static, content-free metadata.
    pub fn new(class_name: &'static str, window_title: &'static str) -> Self {
        Self {
            class_name,
            window_title,
            class_name_wide: wide_null_terminated(class_name),
            window_title_wide: wide_null_terminated(window_title),
        }
    }

    /// Win32 class name.
    pub const fn class_name(&self) -> &'static str {
        self.class_name
    }

    /// Content-free Win32 window title.
    pub const fn window_title(&self) -> &'static str {
        self.window_title
    }

    /// Null-terminated UTF-16 class name.
    pub fn class_name_wide(&self) -> &[u16] {
        &self.class_name_wide
    }

    /// Null-terminated UTF-16 window title.
    pub fn window_title_wide(&self) -> &[u16] {
        &self.window_title_wide
    }
}

fn wide_null_terminated(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Preflight failure before a command is allowed to reach HWND FFI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HwndCommandPreflightError {
    /// Command requires a compact window that has not been created.
    WindowMissing,
    /// Command attempted to create a second compact window.
    WindowAlreadyCreated,
    /// Move/resize payload has a non-positive dimension.
    InvalidPlacement,
    /// Hotkey payload is incomplete or invalid.
    InvalidHotkey,
}

/// Non-null raw HWND value kept out of provider-neutral crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawHwnd(NonZeroIsize);

impl RawHwnd {
    /// Create a raw HWND wrapper from a non-zero platform handle value.
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

/// Factory responsible for creating and destroying the compact HWND.
pub trait HwndCompactWindowFactory {
    /// Create the compact Island window and return its HWND.
    fn create_compact_window(&mut self) -> Option<RawHwnd>;

    /// Destroy a previously created compact Island window.
    fn destroy_compact_window(&mut self, hwnd: RawHwnd) -> bool;
}

/// Non-zero budget for nonblocking HWND message pumping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HwndMessagePumpBudget {
    max_messages: u32,
}

impl HwndMessagePumpBudget {
    /// Create a bounded pump budget.
    pub const fn new(max_messages: u32) -> Option<Self> {
        if max_messages == 0 {
            None
        } else {
            Some(Self { max_messages })
        }
    }

    /// Maximum messages to remove in one nonblocking drain.
    pub const fn max_messages(self) -> u32 {
        self.max_messages
    }
}

/// Content-free result of a nonblocking HWND message-pump drain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HwndMessagePumpReport {
    /// Number of messages removed from the queue.
    pub removed_messages: u32,
    /// Number of dispatch calls made.
    pub dispatch_calls: u32,
}

impl HwndMessagePumpReport {
    /// Return a report with one additional removed message.
    pub const fn record_removed_message(mut self) -> Self {
        self.removed_messages = self.removed_messages.saturating_add(1);
        self
    }

    /// Return a report with one additional dispatch call.
    pub const fn record_dispatch_call(mut self) -> Self {
        self.dispatch_calls = self.dispatch_calls.saturating_add(1);
        self
    }
}

/// Nonblocking message pump for an existing compact HWND.
pub trait HwndMessagePump {
    /// Drain currently pending messages for one HWND up to the provided budget.
    fn drain_pending_for_window(
        &mut self,
        hwnd: RawHwnd,
        budget: HwndMessagePumpBudget,
    ) -> HwndMessagePumpReport;
}

/// Content-free bridge from cached compact HWND geometry to `WM_NCHITTEST` codes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HwndHitTestBridge {
    layout: Option<HitTestLayout>,
}

impl HwndHitTestBridge {
    /// Update the cached hit-test layout for a compact HWND.
    pub const fn update_layout(&mut self, layout: HitTestLayout) {
        self.layout = Some(layout);
    }

    /// Convert a client point into a Win32 `WM_NCHITTEST` result code.
    pub fn nchittest_for_client_point(self, point: PointPx) -> Option<i32> {
        self.layout
            .map(|layout| Win32HitTestCode::from_target(layout.hit_test(point)).value())
    }
}

/// Content-free mouse input event emitted by the compact HWND boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwndMouseInputEvent {
    /// Primary click on the compact Island client body.
    CompactPrimaryClick,
}

/// Content-free bridge from cached compact HWND geometry to mouse input events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HwndMouseInputBridge {
    layout: Option<HitTestLayout>,
}

impl HwndMouseInputBridge {
    /// Update the cached hit-test layout for mouse input dispatch.
    pub const fn update_layout(&mut self, layout: HitTestLayout) {
        self.layout = Some(layout);
    }

    /// Convert a client-coordinate left-button release into a content-free input event.
    pub fn event_for_left_button_up(self, point: PointPx) -> Option<HwndMouseInputEvent> {
        if self.layout?.hit_test(point) == HitTarget::Client {
            Some(HwndMouseInputEvent::CompactPrimaryClick)
        } else {
            None
        }
    }
}

/// Content-free render event emitted by the compact HWND boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwndRenderEvent {
    /// The compact Island HWND received a repaint request.
    CompactRepaintRequested,
}

/// Content-free bridge from `WM_PAINT` to render readiness events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HwndPaintBridge;

impl HwndPaintBridge {
    /// Convert a paint message into a content-free render event.
    pub const fn event_for_paint(self) -> HwndRenderEvent {
        HwndRenderEvent::CompactRepaintRequested
    }
}

/// Native Win32 calls needed by the W2 command backend after preflight.
pub trait HwndNativeApi {
    /// Apply style and extended-style bits to an existing HWND.
    fn apply_window_styles(&mut self, hwnd: RawHwnd, style_bits: Win32StyleBits) -> bool;

    /// Store or update the hit-test layout associated with an existing HWND.
    fn update_hit_test_layout(&mut self, hwnd: RawHwnd, hit_test: HitTestLayout) -> bool;

    /// Move and resize an existing HWND.
    fn move_resize(&mut self, hwnd: RawHwnd, placement: WindowPlacement) -> bool;

    /// Show an existing HWND without activation.
    fn show_no_activate(&mut self, hwnd: RawHwnd) -> bool;

    /// Hide an existing HWND.
    fn hide_compact_window(&mut self, hwnd: RawHwnd) -> bool;

    /// Keep an existing HWND above ordinary windows.
    fn set_topmost(&mut self, hwnd: RawHwnd) -> bool;

    /// Remove topmost placement from an existing HWND.
    fn clear_topmost(&mut self, hwnd: RawHwnd) -> bool;

    /// Register the Palette global hotkey for an existing HWND.
    fn register_hotkey(&mut self, hwnd: RawHwnd, hotkey: Win32HotkeyChord) -> bool;

    /// Unregister the Palette global hotkey for an existing HWND.
    fn unregister_hotkey(&mut self, hwnd: RawHwnd, hotkey_id: i32) -> bool;
}

/// Native HWND backend state owned only by the HWND boundary crate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HwndNativeBackendState {
    /// Current compact Island HWND, when created.
    pub compact_hwnd: Option<RawHwnd>,
    /// Currently registered Palette hotkey id, when registered.
    pub registered_hotkey_id: Option<i32>,
}

/// Error returned by the native HWND backend boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HwndNativeBackendError {
    /// Preflight rejected the command before any native API call.
    Preflight(HwndCommandPreflightError),
    /// The compact window factory failed to create an HWND.
    CreateCompactWindowFailed,
    /// The compact window factory failed to destroy an HWND.
    DestroyCompactWindowFailed,
    /// A command needed an HWND, but none was available in backend state.
    WindowMissing,
    /// A native API call returned failure.
    NativeCallFailed(&'static str),
}

/// Safe command executor that gates native HWND calls behind preflight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HwndNativeBackend<F, A> {
    factory: F,
    api: A,
    preflight: HwndCommandPreflightSink,
    state: HwndNativeBackendState,
}

impl<F, A> HwndNativeBackend<F, A>
where
    F: HwndCompactWindowFactory,
    A: HwndNativeApi,
{
    /// Create a native backend executor from a compact-window factory and API adapter.
    pub fn new(factory: F, api: A) -> Self {
        Self {
            factory,
            api,
            preflight: HwndCommandPreflightSink::default(),
            state: HwndNativeBackendState::default(),
        }
    }

    /// Current backend-owned HWND state.
    pub const fn state(&self) -> &HwndNativeBackendState {
        &self.state
    }

    /// Borrow the underlying native API adapter for diagnostics and tests.
    pub const fn api(&self) -> &A {
        &self.api
    }

    /// Validate and apply one payload command to the native backend.
    pub fn apply_native_command(
        &mut self,
        command: NativeWindowAdapterCommand,
    ) -> Result<(), HwndNativeBackendError> {
        let previous_preflight = self.preflight.clone();
        self.preflight
            .validate_command(command)
            .map_err(HwndNativeBackendError::Preflight)?;

        if let Err(error) = self.apply_preflighted_native_command(command) {
            self.preflight = previous_preflight;
            return Err(error);
        }

        Ok(())
    }

    fn apply_preflighted_native_command(
        &mut self,
        command: NativeWindowAdapterCommand,
    ) -> Result<(), HwndNativeBackendError> {
        match command {
            NativeWindowAdapterCommand::CreateCompactWindow => self.create_compact_window(),
            NativeWindowAdapterCommand::DestroyCompactWindow => self.destroy_compact_window(),
            NativeWindowAdapterCommand::ApplyWindowStyles(style_bits) => {
                let hwnd = self.compact_hwnd()?;
                native_call(
                    self.api.apply_window_styles(hwnd, style_bits),
                    "ApplyWindowStyles",
                )
            }
            NativeWindowAdapterCommand::UpdateHitTestLayout(hit_test) => {
                let hwnd = self.compact_hwnd()?;
                native_call(
                    self.api.update_hit_test_layout(hwnd, hit_test),
                    "UpdateHitTestLayout",
                )
            }
            NativeWindowAdapterCommand::MoveResize(placement) => {
                let hwnd = self.compact_hwnd()?;
                native_call(self.api.move_resize(hwnd, placement), "MoveResize")
            }
            NativeWindowAdapterCommand::ShowNoActivate => {
                let hwnd = self.compact_hwnd()?;
                native_call(self.api.show_no_activate(hwnd), "ShowNoActivate")
            }
            NativeWindowAdapterCommand::HideCompactWindow => {
                let hwnd = self.compact_hwnd()?;
                native_call(self.api.hide_compact_window(hwnd), "HideCompactWindow")
            }
            NativeWindowAdapterCommand::SetTopmost => {
                let hwnd = self.compact_hwnd()?;
                native_call(self.api.set_topmost(hwnd), "SetTopmost")
            }
            NativeWindowAdapterCommand::ClearTopmost => {
                let hwnd = self.compact_hwnd()?;
                native_call(self.api.clear_topmost(hwnd), "ClearTopmost")
            }
            NativeWindowAdapterCommand::RegisterHotkey(hotkey) => {
                let hwnd = self.compact_hwnd()?;
                native_call(self.api.register_hotkey(hwnd, hotkey), "RegisterHotKey")?;
                self.state.registered_hotkey_id = Some(hotkey.id);
                Ok(())
            }
            NativeWindowAdapterCommand::UnregisterHotkey => self.unregister_hotkey(),
        }
    }

    fn create_compact_window(&mut self) -> Result<(), HwndNativeBackendError> {
        let hwnd = self
            .factory
            .create_compact_window()
            .ok_or(HwndNativeBackendError::CreateCompactWindowFailed)?;
        self.state.compact_hwnd = Some(hwnd);
        Ok(())
    }

    fn destroy_compact_window(&mut self) -> Result<(), HwndNativeBackendError> {
        let hwnd = self.compact_hwnd()?;
        if self.factory.destroy_compact_window(hwnd) {
            self.state.compact_hwnd = None;
            self.state.registered_hotkey_id = None;
            Ok(())
        } else {
            Err(HwndNativeBackendError::DestroyCompactWindowFailed)
        }
    }

    fn unregister_hotkey(&mut self) -> Result<(), HwndNativeBackendError> {
        let Some(hotkey_id) = self.state.registered_hotkey_id else {
            return Ok(());
        };
        let hwnd = self.compact_hwnd()?;
        native_call(
            self.api.unregister_hotkey(hwnd, hotkey_id),
            "UnregisterHotKey",
        )?;
        self.state.registered_hotkey_id = None;
        Ok(())
    }

    fn compact_hwnd(&self) -> Result<RawHwnd, HwndNativeBackendError> {
        self.state
            .compact_hwnd
            .ok_or(HwndNativeBackendError::WindowMissing)
    }
}

impl<F, A> NativeWindowCommandSink for HwndNativeBackend<F, A>
where
    F: HwndCompactWindowFactory,
    A: HwndNativeApi,
{
    fn apply_command(&mut self, command: NativeWindowAdapterCommand) -> bool {
        self.apply_native_command(command).is_ok()
    }
}

fn native_call(passed: bool, name: &'static str) -> Result<(), HwndNativeBackendError> {
    if passed {
        Ok(())
    } else {
        Err(HwndNativeBackendError::NativeCallFailed(name))
    }
}

/// Safe preflight sink for future HWND command execution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HwndCommandPreflightSink {
    compact_window_created: bool,
    created_windows: u32,
    hotkey_registered: bool,
    applied_commands: Vec<NativeWindowAdapterCommand>,
}

impl HwndCommandPreflightSink {
    /// Validate and record a command before it can reach the HWND backend.
    pub fn validate_command(
        &mut self,
        command: NativeWindowAdapterCommand,
    ) -> Result<(), HwndCommandPreflightError> {
        self.validate(command)?;
        self.apply_validated(command);
        Ok(())
    }

    /// Commands accepted by this preflight sink.
    pub fn applied_commands(&self) -> Vec<NativeWindowAdapterCommand> {
        self.applied_commands.clone()
    }

    /// Number of compact window creations accepted by this sink.
    pub const fn created_windows(&self) -> u32 {
        self.created_windows
    }

    /// Whether the Palette hotkey is registered according to accepted commands.
    pub const fn hotkey_registered(&self) -> bool {
        self.hotkey_registered
    }

    fn validate(
        &self,
        command: NativeWindowAdapterCommand,
    ) -> Result<(), HwndCommandPreflightError> {
        match command {
            NativeWindowAdapterCommand::CreateCompactWindow => {
                if self.compact_window_created {
                    Err(HwndCommandPreflightError::WindowAlreadyCreated)
                } else {
                    Ok(())
                }
            }
            NativeWindowAdapterCommand::DestroyCompactWindow
            | NativeWindowAdapterCommand::ApplyWindowStyles(_)
            | NativeWindowAdapterCommand::UpdateHitTestLayout(_)
            | NativeWindowAdapterCommand::MoveResize(_)
            | NativeWindowAdapterCommand::ShowNoActivate
            | NativeWindowAdapterCommand::HideCompactWindow
            | NativeWindowAdapterCommand::SetTopmost
            | NativeWindowAdapterCommand::ClearTopmost => {
                self.require_window()?;
                if let NativeWindowAdapterCommand::MoveResize(placement) = command {
                    validate_placement(placement)?;
                }
                Ok(())
            }
            NativeWindowAdapterCommand::RegisterHotkey(hotkey) => validate_hotkey(hotkey),
            NativeWindowAdapterCommand::UnregisterHotkey => Ok(()),
        }
    }

    fn require_window(&self) -> Result<(), HwndCommandPreflightError> {
        if self.compact_window_created {
            Ok(())
        } else {
            Err(HwndCommandPreflightError::WindowMissing)
        }
    }

    fn apply_validated(&mut self, command: NativeWindowAdapterCommand) {
        match command {
            NativeWindowAdapterCommand::CreateCompactWindow => {
                self.compact_window_created = true;
                self.created_windows = self.created_windows.saturating_add(1);
            }
            NativeWindowAdapterCommand::DestroyCompactWindow => {
                self.compact_window_created = false;
            }
            NativeWindowAdapterCommand::RegisterHotkey(_) => {
                self.hotkey_registered = true;
            }
            NativeWindowAdapterCommand::UnregisterHotkey => {
                self.hotkey_registered = false;
            }
            NativeWindowAdapterCommand::ApplyWindowStyles(_)
            | NativeWindowAdapterCommand::UpdateHitTestLayout(_)
            | NativeWindowAdapterCommand::MoveResize(_)
            | NativeWindowAdapterCommand::ShowNoActivate
            | NativeWindowAdapterCommand::HideCompactWindow
            | NativeWindowAdapterCommand::SetTopmost
            | NativeWindowAdapterCommand::ClearTopmost => {}
        }
        self.applied_commands.push(command);
    }
}

impl NativeWindowCommandSink for HwndCommandPreflightSink {
    fn apply_command(&mut self, command: NativeWindowAdapterCommand) -> bool {
        self.validate_command(command).is_ok()
    }
}

fn validate_placement(placement: WindowPlacement) -> Result<(), HwndCommandPreflightError> {
    validate_size(placement.size)
}

fn validate_size(size: SizePx) -> Result<(), HwndCommandPreflightError> {
    if size.width > 0 && size.height > 0 {
        Ok(())
    } else {
        Err(HwndCommandPreflightError::InvalidPlacement)
    }
}

fn validate_hotkey(hotkey: Win32HotkeyChord) -> Result<(), HwndCommandPreflightError> {
    if hotkey.id > 0 && hotkey.modifiers != 0 && hotkey.virtual_key != 0 {
        Ok(())
    } else {
        Err(HwndCommandPreflightError::InvalidHotkey)
    }
}

/// Native API adapter backed by `windows-sys` for MSVC Windows builds.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysHwndApi;

#[cfg(target_env = "msvc")]
impl HwndNativeApi for WindowsSysHwndApi {
    fn apply_window_styles(&mut self, hwnd: RawHwnd, style_bits: Win32StyleBits) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowLongPtrW, GWL_EXSTYLE, GWL_STYLE,
        };

        let hwnd = windows_sys_hwnd(hwnd);
        unsafe {
            SetWindowLongPtrW(hwnd, GWL_STYLE, style_bits.style as isize);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style_bits.extended_style as isize);
        }
        true
    }

    fn update_hit_test_layout(&mut self, hwnd: RawHwnd, hit_test: HitTestLayout) -> bool {
        update_registered_hit_test_layout(hwnd, hit_test);
        true
    }

    fn move_resize(&mut self, hwnd: RawHwnd, placement: WindowPlacement) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
        };

        let hwnd = windows_sys_hwnd(hwnd);
        unsafe {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                placement.origin.x,
                placement.origin.y,
                placement.size.width,
                placement.size.height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            ) != 0
        }
    }

    fn show_no_activate(&mut self, hwnd: RawHwnd) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNOACTIVATE};

        unsafe {
            ShowWindow(windows_sys_hwnd(hwnd), SW_SHOWNOACTIVATE);
        }
        true
    }

    fn hide_compact_window(&mut self, hwnd: RawHwnd) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};

        unsafe {
            ShowWindow(windows_sys_hwnd(hwnd), SW_HIDE);
        }
        true
    }

    fn set_topmost(&mut self, hwnd: RawHwnd) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        };

        unsafe {
            SetWindowPos(
                windows_sys_hwnd(hwnd),
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            ) != 0
        }
    }

    fn clear_topmost(&mut self, hwnd: RawHwnd) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_NOTOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        };

        unsafe {
            SetWindowPos(
                windows_sys_hwnd(hwnd),
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            ) != 0
        }
    }

    fn register_hotkey(&mut self, hwnd: RawHwnd, hotkey: Win32HotkeyChord) -> bool {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;

        unsafe {
            RegisterHotKey(
                windows_sys_hwnd(hwnd),
                hotkey.id,
                hotkey.modifiers,
                hotkey.virtual_key,
            ) != 0
        }
    }

    fn unregister_hotkey(&mut self, hwnd: RawHwnd, hotkey_id: i32) -> bool {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey;

        unsafe { UnregisterHotKey(windows_sys_hwnd(hwnd), hotkey_id) != 0 }
    }
}

#[cfg(target_env = "msvc")]
fn windows_sys_hwnd(hwnd: RawHwnd) -> windows_sys::Win32::Foundation::HWND {
    hwnd.value() as windows_sys::Win32::Foundation::HWND
}

#[cfg(target_env = "msvc")]
fn update_registered_hit_test_layout(hwnd: RawHwnd, layout: HitTestLayout) {
    let mut registry = hit_test_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, bridge)) = registry
        .iter_mut()
        .find(|(registered, _)| *registered == hwnd)
    {
        bridge.update_layout(layout);
    } else {
        let mut bridge = HwndHitTestBridge::default();
        bridge.update_layout(layout);
        registry.push((hwnd, bridge));
    }
    update_registered_mouse_input_layout(hwnd, layout);
}

#[cfg(target_env = "msvc")]
fn remove_registered_hit_test_layout(hwnd: RawHwnd) {
    let mut registry = hit_test_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    registry.retain(|(registered, _)| *registered != hwnd);
    remove_registered_mouse_input(hwnd);
    remove_registered_render_input(hwnd);
}

#[cfg(target_env = "msvc")]
fn hit_test_code_for_hwnd_client_point(hwnd: RawHwnd, point: PointPx) -> Option<i32> {
    let registry = hit_test_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    registry
        .iter()
        .find(|(registered, _)| *registered == hwnd)
        .and_then(|(_, bridge)| bridge.nchittest_for_client_point(point))
}

#[cfg(target_env = "msvc")]
fn hit_test_registry() -> &'static std::sync::Mutex<Vec<(RawHwnd, HwndHitTestBridge)>> {
    static REGISTRY: std::sync::OnceLock<std::sync::Mutex<Vec<(RawHwnd, HwndHitTestBridge)>>> =
        std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

#[cfg(target_env = "msvc")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct RegisteredMouseInput {
    hwnd: RawHwnd,
    bridge: HwndMouseInputBridge,
    events: Vec<HwndMouseInputEvent>,
}

#[cfg(target_env = "msvc")]
fn update_registered_mouse_input_layout(hwnd: RawHwnd, layout: HitTestLayout) {
    let mut registry = mouse_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(registered) = registry
        .iter_mut()
        .find(|registered| registered.hwnd == hwnd)
    {
        registered.bridge.update_layout(layout);
    } else {
        let mut bridge = HwndMouseInputBridge::default();
        bridge.update_layout(layout);
        registry.push(RegisteredMouseInput {
            hwnd,
            bridge,
            events: Vec::new(),
        });
    }
}

#[cfg(target_env = "msvc")]
fn remove_registered_mouse_input(hwnd: RawHwnd) {
    let mut registry = mouse_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    registry.retain(|registered| registered.hwnd != hwnd);
}

#[cfg(target_env = "msvc")]
fn dispatch_registered_left_button_up(hwnd: RawHwnd, point: PointPx) -> bool {
    let mut registry = mouse_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(registered) = registry
        .iter_mut()
        .find(|registered| registered.hwnd == hwnd)
    else {
        return false;
    };
    let Some(event) = registered.bridge.event_for_left_button_up(point) else {
        return false;
    };
    registered.events.push(event);
    true
}

#[cfg(target_env = "msvc")]
fn drain_registered_mouse_input(hwnd: RawHwnd) -> Vec<HwndMouseInputEvent> {
    let mut registry = mouse_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(registered) = registry
        .iter_mut()
        .find(|registered| registered.hwnd == hwnd)
    else {
        return Vec::new();
    };
    std::mem::take(&mut registered.events)
}

#[cfg(target_env = "msvc")]
fn mouse_input_registry() -> &'static std::sync::Mutex<Vec<RegisteredMouseInput>> {
    static REGISTRY: std::sync::OnceLock<std::sync::Mutex<Vec<RegisteredMouseInput>>> =
        std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// `windows-sys` content-free input queue for compact HWND mouse events.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysHwndInputQueue;

#[cfg(target_env = "msvc")]
impl WindowsSysHwndInputQueue {
    /// Drain content-free mouse input events already dispatched by the compact HWND WndProc.
    pub fn drain(hwnd: RawHwnd) -> Vec<HwndMouseInputEvent> {
        drain_registered_mouse_input(hwnd)
    }
}

#[cfg(target_env = "msvc")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct RegisteredRenderInput {
    hwnd: RawHwnd,
    events: Vec<HwndRenderEvent>,
}

#[cfg(target_env = "msvc")]
fn dispatch_registered_paint(hwnd: RawHwnd) {
    let mut registry = render_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(registered) = registry
        .iter_mut()
        .find(|registered| registered.hwnd == hwnd)
    {
        registered.events.push(HwndPaintBridge.event_for_paint());
    } else {
        registry.push(RegisteredRenderInput {
            hwnd,
            events: vec![HwndPaintBridge.event_for_paint()],
        });
    }
}

#[cfg(target_env = "msvc")]
fn remove_registered_render_input(hwnd: RawHwnd) {
    let mut registry = render_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    registry.retain(|registered| registered.hwnd != hwnd);
}

#[cfg(target_env = "msvc")]
fn drain_registered_render_input(hwnd: RawHwnd) -> Vec<HwndRenderEvent> {
    let mut registry = render_input_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(registered) = registry
        .iter_mut()
        .find(|registered| registered.hwnd == hwnd)
    else {
        return Vec::new();
    };
    std::mem::take(&mut registered.events)
}

#[cfg(target_env = "msvc")]
fn render_input_registry() -> &'static std::sync::Mutex<Vec<RegisteredRenderInput>> {
    static REGISTRY: std::sync::OnceLock<std::sync::Mutex<Vec<RegisteredRenderInput>>> =
        std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// `windows-sys` content-free render queue for compact HWND paint events.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysHwndRenderQueue;

#[cfg(target_env = "msvc")]
impl WindowsSysHwndRenderQueue {
    /// Drain content-free render events already dispatched by the compact HWND WndProc.
    pub fn drain(hwnd: RawHwnd) -> Vec<HwndRenderEvent> {
        drain_registered_render_input(hwnd)
    }
}

/// `windows-sys` compact-window factory for MSVC Windows builds.
#[cfg(target_env = "msvc")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowsSysCompactWindowFactory {
    spec: CompactWindowClassSpec,
    class_registered: bool,
}

#[cfg(target_env = "msvc")]
impl WindowsSysCompactWindowFactory {
    /// Create a compact-window factory using content-free class metadata.
    pub const fn new(spec: CompactWindowClassSpec) -> Self {
        Self {
            spec,
            class_registered: false,
        }
    }

    fn ensure_class_registered(&mut self) -> bool {
        if self.class_registered {
            return true;
        }

        let Some(instance) = module_handle() else {
            return false;
        };
        let class = windows_sys::Win32::UI::WindowsAndMessaging::WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(compact_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: self.spec.class_name_wide().as_ptr(),
        };

        let atom = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::RegisterClassW(&class) };
        if atom != 0 || class_already_registered() {
            self.class_registered = true;
            true
        } else {
            false
        }
    }
}

#[cfg(target_env = "msvc")]
impl HwndCompactWindowFactory for WindowsSysCompactWindowFactory {
    fn create_compact_window(&mut self) -> Option<RawHwnd> {
        if !self.ensure_class_registered() {
            return None;
        }

        let instance = module_handle()?;
        let hwnd = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::CreateWindowExW(
                windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE
                    | windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW
                    | windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TOPMOST,
                self.spec.class_name_wide().as_ptr(),
                self.spec.window_title_wide().as_ptr(),
                windows_sys::Win32::UI::WindowsAndMessaging::WS_POPUP,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            )
        };

        RawHwnd::new(hwnd as isize)
    }

    fn destroy_compact_window(&mut self, hwnd: RawHwnd) -> bool {
        let destroyed = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(windows_sys_hwnd(hwnd)) != 0
        };
        if destroyed {
            remove_registered_hit_test_layout(hwnd);
        }
        destroyed
    }
}

#[cfg(target_env = "msvc")]
fn module_handle() -> Option<windows_sys::Win32::Foundation::HINSTANCE> {
    let module =
        unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null()) };
    if module.is_null() {
        None
    } else {
        Some(module)
    }
}

#[cfg(target_env = "msvc")]
fn class_already_registered() -> bool {
    let last_error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    last_error == windows_sys::Win32::Foundation::ERROR_CLASS_ALREADY_EXISTS
}

/// `windows-sys` nonblocking compact-window message pump.
#[cfg(target_env = "msvc")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowsSysMessagePump;

#[cfg(target_env = "msvc")]
impl HwndMessagePump for WindowsSysMessagePump {
    fn drain_pending_for_window(
        &mut self,
        hwnd: RawHwnd,
        budget: HwndMessagePumpBudget,
    ) -> HwndMessagePumpReport {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        };

        let mut report = HwndMessagePumpReport::default();
        for _ in 0..budget.max_messages() {
            let mut message = MSG::default();
            let has_message =
                unsafe { PeekMessageW(&mut message, windows_sys_hwnd(hwnd), 0, 0, PM_REMOVE) };
            if has_message == 0 {
                break;
            }
            report = report.record_removed_message();
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            report = report.record_dispatch_call();
        }
        report
    }
}

#[cfg(target_env = "msvc")]
unsafe extern "system" fn compact_window_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    message: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT {
    if message == windows_sys::Win32::UI::WindowsAndMessaging::WM_NCHITTEST {
        if let Some(code) = handle_nchittest(hwnd, lparam) {
            return code as windows_sys::Win32::Foundation::LRESULT;
        }
    }
    if message == windows_sys::Win32::UI::WindowsAndMessaging::WM_MOUSEACTIVATE {
        return windows_sys::Win32::UI::WindowsAndMessaging::MA_NOACTIVATE
            as windows_sys::Win32::Foundation::LRESULT;
    }
    if message == windows_sys::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP {
        handle_left_button_up(hwnd, lparam);
        return 0;
    }
    if message == windows_sys::Win32::UI::WindowsAndMessaging::WM_PAINT {
        handle_paint(hwnd);
        return 0;
    }
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

#[cfg(target_env = "msvc")]
fn handle_nchittest(
    hwnd: windows_sys::Win32::Foundation::HWND,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> Option<i32> {
    let raw = RawHwnd::new(hwnd as isize)?;
    let mut point = windows_sys::Win32::Foundation::POINT {
        x: signed_low_word(lparam),
        y: signed_high_word(lparam),
    };
    let converted = unsafe { windows_sys::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut point) };
    if converted == 0 {
        return None;
    }
    hit_test_code_for_hwnd_client_point(
        raw,
        PointPx {
            x: point.x,
            y: point.y,
        },
    )
}

#[cfg(target_env = "msvc")]
fn handle_left_button_up(
    hwnd: windows_sys::Win32::Foundation::HWND,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) {
    let Some(raw) = RawHwnd::new(hwnd as isize) else {
        return;
    };
    let point = PointPx {
        x: signed_low_word(lparam),
        y: signed_high_word(lparam),
    };
    dispatch_registered_left_button_up(raw, point);
}

#[cfg(target_env = "msvc")]
fn handle_paint(hwnd: windows_sys::Win32::Foundation::HWND) {
    if let Some(raw) = RawHwnd::new(hwnd as isize) {
        dispatch_registered_paint(raw);
    }
    let _ = unsafe { windows_sys::Win32::Graphics::Gdi::ValidateRect(hwnd, std::ptr::null()) };
}

#[cfg(target_env = "msvc")]
fn signed_low_word(value: isize) -> i32 {
    i32::from((value as u16) as i16)
}

#[cfg(target_env = "msvc")]
fn signed_high_word(value: isize) -> i32 {
    i32::from(((value >> 16) as u16) as i16)
}
