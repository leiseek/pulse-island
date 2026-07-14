//! Windows platform primitives for Pulse Island.
#![deny(missing_docs)]

/// Content-free local object names for one W3 Pulse Link user/logon-session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkLocalObjectNames {
    /// Single-instance mutex name.
    pub mutex: String,
    /// Shim-to-Link ingress pipe name.
    pub ingress_pipe: String,
    /// Island client pipe name.
    pub island_pipe: String,
    /// Optional Link-ready event name.
    pub ready_event: String,
}

impl LinkLocalObjectNames {
    /// Derive stable names from raw namespace inputs without exposing those inputs.
    pub fn derive(
        install_id: &str,
        user_sid: &str,
        logon_session: &str,
        protocol_major: u16,
    ) -> Self {
        let install_hash = stable_hex_hash(install_id);
        let session_hash = stable_hex_hash(&format!("{user_sid}|{logon_session}"));
        Self {
            mutex: format!(
                r"Local\PulseIsland.Link.{install_hash}.{session_hash}.v{protocol_major}"
            ),
            ingress_pipe: format!(
                r"\\.\pipe\PulseIsland.{install_hash}.{session_hash}.ingress.v{protocol_major}"
            ),
            island_pipe: format!(
                r"\\.\pipe\PulseIsland.{install_hash}.{session_hash}.island.v{protocol_major}"
            ),
            ready_event: format!(
                r"Local\PulseIsland.LinkReady.{install_hash}.{session_hash}.v{protocol_major}"
            ),
        }
    }

    /// Borrow every generated object name for diagnostics/tests.
    pub fn as_slice(&self) -> [&str; 4] {
        [
            self.mutex.as_str(),
            self.ingress_pipe.as_str(),
            self.island_pipe.as_str(),
            self.ready_event.as_str(),
        ]
    }
}

/// Observed state when a Link startup path checks local ownership objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkStartupObservation {
    /// No mutex/pipe objects are visible for this scoped Link instance.
    NoExistingObjects,
    /// Another Link appears to own the scoped mutex.
    MutexAlreadyOwned,
    /// Mutex/pipe state looks stale and may be retried only within a small bound.
    StaleMutexOrPipe,
}

/// Pure single-instance ownership decision before OS mutex/pipe handles are wired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkOwnershipDecision {
    /// This Link should become the scoped single instance and create pipe servers.
    OwnInstance,
    /// A Shim/Link should connect to the existing scoped Link.
    ConnectToExisting,
    /// Retry stale mutex/pipe observation within a fixed small budget.
    RetryBounded,
    /// Give up and fail open without affecting the synthetic provider.
    FailOpen,
}

/// Pure registry that models per-user/per-logon-session Link ownership.
#[derive(Clone, Debug, Default)]
pub struct LinkOwnershipRegistry {
    owned_mutexes: Vec<String>,
    stale_retry_counts: Vec<(String, u8)>,
}

impl LinkOwnershipRegistry {
    /// Observe one startup attempt and return the bounded ownership decision.
    pub fn observe_start(
        &mut self,
        names: &LinkLocalObjectNames,
        observation: LinkStartupObservation,
    ) -> LinkOwnershipDecision {
        match observation {
            LinkStartupObservation::NoExistingObjects => {
                if self.owned_mutexes.contains(&names.mutex) {
                    LinkOwnershipDecision::ConnectToExisting
                } else {
                    self.owned_mutexes.push(names.mutex.clone());
                    LinkOwnershipDecision::OwnInstance
                }
            }
            LinkStartupObservation::MutexAlreadyOwned => LinkOwnershipDecision::ConnectToExisting,
            LinkStartupObservation::StaleMutexOrPipe => self.observe_stale(names),
        }
    }

    /// Number of modeled owners for this scoped Link name.
    pub fn owner_count(&self, names: &LinkLocalObjectNames) -> usize {
        self.owned_mutexes
            .iter()
            .filter(|mutex| *mutex == &names.mutex)
            .count()
    }

    fn observe_stale(&mut self, names: &LinkLocalObjectNames) -> LinkOwnershipDecision {
        const MAX_STALE_RETRIES: u8 = 2;

        if let Some((_, count)) = self
            .stale_retry_counts
            .iter_mut()
            .find(|(mutex, _)| mutex == &names.mutex)
        {
            if *count >= MAX_STALE_RETRIES {
                return LinkOwnershipDecision::FailOpen;
            }
            *count += 1;
            return LinkOwnershipDecision::RetryBounded;
        }

        self.stale_retry_counts.push((names.mutex.clone(), 1));
        LinkOwnershipDecision::RetryBounded
    }
}

fn stable_hex_hash(input: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

/// Physical pixel point in a window-local coordinate space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointPx {
    /// Horizontal coordinate in physical pixels.
    pub x: i32,
    /// Vertical coordinate in physical pixels.
    pub y: i32,
}

/// Physical pixel size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizePx {
    /// Width in physical pixels.
    pub width: i32,
    /// Height in physical pixels.
    pub height: i32,
}

/// Physical pixel rectangle using left/top inclusive and right/bottom exclusive edges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectPx {
    /// Left edge in physical pixels.
    pub left: i32,
    /// Top edge in physical pixels.
    pub top: i32,
    /// Right edge in physical pixels.
    pub right: i32,
    /// Bottom edge in physical pixels.
    pub bottom: i32,
}

/// Window origin and size in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowPlacement {
    /// Top-left window origin.
    pub origin: PointPx,
    /// Window size.
    pub size: SizePx,
}

/// Logical pixel point remembered per monitor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalPoint {
    /// Horizontal logical coordinate.
    pub x: u32,
    /// Vertical logical coordinate.
    pub y: u32,
}

/// Logical pixel size remembered per monitor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalSize {
    /// Width in logical pixels.
    pub width: u32,
    /// Height in logical pixels.
    pub height: u32,
}

/// Stable monitor identifier used by the pure placement policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorId(pub u32);

/// One monitor's work area and DPI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorWorkArea {
    /// Monitor identifier.
    pub id: MonitorId,
    /// Current work area in physical pixels.
    pub work_area: RectPx,
    /// Monitor DPI.
    pub dpi: DpiScale,
}

/// Current display topology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayTopology {
    /// Monitors in deterministic enumeration order.
    pub monitors: Vec<MonitorWorkArea>,
}

/// User-remembered compact placement in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RememberedLogicalPlacement {
    /// Monitor this placement was saved against.
    pub monitor_id: MonitorId,
    /// Saved logical origin.
    pub origin_logical: LogicalPoint,
    /// Saved logical size.
    pub size_logical: LogicalSize,
}

/// Resolved physical placement for the current topology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedPlacement {
    /// Monitor used for resolution.
    pub monitor_id: MonitorId,
    /// Physical placement clamped to the current work area.
    pub placement: WindowPlacement,
}

/// Pure compact-window style contract before mapping to Win32 constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactWindowStylePolicy {
    /// Borderless popup window.
    pub popup: bool,
    /// Keep above ordinary windows.
    pub topmost: bool,
    /// Exclude from Alt+Tab/task switcher.
    pub tool_window: bool,
    /// Do not activate on passive hover/state changes.
    pub no_activate: bool,
    /// Whether compact Island should appear in Alt+Tab.
    pub alt_tab_visible: bool,
    /// Whether the whole window is permanently click-through.
    pub permanently_click_through: bool,
}

impl Default for CompactWindowStylePolicy {
    fn default() -> Self {
        Self {
            popup: true,
            topmost: true,
            tool_window: true,
            no_activate: true,
            alt_tab_visible: false,
            permanently_click_through: false,
        }
    }
}

const WS_POPUP: u32 = 0x8000_0000;
const WS_EX_TOPMOST: u32 = 0x0000_0008;
const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_APPWINDOW: u32 = 0x0004_0000;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
const MOD_CONTROL: u32 = 0x0000_0002;
const MOD_SHIFT: u32 = 0x0000_0004;
const VK_SPACE: u32 = 0x20;

/// Concrete Win32 style masks derived from pure compact-window policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Win32StyleBits {
    /// Regular window style bits.
    pub style: u32,
    /// Extended window style bits.
    pub extended_style: u32,
}

impl Win32StyleBits {
    /// Map pure compact-window style policy to documented Win32 style bit values.
    pub fn from_policy(policy: CompactWindowStylePolicy) -> Self {
        let style = if policy.popup { WS_POPUP } else { 0 };
        let mut extended_style = 0;
        if policy.topmost {
            extended_style |= WS_EX_TOPMOST;
        }
        if policy.tool_window {
            extended_style |= WS_EX_TOOLWINDOW;
        }
        if policy.no_activate {
            extended_style |= WS_EX_NOACTIVATE;
        }
        if policy.alt_tab_visible {
            extended_style |= WS_EX_APPWINDOW;
        }
        if policy.permanently_click_through {
            extended_style |= WS_EX_TRANSPARENT;
        }
        Self {
            style,
            extended_style,
        }
    }

    /// Whether popup style is present.
    pub fn has_popup(self) -> bool {
        self.style & WS_POPUP == WS_POPUP
    }

    /// Whether topmost extended style is present.
    pub fn has_topmost(self) -> bool {
        self.extended_style & WS_EX_TOPMOST == WS_EX_TOPMOST
    }

    /// Whether tool-window extended style is present.
    pub fn has_tool_window(self) -> bool {
        self.extended_style & WS_EX_TOOLWINDOW == WS_EX_TOOLWINDOW
    }

    /// Whether no-activate extended style is present.
    pub fn has_no_activate(self) -> bool {
        self.extended_style & WS_EX_NOACTIVATE == WS_EX_NOACTIVATE
    }

    /// Whether app-window extended style is present.
    pub fn has_app_window(self) -> bool {
        self.extended_style & WS_EX_APPWINDOW == WS_EX_APPWINDOW
    }

    /// Whether transparent extended style is present.
    pub fn has_transparent(self) -> bool {
        self.extended_style & WS_EX_TRANSPARENT == WS_EX_TRANSPARENT
    }
}

/// Pure global hotkey policy for the W2 Command Palette shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalHotkeyPolicy {
    /// Whether the global hotkey should be registered.
    pub enabled: bool,
    /// Stable registration id for the process-local hotkey.
    pub id: i32,
    /// Win32 modifier bitmask.
    pub modifiers: u32,
    /// Win32 virtual-key code.
    pub virtual_key: u32,
}

impl GlobalHotkeyPolicy {
    /// Default W2 Palette shortcut: Ctrl+Shift+Space.
    pub const fn palette_default() -> Self {
        Self {
            enabled: true,
            id: 1,
            modifiers: MOD_CONTROL | MOD_SHIFT,
            virtual_key: VK_SPACE,
        }
    }

    /// Return the future `RegisterHotKey` chord when enabled.
    pub const fn registration_chord(self) -> Option<Win32HotkeyChord> {
        if self.enabled {
            Some(Win32HotkeyChord {
                id: self.id,
                modifiers: self.modifiers,
                virtual_key: self.virtual_key,
            })
        } else {
            None
        }
    }
}

/// Concrete hotkey chord values for future `RegisterHotKey` wiring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Win32HotkeyChord {
    /// Process-local hotkey id.
    pub id: i32,
    /// Modifier bitmask.
    pub modifiers: u32,
    /// Virtual-key code.
    pub virtual_key: u32,
}

/// Simulated immersive state used before real fullscreen detection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImmersiveState {
    /// Normal desktop state.
    Normal,
    /// Fullscreen or presentation state.
    Fullscreen,
}

/// Window visibility selected by immersive policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowVisibility {
    /// Compact Island is visible.
    Visible,
    /// Compact Island is hidden.
    Hidden,
}

/// Pure window policy for fullscreen/presentation suppression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImmersiveWindowPolicy {
    /// Selected visibility.
    pub visibility: WindowVisibility,
    /// Whether topmost behavior remains active.
    pub keep_topmost: bool,
    /// Whether missed animations should replay after restore.
    pub replay_missed_animations_on_restore: bool,
}

impl ImmersiveWindowPolicy {
    /// Build immersive window policy from a simulated state.
    pub const fn for_state(state: ImmersiveState) -> Self {
        match state {
            ImmersiveState::Normal => Self {
                visibility: WindowVisibility::Visible,
                keep_topmost: true,
                replay_missed_animations_on_restore: false,
            },
            ImmersiveState::Fullscreen => Self {
                visibility: WindowVisibility::Hidden,
                keep_topmost: false,
                replay_missed_animations_on_restore: false,
            },
        }
    }
}

/// Pure input for the future native compact-window adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeWindowAdapterInput {
    /// Whether the compact Island should be visible from the UI shell state.
    pub compact_visible: bool,
    /// Current immersive/fullscreen policy state.
    pub immersive_state: ImmersiveState,
    /// Resolved physical placement to apply.
    pub placement: WindowPlacement,
    /// Hit-test layout to apply to `WM_NCHITTEST`.
    pub hit_test: HitTestLayout,
    /// Compact window style policy.
    pub style_policy: CompactWindowStylePolicy,
    /// Command Palette hotkey registration policy.
    pub hotkey_policy: GlobalHotkeyPolicy,
}

/// Pure per-frame plan for the future native compact-window adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeWindowAdapterPlan {
    /// Whether the adapter should create the compact HWND if it does not exist.
    pub create_compact_window_if_missing: bool,
    /// Whether the adapter should recreate the compact HWND for this frame.
    pub recreate_compact_window: bool,
    /// Whether the adapter should destroy the compact HWND when hidden.
    pub destroy_compact_window_when_hidden: bool,
    /// Whether showing the compact Island may activate it.
    pub activate_on_show: bool,
    /// Selected compact window visibility.
    pub visibility: WindowVisibility,
    /// Whether topmost behavior should be active for this frame.
    pub keep_topmost: bool,
    /// Whether restore should replay animations missed while hidden.
    pub replay_missed_animations_on_restore: bool,
    /// Resolved placement to apply.
    pub placement: WindowPlacement,
    /// Hit-test layout to use.
    pub hit_test: HitTestLayout,
    /// Concrete style bits derived from compact policy.
    pub style_bits: Win32StyleBits,
    /// Optional Command Palette hotkey chord to register.
    pub hotkey: Option<Win32HotkeyChord>,
}

/// Pure action diff for the future native compact-window adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeWindowAdapterAction {
    /// Create the compact native window.
    CreateCompactWindow,
    /// Destroy the compact native window.
    DestroyCompactWindow,
    /// Apply window style and extended-style bits.
    ApplyWindowStyles,
    /// Update cached hit-test layout used by `WM_NCHITTEST`.
    UpdateHitTestLayout,
    /// Move or resize the compact window.
    MoveResize,
    /// Register the Command Palette hotkey.
    RegisterHotkey,
    /// Unregister the Command Palette hotkey.
    UnregisterHotkey,
    /// Show the compact window without activation.
    ShowNoActivate,
    /// Hide the compact window.
    HideCompactWindow,
    /// Apply topmost positioning.
    SetTopmost,
    /// Remove topmost positioning.
    ClearTopmost,
}

/// Executable native adapter command with the payload needed by a backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeWindowAdapterCommand {
    /// Create the compact native window.
    CreateCompactWindow,
    /// Destroy the compact native window.
    DestroyCompactWindow,
    /// Apply window style and extended-style bits.
    ApplyWindowStyles(Win32StyleBits),
    /// Update cached hit-test layout used by `WM_NCHITTEST`.
    UpdateHitTestLayout(HitTestLayout),
    /// Move or resize the compact window.
    MoveResize(WindowPlacement),
    /// Register the Command Palette hotkey.
    RegisterHotkey(Win32HotkeyChord),
    /// Unregister the Command Palette hotkey.
    UnregisterHotkey,
    /// Show the compact window without activation.
    ShowNoActivate,
    /// Hide the compact window.
    HideCompactWindow,
    /// Apply topmost positioning.
    SetTopmost,
    /// Remove topmost positioning.
    ClearTopmost,
}

impl NativeWindowAdapterCommand {
    /// Return the payload-free action represented by this command.
    pub const fn action(self) -> NativeWindowAdapterAction {
        match self {
            Self::CreateCompactWindow => NativeWindowAdapterAction::CreateCompactWindow,
            Self::DestroyCompactWindow => NativeWindowAdapterAction::DestroyCompactWindow,
            Self::ApplyWindowStyles(_) => NativeWindowAdapterAction::ApplyWindowStyles,
            Self::UpdateHitTestLayout(_) => NativeWindowAdapterAction::UpdateHitTestLayout,
            Self::MoveResize(_) => NativeWindowAdapterAction::MoveResize,
            Self::RegisterHotkey(_) => NativeWindowAdapterAction::RegisterHotkey,
            Self::UnregisterHotkey => NativeWindowAdapterAction::UnregisterHotkey,
            Self::ShowNoActivate => NativeWindowAdapterAction::ShowNoActivate,
            Self::HideCompactWindow => NativeWindowAdapterAction::HideCompactWindow,
            Self::SetTopmost => NativeWindowAdapterAction::SetTopmost,
            Self::ClearTopmost => NativeWindowAdapterAction::ClearTopmost,
        }
    }
}

/// Pure state accumulated by the future native compact-window adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeWindowAdapterState {
    /// Whether the compact native window has been created.
    pub compact_window_created: bool,
    /// Stable compact window generation.
    pub window_generation: u32,
    /// Number of future create-window calls required so far.
    pub create_window_calls: u32,
    /// Number of future destroy-window calls required so far.
    pub destroy_window_calls: u32,
    /// Current compact visibility.
    pub visibility: WindowVisibility,
    /// Whether the Command Palette hotkey is registered.
    pub hotkey_registered: bool,
    /// Number of future hotkey registration calls required so far.
    pub hotkey_register_calls: u32,
    /// Number of future hotkey unregister calls required so far.
    pub hotkey_unregister_calls: u32,
    /// Number of activation attempts requested by plans.
    pub activation_attempts: u32,
    /// Last applied adapter plan.
    pub last_plan: Option<NativeWindowAdapterPlan>,
}

/// Failure returned when a native adapter sink cannot apply an action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeWindowAdapterError {
    /// Action that failed in the native sink.
    pub failed_action: NativeWindowAdapterAction,
    /// Actions that were applied before the failure.
    pub applied_actions: Vec<NativeWindowAdapterAction>,
}

/// Failure returned when a native adapter command sink cannot apply a command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeWindowAdapterCommandError {
    /// Command that failed in the native sink.
    pub failed_command: NativeWindowAdapterCommand,
    /// Commands that were applied before the failure.
    pub applied_commands: Vec<NativeWindowAdapterCommand>,
}

/// Safe command sink implemented by the future HWND-backed native adapter.
pub trait NativeWindowActionSink {
    /// Apply one native adapter action. Return `false` when the action failed.
    fn apply_action(&mut self, action: NativeWindowAdapterAction) -> bool;
}

/// Safe payload command sink implemented by the future HWND-backed native adapter.
pub trait NativeWindowCommandSink {
    /// Apply one native adapter command. Return `false` when the command failed.
    fn apply_command(&mut self, command: NativeWindowAdapterCommand) -> bool;
}

/// Safe stateful driver that applies native adapter action diffs through a sink.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeWindowAdapterDriver {
    state: NativeWindowAdapterState,
}

impl Default for NativeWindowAdapterState {
    fn default() -> Self {
        Self {
            compact_window_created: false,
            window_generation: 0,
            create_window_calls: 0,
            destroy_window_calls: 0,
            visibility: WindowVisibility::Hidden,
            hotkey_registered: false,
            hotkey_register_calls: 0,
            hotkey_unregister_calls: 0,
            activation_attempts: 0,
            last_plan: None,
        }
    }
}

impl NativeWindowAdapterDriver {
    /// Return the driver's current pure adapter state.
    pub const fn state(&self) -> NativeWindowAdapterState {
        self.state
    }

    /// Apply one adapter plan by sending its ordered action diff to a native sink.
    ///
    /// The pure state advances only after every required action succeeds.
    pub fn apply_plan<S>(
        &mut self,
        plan: NativeWindowAdapterPlan,
        sink: &mut S,
    ) -> Result<Vec<NativeWindowAdapterAction>, NativeWindowAdapterError>
    where
        S: NativeWindowActionSink,
    {
        let actions = self.state.actions_for(plan);
        let mut applied_actions = Vec::new();
        for action in actions.iter().copied() {
            if !sink.apply_action(action) {
                return Err(NativeWindowAdapterError {
                    failed_action: action,
                    applied_actions,
                });
            }
            applied_actions.push(action);
        }
        self.state = self.state.apply(plan);
        Ok(applied_actions)
    }

    /// Apply one adapter plan by sending payload commands to a native sink.
    ///
    /// The pure state advances only after every required command succeeds.
    pub fn apply_plan_commands<S>(
        &mut self,
        plan: NativeWindowAdapterPlan,
        sink: &mut S,
    ) -> Result<Vec<NativeWindowAdapterCommand>, NativeWindowAdapterCommandError>
    where
        S: NativeWindowCommandSink,
    {
        let commands = self.state.commands_for(plan);
        let mut applied_commands = Vec::new();
        for command in commands.iter().copied() {
            if !sink.apply_command(command) {
                return Err(NativeWindowAdapterCommandError {
                    failed_command: command,
                    applied_commands,
                });
            }
            applied_commands.push(command);
        }
        self.state = self.state.apply(plan);
        Ok(applied_commands)
    }
}

impl NativeWindowAdapterPlan {
    /// Build a pure native adapter plan without touching Win32 APIs.
    pub fn from_input(input: NativeWindowAdapterInput) -> Self {
        let immersive = ImmersiveWindowPolicy::for_state(input.immersive_state);
        let visibility =
            if input.compact_visible && immersive.visibility == WindowVisibility::Visible {
                WindowVisibility::Visible
            } else {
                WindowVisibility::Hidden
            };
        Self {
            create_compact_window_if_missing: true,
            recreate_compact_window: false,
            destroy_compact_window_when_hidden: false,
            activate_on_show: false,
            visibility,
            keep_topmost: visibility == WindowVisibility::Visible && immersive.keep_topmost,
            replay_missed_animations_on_restore: immersive.replay_missed_animations_on_restore,
            placement: input.placement,
            hit_test: input.hit_test,
            style_bits: Win32StyleBits::from_policy(input.style_policy),
            hotkey: input.hotkey_policy.registration_chord(),
        }
    }
}

impl NativeWindowAdapterState {
    /// Return ordered future native actions required to apply a plan.
    pub fn actions_for(self, plan: NativeWindowAdapterPlan) -> Vec<NativeWindowAdapterAction> {
        self.commands_for(plan)
            .iter()
            .map(|command| command.action())
            .collect::<Vec<_>>()
    }

    /// Return ordered future native commands required to apply a plan.
    pub fn commands_for(self, plan: NativeWindowAdapterPlan) -> Vec<NativeWindowAdapterCommand> {
        let mut commands = Vec::new();
        if plan.create_compact_window_if_missing && !self.compact_window_created {
            commands.push(NativeWindowAdapterCommand::CreateCompactWindow);
        }
        if plan.destroy_compact_window_when_hidden
            && plan.visibility == WindowVisibility::Hidden
            && self.compact_window_created
        {
            commands.push(NativeWindowAdapterCommand::DestroyCompactWindow);
        }
        if match self.last_plan {
            Some(last) => last.style_bits != plan.style_bits,
            None => true,
        } {
            commands.push(NativeWindowAdapterCommand::ApplyWindowStyles(
                plan.style_bits,
            ));
        }
        if match self.last_plan {
            Some(last) => last.hit_test != plan.hit_test,
            None => true,
        } {
            commands.push(NativeWindowAdapterCommand::UpdateHitTestLayout(
                plan.hit_test,
            ));
        }
        if match self.last_plan {
            Some(last) => last.placement != plan.placement,
            None => true,
        } {
            commands.push(NativeWindowAdapterCommand::MoveResize(plan.placement));
        }
        if let (Some(hotkey), false) = (plan.hotkey, self.hotkey_registered) {
            commands.push(NativeWindowAdapterCommand::RegisterHotkey(hotkey));
        } else if plan.hotkey.is_none() && self.hotkey_registered {
            commands.push(NativeWindowAdapterCommand::UnregisterHotkey);
        }
        if plan.visibility == WindowVisibility::Visible
            && self.visibility != WindowVisibility::Visible
        {
            commands.push(NativeWindowAdapterCommand::ShowNoActivate);
        } else if plan.visibility == WindowVisibility::Hidden
            && self.visibility != WindowVisibility::Hidden
        {
            commands.push(NativeWindowAdapterCommand::HideCompactWindow);
        }
        let last_topmost = self.last_plan.is_some_and(|last| last.keep_topmost);
        if plan.keep_topmost && !last_topmost {
            commands.push(NativeWindowAdapterCommand::SetTopmost);
        } else if !plan.keep_topmost && last_topmost {
            commands.push(NativeWindowAdapterCommand::ClearTopmost);
        }
        commands
    }

    /// Apply one pure adapter plan and return the next pure adapter state.
    pub fn apply(mut self, plan: NativeWindowAdapterPlan) -> Self {
        if plan.create_compact_window_if_missing && !self.compact_window_created {
            self.compact_window_created = true;
            self.window_generation = self.window_generation.saturating_add(1);
            self.create_window_calls = self.create_window_calls.saturating_add(1);
        }
        if plan.destroy_compact_window_when_hidden
            && plan.visibility == WindowVisibility::Hidden
            && self.compact_window_created
        {
            self.compact_window_created = false;
            self.destroy_window_calls = self.destroy_window_calls.saturating_add(1);
        }
        if plan.hotkey.is_some() && !self.hotkey_registered {
            self.hotkey_registered = true;
            self.hotkey_register_calls = self.hotkey_register_calls.saturating_add(1);
        } else if plan.hotkey.is_none() && self.hotkey_registered {
            self.hotkey_registered = false;
            self.hotkey_unregister_calls = self.hotkey_unregister_calls.saturating_add(1);
        }
        if plan.activate_on_show {
            self.activation_attempts = self.activation_attempts.saturating_add(1);
        }
        self.visibility = plan.visibility;
        self.last_plan = Some(plan);
        self
    }
}

impl WindowPlacement {
    /// Clamp the window origin so the placement remains within the supplied work area.
    pub fn clamp_to(self, work_area: RectPx) -> Self {
        Self {
            origin: PointPx {
                x: clamp_axis(
                    self.origin.x,
                    self.size.width,
                    work_area.left,
                    work_area.right,
                ),
                y: clamp_axis(
                    self.origin.y,
                    self.size.height,
                    work_area.top,
                    work_area.bottom,
                ),
            },
            size: self.size,
        }
    }
}

impl RememberedLogicalPlacement {
    /// Resolve remembered logical placement against the current monitor topology.
    pub fn resolve(self, topology: &DisplayTopology) -> Option<ResolvedPlacement> {
        let monitor = topology
            .monitors
            .iter()
            .find(|monitor| monitor.id == self.monitor_id)
            .or_else(|| topology.monitors.first())?;
        let placement = WindowPlacement {
            origin: PointPx {
                x: monitor
                    .work_area
                    .left
                    .saturating_add(u32_to_i32(monitor.dpi.scale_px(self.origin_logical.x))),
                y: monitor
                    .work_area
                    .top
                    .saturating_add(u32_to_i32(monitor.dpi.scale_px(self.origin_logical.y))),
            },
            size: SizePx {
                width: u32_to_i32(monitor.dpi.scale_px(self.size_logical.width)),
                height: u32_to_i32(monitor.dpi.scale_px(self.size_logical.height)),
            },
        }
        .clamp_to(monitor.work_area);
        Some(ResolvedPlacement {
            monitor_id: monitor.id,
            placement,
        })
    }
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn clamp_axis(origin: i32, size: i32, min_edge: i32, max_edge: i32) -> i32 {
    let available = max_edge.saturating_sub(min_edge);
    if size >= available {
        return min_edge;
    }
    let max_origin = max_edge.saturating_sub(size);
    if origin < min_edge {
        min_edge
    } else if origin > max_origin {
        max_origin
    } else {
        origin
    }
}

/// Hit-test result used by the future Win32 `WM_NCHITTEST` adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTarget {
    /// Point is outside the Island window.
    Outside,
    /// Point is transparent margin and should pass through.
    Transparent,
    /// Point is normal interactive client area.
    Client,
    /// Point is a drag grip.
    Drag,
}

/// Documented Win32 `WM_NCHITTEST` result codes used by the future adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Win32HitTestCode {
    /// `HTTRANSPARENT`: let windows below this one receive the hit.
    Transparent,
    /// `HTCLIENT`: interactive client area.
    Client,
    /// `HTCAPTION`: drag-caption behavior.
    Caption,
    /// `HTNOWHERE`: outside this window.
    Nowhere,
}

impl Win32HitTestCode {
    /// Map a pure hit target to the documented Win32 hit-test result.
    pub const fn from_target(target: HitTarget) -> Self {
        match target {
            HitTarget::Transparent => Self::Transparent,
            HitTarget::Client => Self::Client,
            HitTarget::Drag => Self::Caption,
            HitTarget::Outside => Self::Nowhere,
        }
    }

    /// Return the integer value expected by `WM_NCHITTEST`.
    pub const fn value(self) -> i32 {
        match self {
            Self::Transparent => -1,
            Self::Nowhere => 0,
            Self::Client => 1,
            Self::Caption => 2,
        }
    }
}

/// Testable compact-window hit-test layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitTestLayout {
    /// Window width in physical pixels.
    pub width_px: i32,
    /// Window height in physical pixels.
    pub height_px: i32,
    /// Transparent margin size in physical pixels.
    pub transparent_margin_px: i32,
    /// Drag grip width inside the interactive body.
    pub drag_grip_width_px: i32,
}

impl HitTestLayout {
    /// Classify a point for future Win32 hit-test mapping.
    pub fn hit_test(self, point: PointPx) -> HitTarget {
        if point.x < 0 || point.y < 0 || point.x >= self.width_px || point.y >= self.height_px {
            return HitTarget::Outside;
        }
        if point.x < self.transparent_margin_px
            || point.y < self.transparent_margin_px
            || point.x >= self.width_px - self.transparent_margin_px
            || point.y >= self.height_px - self.transparent_margin_px
        {
            return HitTarget::Transparent;
        }
        if point.x < self.transparent_margin_px + self.drag_grip_width_px {
            return HitTarget::Drag;
        }
        HitTarget::Client
    }
}

/// Per-monitor DPI scale helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DpiScale {
    dpi: u32,
}

/// Windows text scaling percentage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextScalePercent {
    percent: u32,
}

impl TextScalePercent {
    /// Create text scale percentage. Values below 100 are clamped to 100.
    pub const fn new(percent: u32) -> Self {
        let percent = if percent < 100 { 100 } else { percent };
        Self { percent }
    }
}

impl DpiScale {
    /// Create a DPI scale. Values below 96 are clamped to standard DPI.
    pub const fn new(dpi: u32) -> Self {
        Self { dpi }
    }

    /// Scale logical pixels to physical pixels, rounding up for non-zero values.
    pub const fn scale_px(self, logical_px: u32) -> u32 {
        let dpi = if self.dpi < 96 { 96 } else { self.dpi };
        let scaled = logical_px.saturating_mul(dpi);
        if scaled == 0 {
            0
        } else {
            scaled.div_ceil(96)
        }
    }

    /// Scale text logical pixels through monitor DPI and Windows text scaling.
    pub const fn scale_text_px(self, logical_px: u32, text_scale: TextScalePercent) -> u32 {
        let dpi = if self.dpi < 96 { 96 } else { self.dpi };
        let scaled = logical_px
            .saturating_mul(dpi)
            .saturating_mul(text_scale.percent);
        if scaled == 0 {
            0
        } else {
            scaled.div_ceil(96 * 100)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_distinguishes_transparent_margin_and_interactive_body() {
        let layout = HitTestLayout {
            width_px: 240,
            height_px: 56,
            transparent_margin_px: 8,
            drag_grip_width_px: 32,
        };

        assert_eq!(
            layout.hit_test(PointPx { x: 2, y: 20 }),
            HitTarget::Transparent
        );
        assert_eq!(layout.hit_test(PointPx { x: 16, y: 20 }), HitTarget::Drag);
        assert_eq!(layout.hit_test(PointPx { x: 80, y: 20 }), HitTarget::Client);
        assert_eq!(
            layout.hit_test(PointPx { x: 300, y: 20 }),
            HitTarget::Outside
        );
    }

    #[test]
    fn hit_targets_map_to_documented_win32_nchittest_codes() {
        assert_eq!(
            Win32HitTestCode::from_target(HitTarget::Transparent),
            Win32HitTestCode::Transparent
        );
        assert_eq!(
            Win32HitTestCode::from_target(HitTarget::Client),
            Win32HitTestCode::Client
        );
        assert_eq!(
            Win32HitTestCode::from_target(HitTarget::Drag),
            Win32HitTestCode::Caption
        );
        assert_eq!(
            Win32HitTestCode::from_target(HitTarget::Outside),
            Win32HitTestCode::Nowhere
        );
        assert_eq!(Win32HitTestCode::Transparent.value(), -1);
        assert_eq!(Win32HitTestCode::Client.value(), 1);
        assert_eq!(Win32HitTestCode::Caption.value(), 2);
        assert_eq!(Win32HitTestCode::Nowhere.value(), 0);
    }

    #[test]
    fn dpi_scale_converts_logical_to_physical_without_zeroing() {
        let dpi = DpiScale::new(144);

        assert_eq!(dpi.scale_px(100), 150);
        assert_eq!(dpi.scale_px(1), 2);
    }

    #[test]
    fn remembered_logical_placement_resolves_against_monitor_dpi_and_work_area() {
        let topology = DisplayTopology {
            monitors: vec![
                MonitorWorkArea {
                    id: MonitorId(1),
                    work_area: RectPx {
                        left: 0,
                        top: 0,
                        right: 1280,
                        bottom: 720,
                    },
                    dpi: DpiScale::new(96),
                },
                MonitorWorkArea {
                    id: MonitorId(7),
                    work_area: RectPx {
                        left: 1280,
                        top: 0,
                        right: 3200,
                        bottom: 1440,
                    },
                    dpi: DpiScale::new(144),
                },
            ],
        };
        let remembered = RememberedLogicalPlacement {
            monitor_id: MonitorId(7),
            origin_logical: LogicalPoint { x: 1400, y: 900 },
            size_logical: LogicalSize {
                width: 260,
                height: 64,
            },
        };
        let missing_monitor = RememberedLogicalPlacement {
            monitor_id: MonitorId(99),
            origin_logical: LogicalPoint { x: 100, y: 100 },
            size_logical: LogicalSize {
                width: 260,
                height: 64,
            },
        };

        let Some(resolved) = remembered.resolve(&topology) else {
            return;
        };
        let Some(fallback) = missing_monitor.resolve(&topology) else {
            return;
        };

        assert_eq!(resolved.monitor_id, MonitorId(7));
        assert_eq!(
            resolved.placement,
            WindowPlacement {
                origin: PointPx { x: 2810, y: 1344 },
                size: SizePx {
                    width: 390,
                    height: 96,
                },
            }
        );
        assert_eq!(fallback.monitor_id, MonitorId(1));
        assert_eq!(fallback.placement.origin, PointPx { x: 100, y: 100 });
    }

    #[test]
    fn dpi_and_text_scale_preserve_nonzero_text_dimensions() {
        let scale = DpiScale::new(144);
        let text = TextScalePercent::new(150);

        assert_eq!(scale.scale_text_px(12, text), 27);
        assert_eq!(scale.scale_text_px(1, text), 3);
        assert_eq!(
            DpiScale::new(48).scale_text_px(1, TextScalePercent::new(50)),
            1
        );
    }

    #[test]
    fn placement_clamps_window_to_current_work_area() {
        let work_area = RectPx {
            left: 100,
            top: 80,
            right: 2020,
            bottom: 1160,
        };
        let placement = WindowPlacement {
            origin: PointPx { x: 40, y: 40 },
            size: SizePx {
                width: 260,
                height: 64,
            },
        };

        assert_eq!(
            placement.clamp_to(work_area),
            WindowPlacement {
                origin: PointPx { x: 100, y: 80 },
                size: SizePx {
                    width: 260,
                    height: 64,
                },
            }
        );
    }

    #[test]
    fn placement_clamps_bottom_right_without_resizing() {
        let work_area = RectPx {
            left: 0,
            top: 0,
            right: 1280,
            bottom: 720,
        };
        let placement = WindowPlacement {
            origin: PointPx { x: 1240, y: 700 },
            size: SizePx {
                width: 260,
                height: 64,
            },
        };

        assert_eq!(
            placement.clamp_to(work_area).origin,
            PointPx { x: 1020, y: 656 }
        );
    }

    #[test]
    fn placement_anchors_at_work_area_origin_when_window_exceeds_area() {
        let work_area = RectPx {
            left: 50,
            top: 50,
            right: 200,
            bottom: 120,
        };
        let placement = WindowPlacement {
            origin: PointPx { x: 80, y: 90 },
            size: SizePx {
                width: 260,
                height: 96,
            },
        };

        assert_eq!(
            placement.clamp_to(work_area).origin,
            PointPx { x: 50, y: 50 }
        );
    }

    #[test]
    fn compact_window_style_policy_matches_non_activating_toolwindow_contract() {
        let policy = CompactWindowStylePolicy::default();

        assert!(policy.popup);
        assert!(policy.topmost);
        assert!(policy.tool_window);
        assert!(policy.no_activate);
        assert!(!policy.alt_tab_visible);
        assert!(!policy.permanently_click_through);
    }

    #[test]
    fn palette_hotkey_policy_maps_to_ctrl_shift_space_registration_chord(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let enabled = GlobalHotkeyPolicy::palette_default();
        let disabled = GlobalHotkeyPolicy {
            enabled: false,
            ..enabled
        };

        let Some(chord) = enabled.registration_chord() else {
            return Err(Box::new(std::io::Error::other(
                "enabled palette hotkey should register",
            )));
        };

        assert_eq!(chord.id, 1);
        assert_eq!(chord.modifiers, 0x0002 | 0x0004);
        assert_eq!(chord.virtual_key, 0x20);
        assert_eq!(disabled.registration_chord(), None);
        Ok(())
    }

    #[test]
    fn immersive_window_policy_hides_without_fighting_fullscreen_surfaces() {
        let policy = ImmersiveWindowPolicy::for_state(ImmersiveState::Fullscreen);

        assert_eq!(policy.visibility, WindowVisibility::Hidden);
        assert!(!policy.keep_topmost);
        assert!(!policy.replay_missed_animations_on_restore);
    }

    #[test]
    fn normal_window_policy_keeps_topmost_non_activating_compact_surface() {
        let policy = ImmersiveWindowPolicy::for_state(ImmersiveState::Normal);

        assert_eq!(policy.visibility, WindowVisibility::Visible);
        assert!(policy.keep_topmost);
        assert!(!policy.replay_missed_animations_on_restore);
    }

    #[test]
    fn compact_window_style_policy_maps_to_documented_win32_style_bits() {
        let bits = Win32StyleBits::from_policy(CompactWindowStylePolicy::default());

        assert!(bits.has_popup());
        assert!(bits.has_topmost());
        assert!(bits.has_tool_window());
        assert!(bits.has_no_activate());
        assert!(!bits.has_app_window());
        assert!(!bits.has_transparent());
    }

    #[cfg(target_env = "msvc")]
    #[test]
    fn style_bits_match_current_windows_sys_bindings() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            WS_EX_APPWINDOW as SYS_WS_EX_APPWINDOW, WS_EX_NOACTIVATE as SYS_WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW as SYS_WS_EX_TOOLWINDOW, WS_EX_TOPMOST as SYS_WS_EX_TOPMOST,
            WS_EX_TRANSPARENT as SYS_WS_EX_TRANSPARENT, WS_POPUP as SYS_WS_POPUP,
        };

        assert_eq!(WS_POPUP, SYS_WS_POPUP);
        assert_eq!(WS_EX_TOPMOST, SYS_WS_EX_TOPMOST);
        assert_eq!(WS_EX_TRANSPARENT, SYS_WS_EX_TRANSPARENT);
        assert_eq!(WS_EX_TOOLWINDOW, SYS_WS_EX_TOOLWINDOW);
        assert_eq!(WS_EX_APPWINDOW, SYS_WS_EX_APPWINDOW);
        assert_eq!(WS_EX_NOACTIVATE, SYS_WS_EX_NOACTIVATE);
    }

    #[cfg(target_env = "msvc")]
    #[test]
    fn hit_test_and_hotkey_values_match_current_windows_sys_bindings() {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            MOD_CONTROL as SYS_MOD_CONTROL, MOD_SHIFT as SYS_MOD_SHIFT, VK_SPACE as SYS_VK_SPACE,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            HTCAPTION as SYS_HTCAPTION, HTCLIENT as SYS_HTCLIENT, HTNOWHERE as SYS_HTNOWHERE,
            HTTRANSPARENT as SYS_HTTRANSPARENT,
        };

        assert_eq!(Win32HitTestCode::Transparent.value(), SYS_HTTRANSPARENT);
        assert_eq!(
            u32::try_from(Win32HitTestCode::Nowhere.value()),
            Ok(SYS_HTNOWHERE)
        );
        assert_eq!(
            u32::try_from(Win32HitTestCode::Client.value()),
            Ok(SYS_HTCLIENT)
        );
        assert_eq!(
            u32::try_from(Win32HitTestCode::Caption.value()),
            Ok(SYS_HTCAPTION)
        );
        assert_eq!(MOD_CONTROL, SYS_MOD_CONTROL);
        assert_eq!(MOD_SHIFT, SYS_MOD_SHIFT);
        assert_eq!(VK_SPACE, u32::from(SYS_VK_SPACE));
    }

    #[test]
    fn native_adapter_plan_reuses_non_activating_compact_window() {
        let placement = WindowPlacement {
            origin: PointPx { x: 100, y: 80 },
            size: SizePx {
                width: 260,
                height: 64,
            },
        };
        let hit_test = HitTestLayout {
            width_px: 260,
            height_px: 64,
            transparent_margin_px: 8,
            drag_grip_width_px: 32,
        };

        let plan = NativeWindowAdapterPlan::from_input(NativeWindowAdapterInput {
            compact_visible: true,
            immersive_state: ImmersiveState::Normal,
            placement,
            hit_test,
            style_policy: CompactWindowStylePolicy::default(),
            hotkey_policy: GlobalHotkeyPolicy::palette_default(),
        });

        assert!(plan.create_compact_window_if_missing);
        assert!(!plan.recreate_compact_window);
        assert!(!plan.destroy_compact_window_when_hidden);
        assert!(!plan.activate_on_show);
        assert_eq!(plan.visibility, WindowVisibility::Visible);
        assert!(plan.keep_topmost);
        assert_eq!(plan.placement, placement);
        assert_eq!(plan.hit_test, hit_test);
        assert!(plan.style_bits.has_no_activate());
        assert!(!plan.style_bits.has_transparent());
        assert_eq!(
            plan.hotkey,
            GlobalHotkeyPolicy::palette_default().registration_chord()
        );
    }

    #[test]
    fn native_adapter_plan_hides_immersive_without_destroying_or_replaying() {
        let plan = NativeWindowAdapterPlan::from_input(NativeWindowAdapterInput {
            compact_visible: true,
            immersive_state: ImmersiveState::Fullscreen,
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
        });

        assert_eq!(plan.visibility, WindowVisibility::Hidden);
        assert!(!plan.keep_topmost);
        assert!(!plan.destroy_compact_window_when_hidden);
        assert!(!plan.replay_missed_animations_on_restore);
        assert!(!plan.activate_on_show);
    }

    #[test]
    fn native_adapter_state_creates_once_across_repeated_visible_frames() {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let first = NativeWindowAdapterState::default().apply(plan);
        let second = first.apply(plan);

        assert_eq!(first.window_generation, 1);
        assert_eq!(second.window_generation, 1);
        assert_eq!(first.create_window_calls, 1);
        assert_eq!(second.create_window_calls, 1);
        assert_eq!(second.visibility, WindowVisibility::Visible);
        assert!(second.hotkey_registered);
        assert_eq!(second.hotkey_register_calls, 1);
        assert_eq!(second.hotkey_unregister_calls, 0);
    }

    #[test]
    fn native_adapter_state_hides_and_restores_without_recreate_or_hotkey_churn() {
        let visible = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let immersive = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Fullscreen,
        ));

        let state = NativeWindowAdapterState::default()
            .apply(visible)
            .apply(immersive)
            .apply(visible);

        assert_eq!(state.window_generation, 1);
        assert_eq!(state.create_window_calls, 1);
        assert_eq!(state.destroy_window_calls, 0);
        assert_eq!(state.visibility, WindowVisibility::Visible);
        assert!(state.hotkey_registered);
        assert_eq!(state.hotkey_register_calls, 1);
        assert_eq!(state.hotkey_unregister_calls, 0);
        assert_eq!(state.activation_attempts, 0);
    }

    #[test]
    fn native_adapter_actions_are_ordered_and_idempotent_for_visible_plan() {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let initial = NativeWindowAdapterState::default();

        assert_eq!(
            initial.actions_for(plan),
            vec![
                NativeWindowAdapterAction::CreateCompactWindow,
                NativeWindowAdapterAction::ApplyWindowStyles,
                NativeWindowAdapterAction::UpdateHitTestLayout,
                NativeWindowAdapterAction::MoveResize,
                NativeWindowAdapterAction::RegisterHotkey,
                NativeWindowAdapterAction::ShowNoActivate,
                NativeWindowAdapterAction::SetTopmost,
            ]
        );
        assert!(initial.apply(plan).actions_for(plan).is_empty());
    }

    #[test]
    fn native_adapter_commands_carry_backend_payloads_for_visible_plan(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));

        assert_eq!(
            NativeWindowAdapterState::default().commands_for(plan),
            vec![
                NativeWindowAdapterCommand::CreateCompactWindow,
                NativeWindowAdapterCommand::ApplyWindowStyles(plan.style_bits),
                NativeWindowAdapterCommand::UpdateHitTestLayout(plan.hit_test),
                NativeWindowAdapterCommand::MoveResize(plan.placement),
                NativeWindowAdapterCommand::RegisterHotkey(
                    GlobalHotkeyPolicy::palette_default()
                        .registration_chord()
                        .ok_or("missing test hotkey")
                        .map_err(std::io::Error::other)?
                ),
                NativeWindowAdapterCommand::ShowNoActivate,
                NativeWindowAdapterCommand::SetTopmost,
            ]
        );
        Ok(())
    }

    #[test]
    fn native_adapter_actions_hide_immersive_without_destroy_or_hotkey_churn() {
        let visible = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let immersive = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Fullscreen,
        ));
        let visible_state = NativeWindowAdapterState::default().apply(visible);

        assert_eq!(
            visible_state.actions_for(immersive),
            vec![
                NativeWindowAdapterAction::HideCompactWindow,
                NativeWindowAdapterAction::ClearTopmost,
            ]
        );
    }

    #[test]
    fn native_adapter_actions_update_hit_test_layout_without_window_churn() {
        let visible = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let mut changed_input = native_adapter_test_input(true, ImmersiveState::Normal);
        changed_input.hit_test.transparent_margin_px = 12;
        changed_input.hit_test.drag_grip_width_px = 40;
        let changed_hit_test = NativeWindowAdapterPlan::from_input(changed_input);
        let visible_state = NativeWindowAdapterState::default().apply(visible);

        assert_eq!(
            visible_state.actions_for(changed_hit_test),
            vec![NativeWindowAdapterAction::UpdateHitTestLayout]
        );
    }

    #[test]
    fn native_adapter_actions_diff_placement_style_and_hotkey_independently() {
        let visible = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let visible_state = NativeWindowAdapterState::default().apply(visible);
        let mut placement_input = native_adapter_test_input(true, ImmersiveState::Normal);
        placement_input.placement.origin.x = 140;
        let mut style_input = native_adapter_test_input(true, ImmersiveState::Normal);
        style_input.style_policy.topmost = false;
        let mut hotkey_input = native_adapter_test_input(true, ImmersiveState::Normal);
        hotkey_input.hotkey_policy.enabled = false;

        assert_eq!(
            visible_state.actions_for(NativeWindowAdapterPlan::from_input(placement_input)),
            vec![NativeWindowAdapterAction::MoveResize]
        );
        assert_eq!(
            visible_state.actions_for(NativeWindowAdapterPlan::from_input(style_input)),
            vec![NativeWindowAdapterAction::ApplyWindowStyles]
        );
        assert_eq!(
            visible_state.actions_for(NativeWindowAdapterPlan::from_input(hotkey_input)),
            vec![NativeWindowAdapterAction::UnregisterHotkey]
        );
    }

    #[test]
    fn native_adapter_driver_applies_ordered_actions_and_updates_state(
    ) -> Result<(), NativeWindowAdapterError> {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let mut driver = NativeWindowAdapterDriver::default();
        let mut sink = RecordingNativeWindowSink::default();

        let applied = driver.apply_plan(plan, &mut sink)?;

        assert_eq!(
            applied,
            vec![
                NativeWindowAdapterAction::CreateCompactWindow,
                NativeWindowAdapterAction::ApplyWindowStyles,
                NativeWindowAdapterAction::UpdateHitTestLayout,
                NativeWindowAdapterAction::MoveResize,
                NativeWindowAdapterAction::RegisterHotkey,
                NativeWindowAdapterAction::ShowNoActivate,
                NativeWindowAdapterAction::SetTopmost,
            ]
        );
        assert_eq!(sink.actions, applied);
        assert_eq!(driver.state().window_generation, 1);
        assert_eq!(driver.state().visibility, WindowVisibility::Visible);
        assert!(driver.state().hotkey_registered);
        assert!(driver.apply_plan(plan, &mut sink)?.is_empty());
        Ok(())
    }

    #[test]
    fn native_adapter_driver_applies_payload_commands_to_command_sink(
    ) -> Result<(), NativeWindowAdapterCommandError> {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let mut driver = NativeWindowAdapterDriver::default();
        let mut sink = RecordingNativeWindowCommandSink::default();

        let applied = driver.apply_plan_commands(plan, &mut sink)?;

        assert_eq!(
            applied,
            NativeWindowAdapterState::default().commands_for(plan)
        );
        assert_eq!(sink.commands, applied);
        assert_eq!(driver.state().visibility, WindowVisibility::Visible);
        Ok(())
    }

    #[test]
    fn native_adapter_driver_does_not_advance_state_after_sink_failure() {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let mut driver = NativeWindowAdapterDriver::default();
        let mut sink = RecordingNativeWindowSink::failing_on(NativeWindowAdapterAction::MoveResize);

        let result = driver.apply_plan(plan, &mut sink);

        assert_eq!(
            result,
            Err(NativeWindowAdapterError {
                failed_action: NativeWindowAdapterAction::MoveResize,
                applied_actions: vec![
                    NativeWindowAdapterAction::CreateCompactWindow,
                    NativeWindowAdapterAction::ApplyWindowStyles,
                    NativeWindowAdapterAction::UpdateHitTestLayout,
                ],
            })
        );
        assert_eq!(driver.state(), NativeWindowAdapterState::default());
    }

    #[test]
    fn native_adapter_driver_does_not_advance_state_after_command_sink_failure() {
        let plan = NativeWindowAdapterPlan::from_input(native_adapter_test_input(
            true,
            ImmersiveState::Normal,
        ));
        let mut driver = NativeWindowAdapterDriver::default();
        let mut sink = RecordingNativeWindowCommandSink::failing_on(
            NativeWindowAdapterCommand::MoveResize(plan.placement),
        );

        let result = driver.apply_plan_commands(plan, &mut sink);

        assert_eq!(
            result,
            Err(NativeWindowAdapterCommandError {
                failed_command: NativeWindowAdapterCommand::MoveResize(plan.placement),
                applied_commands: vec![
                    NativeWindowAdapterCommand::CreateCompactWindow,
                    NativeWindowAdapterCommand::ApplyWindowStyles(plan.style_bits),
                    NativeWindowAdapterCommand::UpdateHitTestLayout(plan.hit_test),
                ],
            })
        );
        assert_eq!(driver.state(), NativeWindowAdapterState::default());
    }

    #[derive(Default)]
    struct RecordingNativeWindowSink {
        actions: Vec<NativeWindowAdapterAction>,
        fail_on: Option<NativeWindowAdapterAction>,
    }

    impl RecordingNativeWindowSink {
        fn failing_on(action: NativeWindowAdapterAction) -> Self {
            Self {
                actions: Vec::new(),
                fail_on: Some(action),
            }
        }
    }

    impl NativeWindowActionSink for RecordingNativeWindowSink {
        fn apply_action(&mut self, action: NativeWindowAdapterAction) -> bool {
            if self.fail_on == Some(action) {
                return false;
            }
            self.actions.push(action);
            true
        }
    }

    #[derive(Default)]
    struct RecordingNativeWindowCommandSink {
        commands: Vec<NativeWindowAdapterCommand>,
        fail_on: Option<NativeWindowAdapterCommand>,
    }

    impl RecordingNativeWindowCommandSink {
        fn failing_on(command: NativeWindowAdapterCommand) -> Self {
            Self {
                commands: Vec::new(),
                fail_on: Some(command),
            }
        }
    }

    impl NativeWindowCommandSink for RecordingNativeWindowCommandSink {
        fn apply_command(&mut self, command: NativeWindowAdapterCommand) -> bool {
            if self.fail_on == Some(command) {
                return false;
            }
            self.commands.push(command);
            true
        }
    }

    fn native_adapter_test_input(
        compact_visible: bool,
        immersive_state: ImmersiveState,
    ) -> NativeWindowAdapterInput {
        NativeWindowAdapterInput {
            compact_visible,
            immersive_state,
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
        }
    }
}
