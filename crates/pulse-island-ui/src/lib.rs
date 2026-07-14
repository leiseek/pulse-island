//! Native Island UI view-model seam.
#![deny(missing_docs)]

use pulse_arbitration::PresentationPlan;
use pulse_domain::{
    Attention, BoundedText, DomainError, Lifecycle, ProviderId, RouteStrength, SafeSummary,
    TaskHealth, TaskId, TaskSnapshot, TimestampMs,
};
use pulse_routing::{label_for, RouteActionLabel};

/// Compact signal state rendered by the future native shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalState {
    /// No primary task is available.
    Idle,
    /// Primary task is running.
    Running,
    /// Primary task is waiting for user attention.
    Waiting,
    /// Primary task is failed or blocked.
    Failed,
    /// Primary task has completed.
    Completed,
    /// Primary task is observed/degraded/unknown.
    Observed,
}

/// Compact Island view model derived from an arbitration plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalViewModel {
    /// Opaque primary task id for UI correlation.
    pub primary_task_id: Option<String>,
    /// Primary signal state.
    pub state: SignalState,
    /// Number of secondary Peek items.
    pub overflow_count: usize,
    /// Route label supplied by routing policy for the primary task.
    pub primary_route_label: Option<RouteActionLabel>,
}

/// One row in the Peek attention queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeekRowViewModel {
    /// Opaque task id for row selection.
    pub task_id: String,
    /// Row state label.
    pub state: SignalState,
    /// Route label supplied by routing policy.
    pub route_label: Option<RouteActionLabel>,
}

/// Peek view model derived from the plan's existing ranked rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeekViewModel {
    /// Up to three rows already ranked by arbitration.
    pub rows: Vec<PeekRowViewModel>,
    /// Number of plan rows not shown because Peek is capped.
    pub hidden_count: usize,
}

impl PeekViewModel {
    /// Build Peek rows without re-running arbitration.
    pub fn from_plan(plan: &PresentationPlan) -> Self {
        let rows = plan
            .peek
            .iter()
            .take(3)
            .map(|task| PeekRowViewModel {
                task_id: task.task_id.0.as_str().to_owned(),
                state: state_for_lifecycle(task.lifecycle),
                route_label: label_for(task.route_strength),
            })
            .collect::<Vec<_>>();
        Self {
            rows,
            hidden_count: plan.peek.len().saturating_sub(3),
        }
    }
}

/// Focus Card view model for the current primary task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusCardViewModel {
    /// Opaque task id for correlation.
    pub task_id: String,
    /// Focus Card state label.
    pub state: SignalState,
    /// Primary route label supplied by routing policy.
    pub route_label: Option<RouteActionLabel>,
    /// P0 exposes no provider control actions.
    pub control_actions: Vec<ControlAction>,
}

impl FocusCardViewModel {
    /// Build a Focus Card for the current primary task.
    pub fn from_primary(plan: &PresentationPlan) -> Option<Self> {
        plan.primary.as_ref().map(|task| Self {
            task_id: task.task_id.0.as_str().to_owned(),
            state: state_for_lifecycle(task.lifecycle),
            route_label: label_for(task.route_strength),
            control_actions: Vec::new(),
        })
    }
}

/// Provider control action placeholder. P0 must not produce any values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAction {}

impl SignalViewModel {
    /// Build a signal view model without re-sorting or interpreting raw provider data.
    pub fn from_plan(plan: &PresentationPlan) -> Self {
        let primary = plan.primary.as_ref();
        Self {
            primary_task_id: primary.map(|task| task.task_id.0.as_str().to_owned()),
            state: primary
                .map(|task| state_for_lifecycle(task.lifecycle))
                .unwrap_or(SignalState::Idle),
            overflow_count: plan.peek.len(),
            primary_route_label: primary.and_then(|task| label_for(task.route_strength)),
        }
    }
}

/// Fuel candidate status supplied to the compact Signal presentation layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuelThreadCandidate {
    /// No trustworthy low-Fuel candidate is available.
    None,
    /// Arbitration selected a trustworthy low-Fuel candidate.
    TrustworthyLow,
}

/// Source of the compact Island primary story.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryStorySource {
    /// Primary story comes from the already-ranked presentation plan primary.
    PresentationPlanPrimary,
    /// No primary story is present.
    None,
}

/// Role assigned to Fuel Thread in compact Signal rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuelThreadRole {
    /// Fuel Thread is hidden.
    Hidden,
    /// Fuel Thread is visible but subordinate to task state.
    Secondary,
}

/// Compact Signal truth-priority decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalTruthPriorityDecision {
    /// Primary task id retained from the presentation plan.
    pub primary_task_id: Option<String>,
    /// Primary state retained from the presentation plan.
    pub primary_state: SignalState,
    /// Source for the primary story.
    pub primary_story_source: PrimaryStorySource,
    /// Fuel Thread role.
    pub fuel_thread_role: FuelThreadRole,
    /// Whether Fuel Thread is visible.
    pub fuel_thread_visible: bool,
    /// Timer-based primary rotation is forbidden in W2.
    pub timer_rotation_allowed: bool,
    /// Fuel must never override the primary task state.
    pub fuel_can_override_primary_state: bool,
}

impl SignalTruthPriorityDecision {
    /// Build the W2 compact Signal priority decision.
    pub fn from_signal(signal: &SignalViewModel, fuel: FuelThreadCandidate) -> Self {
        let has_primary = signal.primary_task_id.is_some();
        let fuel_thread_visible = has_primary && fuel == FuelThreadCandidate::TrustworthyLow;
        Self {
            primary_task_id: signal.primary_task_id.clone(),
            primary_state: signal.state,
            primary_story_source: if has_primary {
                PrimaryStorySource::PresentationPlanPrimary
            } else {
                PrimaryStorySource::None
            },
            fuel_thread_role: if fuel_thread_visible {
                FuelThreadRole::Secondary
            } else {
                FuelThreadRole::Hidden
            },
            fuel_thread_visible,
            timer_rotation_allowed: false,
            fuel_can_override_primary_state: false,
        }
    }
}

/// Compact Signal width policy using the priority order from the native UI spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactSignalLayoutPolicy {
    /// Available compact surface width.
    pub available_width_px: u32,
    /// Horizontal gap between visible slots.
    pub gap_width_px: u32,
    /// State glyph width.
    pub state_glyph_width_px: u32,
    /// Provider/workspace subject width.
    pub subject_width_px: u32,
    /// Short reason width.
    pub reason_width_px: u32,
    /// Active count badge width.
    pub active_count_width_px: u32,
    /// Secondary Fuel text width.
    pub secondary_fuel_width_px: u32,
}

/// Compact Signal layout decision before native text rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactSignalLayoutDecision {
    /// Whether the state glyph is visible.
    pub show_state_glyph: bool,
    /// Whether the subject slot is visible.
    pub show_subject: bool,
    /// Whether the subject must be truncated to preserve the core state.
    pub subject_truncated: bool,
    /// Whether the short reason is visible.
    pub show_reason: bool,
    /// Whether the active count badge is visible.
    pub show_active_count: bool,
    /// Whether secondary Fuel text is visible.
    pub show_secondary_fuel: bool,
}

impl CompactSignalLayoutPolicy {
    /// Evaluate which compact slots fit without violating the spec's priority order.
    pub fn evaluate(self) -> CompactSignalLayoutDecision {
        if self.available_width_px < self.state_glyph_width_px {
            return CompactSignalLayoutDecision {
                show_state_glyph: false,
                show_subject: false,
                subject_truncated: false,
                show_reason: false,
                show_active_count: false,
                show_secondary_fuel: false,
            };
        }

        let core_without_subject = self.state_glyph_width_px.saturating_add(self.gap_width_px);
        let show_subject = self.available_width_px > core_without_subject;
        let core_full_width = core_without_subject.saturating_add(self.subject_width_px);
        let subject_truncated = show_subject && core_full_width > self.available_width_px;
        if subject_truncated {
            return CompactSignalLayoutDecision {
                show_state_glyph: true,
                show_subject,
                subject_truncated,
                show_reason: false,
                show_active_count: false,
                show_secondary_fuel: false,
            };
        }

        let mut used_width = if show_subject {
            core_full_width
        } else {
            self.state_glyph_width_px
        };
        let show_reason = can_append_slot(
            used_width,
            self.gap_width_px,
            self.reason_width_px,
            self.available_width_px,
        );
        if show_reason {
            used_width = used_width
                .saturating_add(self.gap_width_px)
                .saturating_add(self.reason_width_px);
        }
        let show_active_count = show_reason
            && can_append_slot(
                used_width,
                self.gap_width_px,
                self.active_count_width_px,
                self.available_width_px,
            );
        if show_active_count {
            used_width = used_width
                .saturating_add(self.gap_width_px)
                .saturating_add(self.active_count_width_px);
        }
        let show_secondary_fuel = show_active_count
            && can_append_slot(
                used_width,
                self.gap_width_px,
                self.secondary_fuel_width_px,
                self.available_width_px,
            );

        CompactSignalLayoutDecision {
            show_state_glyph: true,
            show_subject,
            subject_truncated,
            show_reason,
            show_active_count,
            show_secondary_fuel,
        }
    }
}

fn can_append_slot(used_width: u32, gap_width: u32, slot_width: u32, available_width: u32) -> bool {
    used_width
        .saturating_add(gap_width)
        .saturating_add(slot_width)
        <= available_width
}

fn state_for_lifecycle(lifecycle: Lifecycle) -> SignalState {
    match lifecycle {
        Lifecycle::Running => SignalState::Running,
        Lifecycle::WaitingUser | Lifecycle::Limited => SignalState::Waiting,
        Lifecycle::Failed => SignalState::Failed,
        Lifecycle::Completed => SignalState::Completed,
        Lifecycle::Observed | Lifecycle::Unknown => SignalState::Observed,
    }
}

/// Source of current presentation plans for the UI.
pub trait PresentationPlanSource {
    /// Return the current plan snapshot.
    fn current_plan(&self) -> PresentationPlan;

    /// Subscribe to plan changes through the UI-facing seam.
    fn subscribe(&self, callback: PlanChangedCallback<'_>);
}

/// Callback invoked when a presentation plan is available to the UI.
pub type PlanChangedCallback<'a> = &'a mut dyn FnMut(PresentationPlan);

/// Deterministic mock source used by W2 Spike A.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockPresentationPlanSource {
    plan: PresentationPlan,
}

impl MockPresentationPlanSource {
    /// Create a mock source from a fixed presentation plan.
    pub fn new(plan: PresentationPlan) -> Self {
        Self { plan }
    }
}

impl PresentationPlanSource for MockPresentationPlanSource {
    fn current_plan(&self) -> PresentationPlan {
        self.plan.clone()
    }

    fn subscribe(&self, callback: PlanChangedCallback<'_>) {
        callback(self.plan.clone());
    }
}

/// Spike A deterministic scenario identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MockScenarioId {
    /// S0: no active task; Island is parked or hidden.
    S0IdleParked,
    /// S1: one running task.
    S1OneRunningTask,
    /// S2: one waiting task with background work.
    S2WaitingWithBackgroundWork,
    /// S3: one failed task.
    S3FailedTask,
    /// S4: aggregate active work with low Fuel.
    S4AggregateActiveFuelLow,
    /// S5: degraded observed state.
    S5DegradedObserved,
    /// S6: completion settle-out.
    S6CompletionSettleOut,
    /// S7: deterministic rapid state-change sequence.
    S7RapidStateChanges,
    /// S8: immersive/fullscreen suppression simulation.
    S8ImmersiveSimulation,
}

/// Shell environment switches that affect visibility and motion, not task truth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellEnvironment {
    /// Whether fullscreen/presentation policy currently suppresses the Island.
    pub immersive_active: bool,
    /// Whether reduced motion should remove pulses and scale movement.
    pub reduced_motion: bool,
    /// Whether high-contrast rendering tokens should be selected.
    pub high_contrast: bool,
}

/// UI motion policy selected from environment and scenario state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionPolicy {
    /// Normal compositor-owned animation is allowed.
    Normal,
    /// Reduced-motion treatment keeps meaning but removes pulsing/scale movement.
    Reduced,
    /// Animation must stop while the Island is suppressed.
    Stopped,
}

/// Deterministic mock scenario used by Spike A.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockScenario {
    /// Stable scenario id from the Spike A catalog.
    pub id: MockScenarioId,
    /// Human-readable scenario name for diagnostics.
    pub name: &'static str,
    /// Initial mocked presentation-plan source.
    pub source: MockPresentationPlanSource,
    /// Optional deterministic transition sequence for scenario runners.
    pub transitions: Vec<PresentationPlan>,
    /// Environment policy applied to the shell.
    pub environment: ShellEnvironment,
}

/// Combined Shell view model for Signal, Peek, Focus Card, and shell policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellViewModel {
    /// Compact Signal model.
    pub signal: SignalViewModel,
    /// Peek rows derived from the current plan.
    pub peek: PeekViewModel,
    /// Focus Card for the primary task, if any.
    pub focus_card: Option<FocusCardViewModel>,
    /// Whether the compact Island should be visible.
    pub compact_visible: bool,
    /// Whether the Command Palette is visible.
    pub palette_visible: bool,
    /// Motion policy selected for the shell.
    pub motion_policy: MotionPolicy,
    /// Whether high-contrast tokens should be used by the renderer.
    pub high_contrast: bool,
}

/// Currently opened user surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenSurface {
    /// No expanded surface is open.
    None,
    /// Peek surface is open.
    Peek,
    /// Focus Card surface is open.
    FocusCard,
    /// Command Palette surface is open.
    CommandPalette,
}

/// Logical keyboard focus owner for W2 focus-policy tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusOwner {
    /// The previously active external app keeps focus.
    ExternalApp,
    /// Focus Card owns keyboard focus after explicit row selection.
    FocusCard,
    /// Command Palette owns keyboard focus after explicit shortcut/action.
    CommandPalette,
}

/// User or passive shell event handled by the W2 pure interaction seam.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellUserEvent {
    /// Presentation state changed passively.
    PassivePlanUpdate,
    /// User clicked the compact Island.
    CompactClicked,
    /// User invoked the global Command Palette shortcut.
    PaletteShortcut,
    /// User clicked a Peek row.
    PeekRowClicked {
        /// Opaque task id selected by the user.
        task_id: String,
    },
    /// User pressed Escape.
    Escape,
}

/// Policy controlling whether a global shortcut may invoke Command Palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteInvocationPolicy {
    /// Whether the global shortcut is enabled.
    pub global_shortcut_enabled: bool,
    /// Whether Palette may open while immersive suppression is active.
    pub allow_during_immersive: bool,
}

impl Default for PaletteInvocationPolicy {
    fn default() -> Self {
        Self {
            global_shortcut_enabled: true,
            allow_during_immersive: true,
        }
    }
}

/// Pure shell interaction state for focus/visibility policy before real Win32 wiring.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellInteractionState {
    /// Currently open expanded surface.
    pub open_surface: OpenSurface,
    /// Logical focus owner.
    pub focus_owner: FocusOwner,
    /// Focus Card task id selected through an explicit user action.
    pub focused_task_id: Option<String>,
}

impl Default for ShellInteractionState {
    fn default() -> Self {
        Self {
            open_surface: OpenSurface::None,
            focus_owner: FocusOwner::ExternalApp,
            focused_task_id: None,
        }
    }
}

impl ShellInteractionState {
    /// Apply one pure shell event and return the next interaction state.
    pub fn apply(self, event: ShellUserEvent) -> Self {
        match event {
            ShellUserEvent::PassivePlanUpdate => self,
            ShellUserEvent::CompactClicked => Self {
                open_surface: OpenSurface::Peek,
                focus_owner: FocusOwner::ExternalApp,
                focused_task_id: None,
            },
            ShellUserEvent::PaletteShortcut => Self {
                open_surface: OpenSurface::CommandPalette,
                focus_owner: FocusOwner::CommandPalette,
                focused_task_id: None,
            },
            ShellUserEvent::PeekRowClicked { task_id } => Self {
                open_surface: OpenSurface::FocusCard,
                focus_owner: FocusOwner::FocusCard,
                focused_task_id: Some(task_id),
            },
            ShellUserEvent::Escape => Self::default(),
        }
    }

    /// Apply the global Palette shortcut only when policy allows invocation.
    pub fn apply_palette_invocation(
        self,
        policy: PaletteInvocationPolicy,
        environment: ShellEnvironment,
    ) -> Self {
        if !policy.global_shortcut_enabled {
            return self;
        }
        if environment.immersive_active && !policy.allow_during_immersive {
            return self;
        }
        self.apply(ShellUserEvent::PaletteShortcut)
    }
}

/// Pure shell surface lifecycle counters used before real HWND handle accounting exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellSurfaceLifecycle {
    /// Stable compact window generation for the shell lifetime.
    pub window_generation: u32,
    /// Currently active transient surfaces such as Peek, Focus Card, or Palette.
    pub active_transient_surfaces: u32,
    /// Maximum transient surfaces observed during the run.
    pub max_active_transient_surfaces: u32,
    /// Number of open/close surface cycles completed.
    pub open_close_cycles: u32,
    interaction: ShellInteractionState,
}

impl Default for ShellSurfaceLifecycle {
    fn default() -> Self {
        Self {
            window_generation: 1,
            active_transient_surfaces: 0,
            max_active_transient_surfaces: 0,
            open_close_cycles: 0,
            interaction: ShellInteractionState::default(),
        }
    }
}

impl ShellSurfaceLifecycle {
    /// Apply one shell event and update pure lifecycle counters.
    pub fn apply(mut self, event: ShellUserEvent) -> Self {
        let previous_surface = self.interaction.open_surface;
        self.interaction = self.interaction.apply(event);
        let next_surface = self.interaction.open_surface;
        self.active_transient_surfaces = if next_surface == OpenSurface::None {
            0
        } else {
            1
        };
        self.max_active_transient_surfaces = self
            .max_active_transient_surfaces
            .max(self.active_transient_surfaces);
        if previous_surface != OpenSurface::None && next_surface == OpenSurface::None {
            self.open_close_cycles = self.open_close_cycles.saturating_add(1);
        }
        self
    }
}

/// USER/GDI handle snapshot for focused surface stability diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceHandleSnapshot {
    /// USER handle count.
    pub user_handles: u32,
    /// GDI handle count.
    pub gdi_handles: u32,
}

impl SurfaceHandleSnapshot {
    /// Create a focused surface handle snapshot.
    pub const fn new(user_handles: u32, gdi_handles: u32) -> Self {
        Self {
            user_handles,
            gdi_handles,
        }
    }
}

/// Report proving focused surface handle counts stay stable after repeated cycles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceHandleStabilityReport {
    /// Required Peek/Focus open-close cycles for Spike A.
    pub required_open_close_cycles: u32,
    /// Actual open-close cycles observed.
    pub actual_open_close_cycles: u32,
    /// USER handle count growth.
    pub user_handle_growth: u32,
    /// GDI handle count growth.
    pub gdi_handle_growth: u32,
    /// Whether handle counts remained stable across enough cycles.
    pub passed: bool,
}

impl SurfaceHandleStabilityReport {
    /// Build a surface handle report from lifecycle counters and diagnostic snapshots.
    pub fn from_lifecycle(
        lifecycle: ShellSurfaceLifecycle,
        baseline: SurfaceHandleSnapshot,
        final_snapshot: SurfaceHandleSnapshot,
    ) -> Self {
        let required_open_close_cycles = 2_000;
        let actual_open_close_cycles = lifecycle.open_close_cycles;
        let user_handle_growth = final_snapshot
            .user_handles
            .saturating_sub(baseline.user_handles);
        let gdi_handle_growth = final_snapshot
            .gdi_handles
            .saturating_sub(baseline.gdi_handles);
        let passed = actual_open_close_cycles >= required_open_close_cycles
            && lifecycle.active_transient_surfaces == 0
            && user_handle_growth == 0
            && gdi_handle_growth == 0;

        Self {
            required_open_close_cycles,
            actual_open_close_cycles,
            user_handle_growth,
            gdi_handle_growth,
            passed,
        }
    }
}

/// Accessible compact Signal metadata for screen readers and non-color review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibleSignalViewModel {
    /// Full accessible name assembled from non-content metadata.
    pub name: String,
    /// Textual state label independent from color.
    pub state_label: &'static str,
    /// Whether color is the only state indicator. W2 must keep this false.
    pub uses_color_as_sole_indicator: bool,
}

impl AccessibleSignalViewModel {
    /// Build accessible Signal metadata from the compact Signal view model.
    pub fn from_signal(signal: &SignalViewModel) -> Self {
        let state_label = state_accessible_label(signal.state);
        let mut parts = vec![state_label.to_owned()];
        if let Some(task_id) = signal.primary_task_id.as_deref() {
            parts.push(format!("task {task_id}"));
        }
        if signal.overflow_count > 0 {
            parts.push(format!("{} more active", signal.overflow_count));
        }
        if let Some(route_label) = signal.primary_route_label {
            parts.push(route_accessible_label(route_label).to_owned());
        }
        Self {
            name: parts.join("; "),
            state_label,
            uses_color_as_sole_indicator: false,
        }
    }
}

/// Animation affordance policy after accessibility and immersive preferences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationPolicy {
    /// Whether state changes remain visible.
    pub state_change_visible: bool,
    /// Whether pulsing animation is allowed.
    pub pulse_allowed: bool,
    /// Whether scale/slide motion is allowed.
    pub scale_allowed: bool,
}

impl AnimationPolicy {
    /// Return the animation policy for a state and motion mode.
    pub fn for_state(state: SignalState, motion: MotionPolicy) -> Self {
        match motion {
            MotionPolicy::Stopped => Self {
                state_change_visible: state != SignalState::Idle,
                pulse_allowed: false,
                scale_allowed: false,
            },
            MotionPolicy::Reduced => Self {
                state_change_visible: true,
                pulse_allowed: false,
                scale_allowed: false,
            },
            MotionPolicy::Normal => Self {
                state_change_visible: true,
                pulse_allowed: matches!(state, SignalState::Running | SignalState::Waiting),
                scale_allowed: true,
            },
        }
    }
}

/// W2 compositor-owned animation class from the native shell specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositorAnimationClass {
    /// Island becomes visible.
    Arrival,
    /// Primary narrative or state changes.
    StateTransition,
    /// Waiting/failure attention treatment.
    AttentionPulse,
    /// Peek or Focus Card opens.
    Expansion,
    /// Low-Fuel threshold cue.
    FuelCue,
    /// Brief task completion confirmation.
    Completion,
}

/// Pure animation plan consumed by the future native compositor adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositorAnimationPlan {
    /// Animation class being planned.
    pub class: CompositorAnimationClass,
    /// Whether the animation is intended for compositor-owned property animation.
    pub compositor_owned: bool,
    /// Whether an app-side frame loop is allowed for this animation.
    pub app_side_frame_loop_allowed: bool,
    /// Planned duration in milliseconds. Zero means no animation should run.
    pub duration_ms: u32,
    /// Whether the animation may be interrupted by state changes or Escape.
    pub interruptible: bool,
    /// Maximum repetitions for sparse pulse classes.
    pub max_repetitions: Option<u32>,
    /// Whether the animation must end in a stable static visual state.
    pub settles_to_static: bool,
    /// Whether pulse emphasis is allowed.
    pub pulse_allowed: bool,
    /// Whether scale or slide movement is allowed.
    pub scale_allowed: bool,
}

impl CompositorAnimationPlan {
    /// Return the W2 animation plan for a class and shell motion policy.
    pub fn for_class(class: CompositorAnimationClass, motion: MotionPolicy) -> Self {
        if motion == MotionPolicy::Stopped {
            return Self {
                class,
                compositor_owned: false,
                app_side_frame_loop_allowed: false,
                duration_ms: 0,
                interruptible: true,
                max_repetitions: None,
                settles_to_static: true,
                pulse_allowed: false,
                scale_allowed: false,
            };
        }

        let base = match class {
            CompositorAnimationClass::Arrival => Self {
                class,
                compositor_owned: true,
                app_side_frame_loop_allowed: false,
                duration_ms: 120,
                interruptible: false,
                max_repetitions: None,
                settles_to_static: true,
                pulse_allowed: false,
                scale_allowed: true,
            },
            CompositorAnimationClass::StateTransition => Self {
                class,
                compositor_owned: true,
                app_side_frame_loop_allowed: false,
                duration_ms: 160,
                interruptible: true,
                max_repetitions: None,
                settles_to_static: true,
                pulse_allowed: false,
                scale_allowed: false,
            },
            CompositorAnimationClass::AttentionPulse => Self {
                class,
                compositor_owned: true,
                app_side_frame_loop_allowed: false,
                duration_ms: 600,
                interruptible: true,
                max_repetitions: Some(3),
                settles_to_static: true,
                pulse_allowed: true,
                scale_allowed: false,
            },
            CompositorAnimationClass::Expansion => Self {
                class,
                compositor_owned: true,
                app_side_frame_loop_allowed: false,
                duration_ms: 140,
                interruptible: true,
                max_repetitions: None,
                settles_to_static: true,
                pulse_allowed: false,
                scale_allowed: true,
            },
            CompositorAnimationClass::FuelCue => Self {
                class,
                compositor_owned: true,
                app_side_frame_loop_allowed: false,
                duration_ms: 180,
                interruptible: true,
                max_repetitions: None,
                settles_to_static: true,
                pulse_allowed: false,
                scale_allowed: false,
            },
            CompositorAnimationClass::Completion => Self {
                class,
                compositor_owned: true,
                app_side_frame_loop_allowed: false,
                duration_ms: 220,
                interruptible: false,
                max_repetitions: None,
                settles_to_static: true,
                pulse_allowed: false,
                scale_allowed: false,
            },
        };

        if motion == MotionPolicy::Reduced {
            Self {
                pulse_allowed: false,
                scale_allowed: false,
                ..base
            }
        } else {
            base
        }
    }
}

/// Contrast mode selected for visual tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContrastMode {
    /// Standard Pulse visual tokens.
    Standard,
    /// System high-contrast-compatible tokens.
    HighContrast,
}

/// Visual accessibility policy derived from shell environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualAccessibilityPolicy {
    /// Selected contrast mode.
    pub contrast_mode: ContrastMode,
    /// Whether renderer should use system contrast tokens.
    pub uses_system_contrast_tokens: bool,
}

impl VisualAccessibilityPolicy {
    /// Build visual accessibility policy from shell environment flags.
    pub fn from_environment(environment: ShellEnvironment) -> Self {
        if environment.high_contrast {
            Self {
                contrast_mode: ContrastMode::HighContrast,
                uses_system_contrast_tokens: true,
            }
        } else {
            Self {
                contrast_mode: ContrastMode::Standard,
                uses_system_contrast_tokens: false,
            }
        }
    }
}

/// Keyboard command handled by deterministic Peek/Palette navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardCommand {
    /// Move selection up.
    ArrowUp,
    /// Move selection down.
    ArrowDown,
    /// Activate current selection.
    Enter,
}

/// Initial Command Palette command identifier for the W2 shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandPaletteCommandId {
    /// Open the active task through the current route label.
    OpenActiveTask,
    /// Open the current workspace route.
    OpenWorkspace,
    /// Show the lowest Fuel window when trusted Fuel metadata exists.
    ShowLowestFuelWindow,
    /// Open the provider's official usage destination.
    OpenProviderUsage,
    /// Show active agents.
    ShowActiveAgents,
    /// Pin the current task.
    PinTask,
    /// Follow the current task.
    FollowTask,
    /// Mute the current task or workspace.
    MuteTaskOrWorkspace,
    /// Open Pulse settings.
    OpenPulseSettings,
}

/// One Command Palette entry exposed by the W2 shell model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandPaletteCommand {
    /// Stable command id.
    pub id: CommandPaletteCommandId,
    /// User-facing label.
    pub label: &'static str,
    /// Whether this command controls a provider process or session.
    pub provider_control: bool,
    /// Whether this command is high-risk and therefore disallowed in P0.
    pub high_risk: bool,
}

/// Command Palette view model for explicit keyboard-focused navigation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandPaletteViewModel {
    /// Deterministic initial command list.
    pub commands: Vec<CommandPaletteCommand>,
}

impl CommandPaletteViewModel {
    /// Build the P0 Command Palette shell without provider control commands.
    pub fn p0() -> Self {
        Self {
            commands: vec![
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::OpenActiveTask,
                    "Open active task",
                ),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::OpenWorkspace,
                    "Open workspace",
                ),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::ShowLowestFuelWindow,
                    "Show lowest fuel window",
                ),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::OpenProviderUsage,
                    "Open provider usage",
                ),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::ShowActiveAgents,
                    "Show active agents",
                ),
                CommandPaletteCommand::navigation(CommandPaletteCommandId::PinTask, "Pin task"),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::FollowTask,
                    "Follow task",
                ),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::MuteTaskOrWorkspace,
                    "Mute task/workspace",
                ),
                CommandPaletteCommand::navigation(
                    CommandPaletteCommandId::OpenPulseSettings,
                    "Open Pulse settings",
                ),
            ],
        }
    }
}

impl CommandPaletteCommand {
    const fn navigation(id: CommandPaletteCommandId, label: &'static str) -> Self {
        Self {
            id,
            label,
            provider_control: false,
            high_risk: false,
        }
    }
}

/// Deterministic keyboard navigation state for Peek and Command Palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardNavigationState {
    /// Surface receiving keyboard navigation.
    pub surface: OpenSurface,
    /// Number of selectable rows.
    pub item_count: usize,
    /// Currently selected row.
    pub selected_index: Option<usize>,
    /// Activated row, if Enter has been pressed.
    pub activated_index: Option<usize>,
}

impl KeyboardNavigationState {
    /// Create a navigation state with first row selected when rows exist.
    pub fn new(surface: OpenSurface, item_count: usize) -> Self {
        Self {
            surface,
            item_count,
            selected_index: if item_count == 0 { None } else { Some(0) },
            activated_index: None,
        }
    }

    /// Apply one keyboard command.
    pub fn apply(self, command: KeyboardCommand) -> Self {
        let Some(selected_index) = self.selected_index else {
            return self;
        };
        match command {
            KeyboardCommand::ArrowUp => Self {
                selected_index: Some(selected_index.saturating_sub(1)),
                activated_index: None,
                ..self
            },
            KeyboardCommand::ArrowDown => Self {
                selected_index: Some((selected_index + 1).min(self.item_count.saturating_sub(1))),
                activated_index: None,
                ..self
            },
            KeyboardCommand::Enter => Self {
                activated_index: Some(selected_index),
                ..self
            },
        }
    }
}

/// Gate A measurement metric identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasurementMetricName {
    /// Compact idle P95 memory in megabytes.
    CompactIdleMemoryP95Mb,
    /// Focus Card P95 memory in megabytes.
    FocusCardMemoryP95Mb,
    /// Hard process tree ceiling in megabytes.
    ProcessTreeCeilingMb,
    /// Idle average CPU percentage.
    IdleAverageCpuPercent,
    /// Running-state average CPU percentage.
    RunningAverageCpuPercent,
    /// State update to visible response P95 in milliseconds.
    StateUpdateLatencyP95Ms,
    /// Palette shortcut to first frame P95 in milliseconds.
    PaletteShortcutLatencyP95Ms,
    /// Thirty-minute steady-state memory growth in megabytes.
    SteadyStateMemoryGrowthMb,
    /// Whether a static-state app-side frame loop exists.
    StaticStateFrameLoop,
}

/// Direction used when evaluating a metric against its target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasurementComparator {
    /// Actual value must be less than or equal to target.
    LessThanOrEqual,
    /// Actual value must be strictly less than target.
    LessThan,
    /// Actual value must equal target.
    Equal,
}

/// One Gate A measurement target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasurementTarget {
    /// Metric name.
    pub name: MeasurementMetricName,
    /// Numeric target value.
    pub target: f64,
    /// Comparator used against target.
    pub comparator: MeasurementComparator,
}

/// Measurement policy exported by the Spike A harness.
#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementPolicy {
    /// Metrics required by Gate A.
    pub metrics: Vec<MeasurementTarget>,
    /// Measurements are diagnostic metadata only.
    pub diagnostic_metadata_only: bool,
    /// Measurements must not include task content.
    pub includes_task_content: bool,
}

impl MeasurementPolicy {
    /// Return the Gate A measurement policy from Spike A.
    pub fn gate_a() -> Self {
        Self {
            metrics: vec![
                MeasurementTarget {
                    name: MeasurementMetricName::CompactIdleMemoryP95Mb,
                    target: 45.0,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::FocusCardMemoryP95Mb,
                    target: 85.0,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::ProcessTreeCeilingMb,
                    target: 100.0,
                    comparator: MeasurementComparator::LessThan,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::IdleAverageCpuPercent,
                    target: 0.10,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::RunningAverageCpuPercent,
                    target: 0.35,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::StateUpdateLatencyP95Ms,
                    target: 120.0,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::PaletteShortcutLatencyP95Ms,
                    target: 80.0,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::SteadyStateMemoryGrowthMb,
                    target: 2.0,
                    comparator: MeasurementComparator::LessThanOrEqual,
                },
                MeasurementTarget {
                    name: MeasurementMetricName::StaticStateFrameLoop,
                    target: 0.0,
                    comparator: MeasurementComparator::Equal,
                },
            ],
            diagnostic_metadata_only: true,
            includes_task_content: false,
        }
    }
}

/// One diagnostic measurement sample captured by the Spike A harness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasurementSample {
    /// Metric name.
    pub name: MeasurementMetricName,
    /// Numeric sample value.
    pub value: f64,
}

impl MeasurementSample {
    /// Create one diagnostic sample.
    pub const fn new(name: MeasurementMetricName, value: f64) -> Self {
        Self { name, value }
    }
}

/// Aggregated result for one Gate A measurement target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasurementResult {
    /// Metric name.
    pub name: MeasurementMetricName,
    /// Aggregated value used for comparison.
    pub actual: f64,
    /// Required target.
    pub target: f64,
    /// Comparator used against the target.
    pub comparator: MeasurementComparator,
    /// Whether this metric passed.
    pub passed: bool,
}

/// Gate A measurement report assembled from diagnostic metadata only.
#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementReport {
    /// Per-metric evaluation results.
    pub results: Vec<MeasurementResult>,
    /// Metrics missing from the provided samples.
    pub missing_metrics: Vec<MeasurementMetricName>,
    /// Whether the report is diagnostic metadata only and excludes task content.
    pub diagnostic_metadata_only: bool,
    /// Overall pass/fail result.
    pub passed: bool,
}

impl MeasurementReport {
    /// Build a report by aggregating samples against the supplied policy.
    pub fn from_samples(policy: MeasurementPolicy, samples: Vec<MeasurementSample>) -> Self {
        let mut results = Vec::new();
        let mut missing_metrics = Vec::new();
        for target in policy.metrics {
            let values = samples
                .iter()
                .filter(|sample| sample.name == target.name)
                .map(|sample| sample.value)
                .collect::<Vec<_>>();
            let Some(actual) = aggregate_metric(target.name, values) else {
                missing_metrics.push(target.name);
                continue;
            };
            let passed = compare_measurement(actual, target.target, target.comparator);
            results.push(MeasurementResult {
                name: target.name,
                actual,
                target: target.target,
                comparator: target.comparator,
                passed,
            });
        }
        let results_passed = results.iter().all(|result| result.passed);
        let diagnostic_metadata_only =
            policy.diagnostic_metadata_only && !policy.includes_task_content;
        let passed = diagnostic_metadata_only && results_passed && missing_metrics.is_empty();
        Self {
            results,
            missing_metrics,
            diagnostic_metadata_only,
            passed,
        }
    }
}

/// Performance overlay visibility mode for the Spike A runner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayMode {
    /// Normal user-facing mode. Overlay must be hidden.
    Normal,
    /// Diagnostics mode. Overlay may show measurement metadata.
    Diagnostics,
}

/// One row in the diagnostics-only performance overlay.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerformanceOverlayRow {
    /// Non-content metric label.
    pub label: &'static str,
    /// Aggregated metric value.
    pub actual: f64,
    /// Required metric target.
    pub target: f64,
    /// Whether the metric passed its target.
    pub passed: bool,
}

/// Diagnostics-only performance overlay model.
#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceOverlayViewModel {
    /// Whether the overlay is visible.
    pub visible: bool,
    /// Metric rows shown by the overlay.
    pub rows: Vec<PerformanceOverlayRow>,
    /// Whether all rows passed and no required metrics are missing.
    pub passed: bool,
    /// Overlay contains diagnostic metadata only.
    pub diagnostic_metadata_only: bool,
    /// Overlay must not contain task content.
    pub includes_task_content: bool,
}

impl PerformanceOverlayViewModel {
    /// Build an overlay from a Gate A measurement report.
    pub fn from_report(mode: OverlayMode, report: MeasurementReport) -> Self {
        if mode == OverlayMode::Normal {
            return Self {
                visible: false,
                rows: Vec::new(),
                passed: false,
                diagnostic_metadata_only: true,
                includes_task_content: false,
            };
        }

        let rows = report
            .results
            .iter()
            .map(|result| PerformanceOverlayRow {
                label: measurement_overlay_label(result.name),
                actual: result.actual,
                target: result.target,
                passed: result.passed,
            })
            .collect::<Vec<_>>();
        Self {
            visible: true,
            rows,
            passed: report.passed,
            diagnostic_metadata_only: report.diagnostic_metadata_only,
            includes_task_content: false,
        }
    }
}

fn measurement_overlay_label(name: MeasurementMetricName) -> &'static str {
    match name {
        MeasurementMetricName::CompactIdleMemoryP95Mb => "compact_idle_memory_p95_mb",
        MeasurementMetricName::FocusCardMemoryP95Mb => "focus_card_memory_p95_mb",
        MeasurementMetricName::ProcessTreeCeilingMb => "process_tree_ceiling_mb",
        MeasurementMetricName::IdleAverageCpuPercent => "idle_average_cpu_percent",
        MeasurementMetricName::RunningAverageCpuPercent => "running_average_cpu_percent",
        MeasurementMetricName::StateUpdateLatencyP95Ms => "state_update_latency_p95_ms",
        MeasurementMetricName::PaletteShortcutLatencyP95Ms => "palette_shortcut_latency_p95_ms",
        MeasurementMetricName::SteadyStateMemoryGrowthMb => "steady_state_memory_growth_mb",
        MeasurementMetricName::StaticStateFrameLoop => "static_state_frame_loop",
    }
}

fn aggregate_metric(name: MeasurementMetricName, values: Vec<f64>) -> Option<f64> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    match name {
        MeasurementMetricName::CompactIdleMemoryP95Mb
        | MeasurementMetricName::FocusCardMemoryP95Mb
        | MeasurementMetricName::StateUpdateLatencyP95Ms
        | MeasurementMetricName::PaletteShortcutLatencyP95Ms => percentile_95(values),
        MeasurementMetricName::IdleAverageCpuPercent
        | MeasurementMetricName::RunningAverageCpuPercent => average(values),
        MeasurementMetricName::ProcessTreeCeilingMb
        | MeasurementMetricName::SteadyStateMemoryGrowthMb
        | MeasurementMetricName::StaticStateFrameLoop => maximum(values),
    }
}

fn percentile_95(mut values: Vec<f64>) -> Option<f64> {
    values.sort_by(f64::total_cmp);
    let index = values
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    values.get(index).copied()
}

fn average(values: Vec<f64>) -> Option<f64> {
    let count = values.len() as f64;
    Some(values.iter().sum::<f64>() / count)
}

fn maximum(values: Vec<f64>) -> Option<f64> {
    values.into_iter().reduce(f64::max)
}

fn compare_measurement(actual: f64, target: f64, comparator: MeasurementComparator) -> bool {
    match comparator {
        MeasurementComparator::LessThanOrEqual => actual <= target,
        MeasurementComparator::LessThan => actual < target,
        MeasurementComparator::Equal => actual == target,
    }
}

/// Why the UI may redraw in a static state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedrawReason {
    /// No app-side redraw is needed.
    None,
    /// State is handled by compositor-owned animation only.
    CompositorOnly,
}

/// Static render policy for avoiding app-side frame loops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticRenderPolicy {
    /// Whether app-side frame loop is allowed.
    pub app_side_frame_loop_allowed: bool,
    /// Reason a redraw can occur.
    pub redraw_reason: RedrawReason,
}

impl StaticRenderPolicy {
    /// Derive static render policy from shell state and motion mode.
    pub fn for_shell(shell: &ShellViewModel, motion: MotionPolicy) -> Self {
        if !shell.compact_visible || matches!(motion, MotionPolicy::Stopped | MotionPolicy::Reduced)
        {
            Self {
                app_side_frame_loop_allowed: false,
                redraw_reason: RedrawReason::None,
            }
        } else {
            Self {
                app_side_frame_loop_allowed: false,
                redraw_reason: RedrawReason::CompositorOnly,
            }
        }
    }
}

/// Cache invalidation trigger for native render resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderCacheInvalidation {
    /// Monitor DPI changed.
    DpiChanged,
    /// System or Pulse theme changed.
    ThemeChanged,
    /// Font or text-scale settings changed.
    FontChanged,
    /// State-dependent layout changed.
    StateLayoutChanged,
}

/// W2 render-cache policy for bounded native resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCachePolicy {
    /// Maximum cached short text layouts.
    pub max_cached_text_layouts: usize,
    /// Whether text layout cache is bounded.
    pub text_layout_cache_bounded: bool,
    /// Whether cached entries may contain task content.
    pub caches_task_content: bool,
}

impl RenderCachePolicy {
    /// Return the W2 cache policy.
    pub const fn w2() -> Self {
        Self {
            max_cached_text_layouts: 32,
            text_layout_cache_bounded: true,
            caches_task_content: false,
        }
    }

    /// Return the cache invalidation decision for a trigger.
    pub const fn invalidate(self, trigger: RenderCacheInvalidation) -> RenderCacheInvalidationPlan {
        match trigger {
            RenderCacheInvalidation::DpiChanged => RenderCacheInvalidationPlan {
                invalidates_geometry: true,
                invalidates_text_layouts: true,
                invalidates_brushes: false,
                allows_unbounded_growth: false,
            },
            RenderCacheInvalidation::ThemeChanged => RenderCacheInvalidationPlan {
                invalidates_geometry: false,
                invalidates_text_layouts: false,
                invalidates_brushes: true,
                allows_unbounded_growth: false,
            },
            RenderCacheInvalidation::FontChanged => RenderCacheInvalidationPlan {
                invalidates_geometry: false,
                invalidates_text_layouts: true,
                invalidates_brushes: false,
                allows_unbounded_growth: false,
            },
            RenderCacheInvalidation::StateLayoutChanged => RenderCacheInvalidationPlan {
                invalidates_geometry: true,
                invalidates_text_layouts: true,
                invalidates_brushes: false,
                allows_unbounded_growth: false,
            },
        }
    }
}

/// Concrete cache invalidation effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCacheInvalidationPlan {
    /// Geometry cache must be invalidated.
    pub invalidates_geometry: bool,
    /// Text layout cache must be invalidated.
    pub invalidates_text_layouts: bool,
    /// Brush/material cache must be invalidated.
    pub invalidates_brushes: bool,
    /// Whether the invalidation path allows unbounded cache growth.
    pub allows_unbounded_growth: bool,
}

/// W2 render-resource policy for the future native shell adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderResourcePolicy {
    /// Whether the shell must use one shared D3D device.
    pub shared_d3d_device_required: bool,
    /// Maximum shared D3D devices allowed in the Island process.
    pub max_shared_d3d_devices: u32,
    /// Whether full render surfaces may be allocated per task.
    pub per_task_full_render_surfaces_allowed: bool,
    /// Whether Focus Card rows must be clipped or virtualized.
    pub focus_card_rows_virtualized: bool,
    /// Required state-transition count for the W2 leak check.
    pub max_state_transition_count: u32,
}

impl RenderResourcePolicy {
    /// Return the W2 render-resource policy.
    pub const fn w2() -> Self {
        Self {
            shared_d3d_device_required: true,
            max_shared_d3d_devices: 1,
            per_task_full_render_surfaces_allowed: false,
            focus_card_rows_virtualized: true,
            max_state_transition_count: 1_000,
        }
    }
}

/// Diagnostic resource snapshot captured outside task-content paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderResourceSnapshot {
    /// Number of shared D3D devices observed.
    pub shared_d3d_devices: u32,
    /// Number of render surfaces observed.
    pub render_surfaces: u32,
    /// Number of D3D resources observed.
    pub d3d_resources: u32,
    /// Relevant handle count observed.
    pub handles: u32,
}

impl RenderResourceSnapshot {
    /// Create a diagnostic resource snapshot.
    pub const fn new(
        shared_d3d_devices: u32,
        render_surfaces: u32,
        d3d_resources: u32,
        handles: u32,
    ) -> Self {
        Self {
            shared_d3d_devices,
            render_surfaces,
            d3d_resources,
            handles,
        }
    }
}

/// W2 render-resource stability report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderResourceReport {
    /// Number of state transitions exercised.
    pub state_transition_count: u32,
    /// Growth in shared device count.
    pub device_growth: u32,
    /// Growth in render surface count.
    pub surface_growth: u32,
    /// Growth in D3D resource count.
    pub d3d_resource_growth: u32,
    /// Growth in relevant handle count.
    pub handle_growth: u32,
    /// Whether the resource report passes W2 policy.
    pub passed: bool,
}

impl RenderResourceReport {
    /// Compare two resource snapshots under W2 policy.
    pub fn from_snapshots(
        policy: RenderResourcePolicy,
        baseline: RenderResourceSnapshot,
        final_snapshot: RenderResourceSnapshot,
        state_transition_count: u32,
    ) -> Self {
        let device_growth = final_snapshot
            .shared_d3d_devices
            .saturating_sub(baseline.shared_d3d_devices);
        let surface_growth = final_snapshot
            .render_surfaces
            .saturating_sub(baseline.render_surfaces);
        let d3d_resource_growth = final_snapshot
            .d3d_resources
            .saturating_sub(baseline.d3d_resources);
        let handle_growth = final_snapshot.handles.saturating_sub(baseline.handles);
        let passed = policy.shared_d3d_device_required
            && !policy.per_task_full_render_surfaces_allowed
            && final_snapshot.shared_d3d_devices <= policy.max_shared_d3d_devices
            && state_transition_count >= policy.max_state_transition_count
            && device_growth == 0
            && surface_growth == 0
            && d3d_resource_growth == 0
            && handle_growth == 0;

        Self {
            state_transition_count,
            device_growth,
            surface_growth,
            d3d_resource_growth,
            handle_growth,
            passed,
        }
    }
}

fn state_accessible_label(state: SignalState) -> &'static str {
    match state {
        SignalState::Idle => "Idle",
        SignalState::Running => "Running",
        SignalState::Waiting => "Waiting for user",
        SignalState::Failed => "Failed",
        SignalState::Completed => "Completed",
        SignalState::Observed => "Status unavailable",
    }
}

fn route_accessible_label(label: RouteActionLabel) -> &'static str {
    match label {
        RouteActionLabel::OpenOriginalTask => "Open original task",
        RouteActionLabel::OpenProviderThread => "Open provider thread",
        RouteActionLabel::FocusTerminalTab => "Focus terminal tab",
        RouteActionLabel::FocusAgentWindow => "Focus agent window",
        RouteActionLabel::FocusRelatedTerminal => "Focus related terminal",
        RouteActionLabel::OpenWorkspace => "Open workspace",
        RouteActionLabel::RevealProjectFolder => "Reveal project folder",
        RouteActionLabel::OpenAgent => "Open agent",
        RouteActionLabel::OpenOfficialUsage => "Open official usage",
        RouteActionLabel::ShowProcessDetails => "Show process details",
    }
}

impl ShellViewModel {
    /// Build a shell model from a deterministic mock scenario.
    pub fn from_scenario(scenario: &MockScenario) -> Self {
        Self::from_plan(&scenario.source.current_plan(), scenario.environment)
    }

    /// Build a shell model from a plan plus environment policy.
    pub fn from_plan(plan: &PresentationPlan, environment: ShellEnvironment) -> Self {
        let signal = SignalViewModel::from_plan(plan);
        let compact_visible = signal.state != SignalState::Idle && !environment.immersive_active;
        let motion_policy = if environment.immersive_active {
            MotionPolicy::Stopped
        } else if environment.reduced_motion {
            MotionPolicy::Reduced
        } else {
            MotionPolicy::Normal
        };
        Self {
            signal,
            peek: PeekViewModel::from_plan(plan),
            focus_card: FocusCardViewModel::from_primary(plan),
            compact_visible,
            palette_visible: false,
            motion_policy,
            high_contrast: environment.high_contrast,
        }
    }
}

/// Return the deterministic Spike A mock scenario catalog in S0-S8 order.
pub fn mock_scenario_catalog() -> Result<Vec<MockScenario>, DomainError> {
    Ok(vec![
        MockScenario {
            id: MockScenarioId::S0IdleParked,
            name: "S0 idle / parked",
            source: MockPresentationPlanSource::new(plan(None, Vec::new(), 0)),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S1OneRunningTask,
            name: "S1 one running task",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_task(
                    "s1-running",
                    "Codex",
                    Lifecycle::Running,
                    RouteStrength::Useful,
                    10,
                )?),
                Vec::new(),
                10,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S2WaitingWithBackgroundWork,
            name: "S2 waiting task with background work",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_task(
                    "s2-waiting",
                    "Claude",
                    Lifecycle::WaitingUser,
                    RouteStrength::Useful,
                    20,
                )?),
                vec![
                    mock_task(
                        "s2-background-a",
                        "Codex",
                        Lifecycle::Running,
                        RouteStrength::Useful,
                        19,
                    )?,
                    mock_task(
                        "s2-background-b",
                        "Codex",
                        Lifecycle::Running,
                        RouteStrength::Weak,
                        18,
                    )?,
                ],
                20,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S3FailedTask,
            name: "S3 failed task",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_task(
                    "s3-failed",
                    "Codex",
                    Lifecycle::Failed,
                    RouteStrength::Useful,
                    30,
                )?),
                Vec::new(),
                30,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S4AggregateActiveFuelLow,
            name: "S4 aggregate active work / Fuel low",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_fuel_low_task(
                    "s4-aggregate",
                    "3 agents working",
                    Lifecycle::Running,
                    40,
                )?),
                vec![
                    mock_task(
                        "s4-worker-a",
                        "Codex",
                        Lifecycle::Running,
                        RouteStrength::Useful,
                        39,
                    )?,
                    mock_task(
                        "s4-worker-b",
                        "Claude",
                        Lifecycle::Running,
                        RouteStrength::Strong,
                        38,
                    )?,
                ],
                40,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S5DegradedObserved,
            name: "S5 degraded / observed state",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_observed_task("s5-observed", "Claude", 50)?),
                Vec::new(),
                50,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S6CompletionSettleOut,
            name: "S6 completion settle-out",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_task(
                    "s6-completed",
                    "Codex",
                    Lifecycle::Completed,
                    RouteStrength::Exact,
                    60,
                )?),
                Vec::new(),
                60,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S7RapidStateChanges,
            name: "S7 rapid state changes",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_task(
                    "s7-rapid",
                    "Codex",
                    Lifecycle::Running,
                    RouteStrength::Useful,
                    70,
                )?),
                Vec::new(),
                70,
            )),
            transitions: rapid_state_transitions()?,
            environment: ShellEnvironment::default(),
        },
        MockScenario {
            id: MockScenarioId::S8ImmersiveSimulation,
            name: "S8 immersive mode simulation",
            source: MockPresentationPlanSource::new(plan(
                Some(mock_task(
                    "s8-running",
                    "Codex",
                    Lifecycle::Running,
                    RouteStrength::Useful,
                    80,
                )?),
                Vec::new(),
                80,
            )),
            transitions: Vec::new(),
            environment: ShellEnvironment {
                immersive_active: true,
                reduced_motion: false,
                high_contrast: false,
            },
        },
    ])
}

fn plan(
    primary: Option<TaskSnapshot>,
    peek: Vec<TaskSnapshot>,
    generated_at: u64,
) -> PresentationPlan {
    PresentationPlan {
        primary,
        peek,
        generated_at: TimestampMs(generated_at),
    }
}

fn mock_task(
    id: &str,
    provider: &str,
    lifecycle: Lifecycle,
    route_strength: RouteStrength,
    updated_at: u64,
) -> Result<TaskSnapshot, DomainError> {
    let mut snapshot = TaskSnapshot::generic(
        ProviderId(BoundedText::new(provider)?),
        TaskId(BoundedText::new(id)?),
        TimestampMs(updated_at),
    );
    snapshot.lifecycle = lifecycle;
    snapshot.route_strength = route_strength;
    snapshot.summary = summary_for_lifecycle(lifecycle);
    snapshot.attention = attention_for_lifecycle(lifecycle);
    Ok(snapshot)
}

fn mock_fuel_low_task(
    id: &str,
    provider: &str,
    lifecycle: Lifecycle,
    updated_at: u64,
) -> Result<TaskSnapshot, DomainError> {
    let mut snapshot = mock_task(id, provider, lifecycle, RouteStrength::Useful, updated_at)?;
    snapshot.fuel_risk = true;
    Ok(snapshot)
}

fn mock_observed_task(
    id: &str,
    provider: &str,
    updated_at: u64,
) -> Result<TaskSnapshot, DomainError> {
    let mut snapshot = mock_task(
        id,
        provider,
        Lifecycle::Observed,
        RouteStrength::Weak,
        updated_at,
    )?;
    snapshot.health = TaskHealth::Degraded;
    snapshot.summary = SafeSummary::ObservedProcess;
    Ok(snapshot)
}

fn rapid_state_transitions() -> Result<Vec<PresentationPlan>, DomainError> {
    Ok(vec![
        plan(
            Some(mock_task(
                "s7-rapid",
                "Codex",
                Lifecycle::Running,
                RouteStrength::Useful,
                70,
            )?),
            Vec::new(),
            70,
        ),
        plan(
            Some(mock_task(
                "s7-rapid",
                "Codex",
                Lifecycle::WaitingUser,
                RouteStrength::Useful,
                71,
            )?),
            Vec::new(),
            71,
        ),
        plan(
            Some(mock_task(
                "s7-rapid",
                "Codex",
                Lifecycle::Running,
                RouteStrength::Useful,
                72,
            )?),
            Vec::new(),
            72,
        ),
        plan(
            Some(mock_task(
                "s7-rapid",
                "Codex",
                Lifecycle::Failed,
                RouteStrength::Useful,
                73,
            )?),
            Vec::new(),
            73,
        ),
        plan(
            Some(mock_task(
                "s7-rapid",
                "Codex",
                Lifecycle::Completed,
                RouteStrength::Useful,
                74,
            )?),
            Vec::new(),
            74,
        ),
        plan(None, Vec::new(), 75),
    ])
}

fn summary_for_lifecycle(lifecycle: Lifecycle) -> SafeSummary {
    match lifecycle {
        Lifecycle::WaitingUser => SafeSummary::WaitingForUser,
        Lifecycle::Limited => SafeSummary::LimitReached,
        Lifecycle::Failed => SafeSummary::Failed,
        Lifecycle::Observed => SafeSummary::ObservedProcess,
        Lifecycle::Unknown | Lifecycle::Running | Lifecycle::Completed => SafeSummary::Generic,
    }
}

fn attention_for_lifecycle(lifecycle: Lifecycle) -> Attention {
    match lifecycle {
        Lifecycle::Running => Attention::Active,
        Lifecycle::WaitingUser => Attention::Waiting,
        Lifecycle::Limited => Attention::Limited,
        Lifecycle::Failed => Attention::Failed,
        Lifecycle::Unknown | Lifecycle::Observed | Lifecycle::Completed => Attention::None,
    }
}

#[cfg(test)]
mod tests {
    use pulse_arbitration::{arbitrate, PresentationPlan};
    use pulse_domain::{Lifecycle, RouteStrength, TaskSnapshot, TimestampMs};
    use pulse_routing::RouteActionLabel;
    use pulse_testkit::{provider, task};

    use super::*;

    fn snapshot(
        id: &str,
        lifecycle: Lifecycle,
    ) -> Result<TaskSnapshot, Box<dyn std::error::Error>> {
        let mut snapshot = TaskSnapshot::generic(provider("Codex")?, task(id)?, TimestampMs(1));
        snapshot.lifecycle = lifecycle;
        snapshot.route_strength = RouteStrength::Useful;
        Ok(snapshot)
    }

    fn missing(label: &str) -> Box<dyn std::error::Error> {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            label.to_owned(),
        ))
    }

    #[test]
    fn signal_view_model_renders_primary_plan_without_resorting(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let running = snapshot("running", Lifecycle::Running)?;
        let failed = snapshot("failed", Lifecycle::Failed)?;
        let plan = arbitrate(&[running, failed], None, TimestampMs(10));

        let view = SignalViewModel::from_plan(&plan);

        assert_eq!(view.primary_task_id.as_deref(), Some("failed"));
        assert_eq!(view.state, SignalState::Failed);
        assert_eq!(view.overflow_count, 1);
        assert_eq!(
            view.primary_route_label,
            Some(RouteActionLabel::OpenWorkspace)
        );
        Ok(())
    }

    #[test]
    fn mock_plan_source_returns_current_plan_deterministically(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = PresentationPlan {
            primary: Some(snapshot("current", Lifecycle::WaitingUser)?),
            peek: vec![snapshot("peek", Lifecycle::Running)?],
            generated_at: TimestampMs(20),
        };
        let source = MockPresentationPlanSource::new(plan.clone());

        assert_eq!(source.current_plan(), plan);
        assert_eq!(
            SignalViewModel::from_plan(&source.current_plan()).state,
            SignalState::Waiting
        );
        Ok(())
    }

    #[test]
    fn mock_plan_source_subscribe_delivers_current_plan_without_transport(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = PresentationPlan {
            primary: Some(snapshot("subscribed", Lifecycle::Running)?),
            peek: Vec::new(),
            generated_at: TimestampMs(21),
        };
        let source = MockPresentationPlanSource::new(plan.clone());
        let mut delivered = Vec::new();

        source.subscribe(&mut |changed_plan| delivered.push(changed_plan));

        assert_eq!(delivered, vec![plan]);
        Ok(())
    }

    #[test]
    fn peek_view_model_renders_plan_rows_without_resorting_or_overflow(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = PresentationPlan {
            primary: Some(snapshot("primary", Lifecycle::WaitingUser)?),
            peek: vec![
                snapshot("peek-a", Lifecycle::Running)?,
                snapshot("peek-b", Lifecycle::Failed)?,
                snapshot("peek-c", Lifecycle::Observed)?,
                snapshot("peek-d", Lifecycle::Completed)?,
            ],
            generated_at: TimestampMs(30),
        };

        let peek = PeekViewModel::from_plan(&plan);

        assert_eq!(peek.rows.len(), 3);
        assert_eq!(peek.rows[0].task_id, "peek-a");
        assert_eq!(peek.rows[1].task_id, "peek-b");
        assert_eq!(peek.rows[2].task_id, "peek-c");
        assert_eq!(peek.hidden_count, 1);
        assert_eq!(peek.rows[1].state, SignalState::Failed);
        assert_eq!(
            peek.rows[0].route_label,
            Some(RouteActionLabel::OpenWorkspace)
        );
        Ok(())
    }

    #[test]
    fn focus_card_keeps_route_label_and_exposes_no_p0_controls(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut primary = snapshot("focus", Lifecycle::WaitingUser)?;
        primary.route_strength = RouteStrength::Exact;
        let plan = PresentationPlan {
            primary: Some(primary),
            peek: Vec::new(),
            generated_at: TimestampMs(40),
        };

        let Some(focus) = FocusCardViewModel::from_primary(&plan) else {
            return Err(missing("primary focus card"));
        };

        assert_eq!(focus.task_id, "focus");
        assert_eq!(focus.state, SignalState::Waiting);
        assert_eq!(focus.route_label, Some(RouteActionLabel::OpenOriginalTask));
        assert!(focus.control_actions.is_empty());
        Ok(())
    }

    #[test]
    fn mock_scenario_catalog_covers_spike_a_s0_to_s8_in_order(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ids = mock_scenario_catalog()?
            .iter()
            .map(|scenario| scenario.id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                MockScenarioId::S0IdleParked,
                MockScenarioId::S1OneRunningTask,
                MockScenarioId::S2WaitingWithBackgroundWork,
                MockScenarioId::S3FailedTask,
                MockScenarioId::S4AggregateActiveFuelLow,
                MockScenarioId::S5DegradedObserved,
                MockScenarioId::S6CompletionSettleOut,
                MockScenarioId::S7RapidStateChanges,
                MockScenarioId::S8ImmersiveSimulation,
            ]
        );
        Ok(())
    }

    #[test]
    fn spike_a_mock_scenarios_drive_expected_signal_peek_and_focus_models(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scenarios = mock_scenario_catalog()?;

        let Some(idle) = scenarios
            .iter()
            .find(|scenario| scenario.id == MockScenarioId::S0IdleParked)
        else {
            return Err(missing("idle scenario"));
        };
        let idle_shell = ShellViewModel::from_scenario(idle);
        assert_eq!(idle_shell.signal.state, SignalState::Idle);
        assert!(!idle_shell.compact_visible);
        assert!(idle_shell.focus_card.is_none());

        let Some(waiting) = scenarios
            .iter()
            .find(|scenario| scenario.id == MockScenarioId::S2WaitingWithBackgroundWork)
        else {
            return Err(missing("waiting scenario"));
        };
        let waiting_shell = ShellViewModel::from_scenario(waiting);
        assert_eq!(waiting_shell.signal.state, SignalState::Waiting);
        assert_eq!(waiting_shell.signal.overflow_count, 2);
        assert_eq!(waiting_shell.peek.rows.len(), 2);
        assert_eq!(
            waiting_shell
                .focus_card
                .as_ref()
                .map(|focus| focus.route_label),
            Some(Some(RouteActionLabel::OpenWorkspace))
        );

        let Some(immersive) = scenarios
            .iter()
            .find(|scenario| scenario.id == MockScenarioId::S8ImmersiveSimulation)
        else {
            return Err(missing("immersive scenario"));
        };
        let immersive_shell = ShellViewModel::from_scenario(immersive);
        assert_eq!(immersive_shell.signal.state, SignalState::Running);
        assert!(!immersive_shell.compact_visible);
        assert_eq!(immersive_shell.motion_policy, MotionPolicy::Stopped);
        Ok(())
    }

    #[test]
    fn mock_scenarios_cover_all_w2_route_label_strengths() -> Result<(), Box<dyn std::error::Error>>
    {
        let labels = mock_scenario_catalog()?
            .iter()
            .flat_map(|scenario| {
                let shell = ShellViewModel::from_scenario(scenario);
                let mut labels = Vec::new();
                if let Some(label) = shell.signal.primary_route_label {
                    labels.push(label);
                }
                labels.extend(shell.peek.rows.iter().filter_map(|row| row.route_label));
                if let Some(label) = shell
                    .focus_card
                    .as_ref()
                    .and_then(|focus| focus.route_label)
                {
                    labels.push(label);
                }
                labels
            })
            .collect::<Vec<_>>();

        assert!(labels.contains(&RouteActionLabel::OpenOriginalTask));
        assert!(labels.contains(&RouteActionLabel::FocusAgentWindow));
        assert!(labels.contains(&RouteActionLabel::OpenWorkspace));
        assert!(labels.contains(&RouteActionLabel::ShowProcessDetails));
        Ok(())
    }

    #[test]
    fn passive_plan_update_never_steals_focus_or_opens_surfaces() {
        let state = ShellInteractionState::default();

        let next = state.apply(ShellUserEvent::PassivePlanUpdate);

        assert_eq!(next.focus_owner, FocusOwner::ExternalApp);
        assert_eq!(next.open_surface, OpenSurface::None);
    }

    #[test]
    fn compact_click_opens_peek_without_focus_theft() {
        let state = ShellInteractionState::default();

        let next = state.apply(ShellUserEvent::CompactClicked);

        assert_eq!(next.open_surface, OpenSurface::Peek);
        assert_eq!(next.focus_owner, FocusOwner::ExternalApp);
    }

    #[test]
    fn global_shortcut_opens_palette_with_keyboard_focus() {
        let state = ShellInteractionState::default();

        let next = state.apply(ShellUserEvent::PaletteShortcut);

        assert_eq!(next.open_surface, OpenSurface::CommandPalette);
        assert_eq!(next.focus_owner, FocusOwner::CommandPalette);
    }

    #[test]
    fn palette_invocation_policy_controls_global_shortcut_and_immersive_access() {
        let disabled = PaletteInvocationPolicy {
            global_shortcut_enabled: false,
            allow_during_immersive: true,
        };
        let immersive_blocked = PaletteInvocationPolicy {
            global_shortcut_enabled: true,
            allow_during_immersive: false,
        };
        let immersive_allowed = PaletteInvocationPolicy {
            global_shortcut_enabled: true,
            allow_during_immersive: true,
        };

        assert_eq!(
            ShellInteractionState::default()
                .apply_palette_invocation(disabled, ShellEnvironment::default()),
            ShellInteractionState::default()
        );
        assert_eq!(
            ShellInteractionState::default().apply_palette_invocation(
                immersive_blocked,
                ShellEnvironment {
                    immersive_active: true,
                    reduced_motion: false,
                    high_contrast: false,
                }
            ),
            ShellInteractionState::default()
        );
        assert_eq!(
            ShellInteractionState::default()
                .apply_palette_invocation(
                    immersive_allowed,
                    ShellEnvironment {
                        immersive_active: true,
                        reduced_motion: false,
                        high_contrast: false,
                    }
                )
                .focus_owner,
            FocusOwner::CommandPalette
        );
    }

    #[test]
    fn peek_row_click_opens_focus_card_with_explicit_focus() {
        let state = ShellInteractionState::default().apply(ShellUserEvent::CompactClicked);

        let next = state.apply(ShellUserEvent::PeekRowClicked {
            task_id: "task-a".to_owned(),
        });

        assert_eq!(next.open_surface, OpenSurface::FocusCard);
        assert_eq!(next.focus_owner, FocusOwner::FocusCard);
        assert_eq!(next.focused_task_id.as_deref(), Some("task-a"));
    }

    #[test]
    fn escape_closes_focused_surface_and_returns_external_focus() {
        let state = ShellInteractionState::default().apply(ShellUserEvent::PaletteShortcut);

        let next = state.apply(ShellUserEvent::Escape);

        assert_eq!(next.open_surface, OpenSurface::None);
        assert_eq!(next.focus_owner, FocusOwner::ExternalApp);
        assert_eq!(next.focused_task_id, None);
    }

    #[test]
    fn shell_lifecycle_keeps_window_generation_stable_across_repeated_surface_cycles() {
        let mut lifecycle = ShellSurfaceLifecycle::default();

        for index in 0..1_000 {
            lifecycle = lifecycle
                .apply(ShellUserEvent::CompactClicked)
                .apply(ShellUserEvent::Escape)
                .apply(ShellUserEvent::PaletteShortcut)
                .apply(ShellUserEvent::Escape)
                .apply(ShellUserEvent::CompactClicked)
                .apply(ShellUserEvent::PeekRowClicked {
                    task_id: format!("task-{index}"),
                })
                .apply(ShellUserEvent::Escape);
        }

        assert_eq!(lifecycle.window_generation, 1);
        assert_eq!(lifecycle.active_transient_surfaces, 0);
        assert_eq!(lifecycle.max_active_transient_surfaces, 1);
        assert_eq!(lifecycle.open_close_cycles, 3_000);
    }

    #[test]
    fn surface_handle_report_requires_stable_handles_after_peek_focus_cycles() {
        let mut lifecycle = ShellSurfaceLifecycle::default();

        for index in 0..1_000 {
            lifecycle = lifecycle
                .apply(ShellUserEvent::CompactClicked)
                .apply(ShellUserEvent::Escape)
                .apply(ShellUserEvent::CompactClicked)
                .apply(ShellUserEvent::PeekRowClicked {
                    task_id: format!("task-{index}"),
                })
                .apply(ShellUserEvent::Escape);
        }

        let stable = SurfaceHandleStabilityReport::from_lifecycle(
            lifecycle.clone(),
            SurfaceHandleSnapshot::new(12, 8),
            SurfaceHandleSnapshot::new(12, 8),
        );
        let leaking = SurfaceHandleStabilityReport::from_lifecycle(
            lifecycle,
            SurfaceHandleSnapshot::new(12, 8),
            SurfaceHandleSnapshot::new(13, 11),
        );

        assert_eq!(stable.required_open_close_cycles, 2_000);
        assert_eq!(stable.actual_open_close_cycles, 2_000);
        assert_eq!(stable.user_handle_growth, 0);
        assert_eq!(stable.gdi_handle_growth, 0);
        assert!(stable.passed);
        assert!(!leaking.passed);
        assert!(leaking.user_handle_growth > 0);
        assert!(leaking.gdi_handle_growth > 0);
    }

    #[test]
    fn accessible_signal_label_exposes_state_without_relying_on_color() {
        let view = SignalViewModel {
            primary_task_id: Some("task-a".to_owned()),
            state: SignalState::Waiting,
            overflow_count: 2,
            primary_route_label: Some(RouteActionLabel::OpenWorkspace),
        };

        let accessible = AccessibleSignalViewModel::from_signal(&view);

        assert_eq!(accessible.state_label, "Waiting for user");
        assert_eq!(
            accessible.name,
            "Waiting for user; task task-a; 2 more active; Open workspace"
        );
        assert!(!accessible.uses_color_as_sole_indicator);
    }

    #[test]
    fn reduced_motion_removes_pulse_and_scale_but_keeps_state_change() {
        let normal = AnimationPolicy::for_state(SignalState::Waiting, MotionPolicy::Normal);
        let reduced = AnimationPolicy::for_state(SignalState::Waiting, MotionPolicy::Reduced);

        assert!(normal.state_change_visible);
        assert!(normal.pulse_allowed);
        assert!(normal.scale_allowed);
        assert!(reduced.state_change_visible);
        assert!(!reduced.pulse_allowed);
        assert!(!reduced.scale_allowed);
    }

    #[test]
    fn compositor_animation_plans_are_bounded_and_compositor_owned() {
        let waiting = CompositorAnimationPlan::for_class(
            CompositorAnimationClass::AttentionPulse,
            MotionPolicy::Normal,
        );
        let failure = CompositorAnimationPlan::for_class(
            CompositorAnimationClass::StateTransition,
            MotionPolicy::Normal,
        );
        let expansion = CompositorAnimationPlan::for_class(
            CompositorAnimationClass::Expansion,
            MotionPolicy::Normal,
        );
        let reduced_waiting = CompositorAnimationPlan::for_class(
            CompositorAnimationClass::AttentionPulse,
            MotionPolicy::Reduced,
        );

        assert!(waiting.compositor_owned);
        assert!(!waiting.app_side_frame_loop_allowed);
        assert_eq!(waiting.max_repetitions, Some(3));
        assert!(waiting.settles_to_static);
        assert!(waiting.pulse_allowed);
        assert!(!failure.pulse_allowed);
        assert!(failure.interruptible);
        assert!(failure.settles_to_static);
        assert!(expansion.interruptible);
        assert!(expansion.scale_allowed);
        assert!(!reduced_waiting.pulse_allowed);
        assert!(!reduced_waiting.scale_allowed);
        assert!(reduced_waiting.settles_to_static);
    }

    #[test]
    fn high_contrast_uses_explicit_contrast_tokens() {
        let normal = VisualAccessibilityPolicy::from_environment(ShellEnvironment {
            immersive_active: false,
            reduced_motion: false,
            high_contrast: false,
        });
        let high_contrast = VisualAccessibilityPolicy::from_environment(ShellEnvironment {
            immersive_active: false,
            reduced_motion: false,
            high_contrast: true,
        });

        assert_eq!(normal.contrast_mode, ContrastMode::Standard);
        assert_eq!(high_contrast.contrast_mode, ContrastMode::HighContrast);
        assert!(high_contrast.uses_system_contrast_tokens);
    }

    #[test]
    fn keyboard_navigation_is_deterministic_for_peek_and_palette() {
        let peek = KeyboardNavigationState::new(OpenSurface::Peek, 3)
            .apply(KeyboardCommand::ArrowDown)
            .apply(KeyboardCommand::ArrowDown)
            .apply(KeyboardCommand::ArrowDown);
        let palette = KeyboardNavigationState::new(OpenSurface::CommandPalette, 9)
            .apply(KeyboardCommand::ArrowUp)
            .apply(KeyboardCommand::Enter);

        assert_eq!(peek.selected_index, Some(2));
        assert_eq!(peek.activated_index, None);
        assert_eq!(palette.selected_index, Some(0));
        assert_eq!(palette.activated_index, Some(0));
    }

    #[test]
    fn command_palette_exports_p0_navigation_commands_without_provider_controls() {
        let palette = CommandPaletteViewModel::p0();
        let command_ids = palette
            .commands
            .iter()
            .map(|command| command.id)
            .collect::<Vec<_>>();

        assert_eq!(
            command_ids,
            vec![
                CommandPaletteCommandId::OpenActiveTask,
                CommandPaletteCommandId::OpenWorkspace,
                CommandPaletteCommandId::ShowLowestFuelWindow,
                CommandPaletteCommandId::OpenProviderUsage,
                CommandPaletteCommandId::ShowActiveAgents,
                CommandPaletteCommandId::PinTask,
                CommandPaletteCommandId::FollowTask,
                CommandPaletteCommandId::MuteTaskOrWorkspace,
                CommandPaletteCommandId::OpenPulseSettings,
            ]
        );
        assert!(palette
            .commands
            .iter()
            .all(|command| !command.provider_control));
        assert!(palette.commands.iter().all(|command| !command.high_risk));
        assert_eq!(palette.commands.len(), 9);
    }

    #[test]
    fn compact_signal_layout_preserves_core_state_under_width_pressure() {
        let policy = CompactSignalLayoutPolicy {
            available_width_px: 128,
            gap_width_px: 8,
            state_glyph_width_px: 16,
            subject_width_px: 96,
            reason_width_px: 88,
            active_count_width_px: 24,
            secondary_fuel_width_px: 40,
        };
        let narrow = CompactSignalLayoutPolicy {
            available_width_px: 72,
            ..policy
        };

        let decision = policy.evaluate();
        let narrow_decision = narrow.evaluate();

        assert!(decision.show_state_glyph);
        assert!(decision.show_subject);
        assert!(!decision.subject_truncated);
        assert!(!decision.show_secondary_fuel);
        assert!(!decision.show_active_count);
        assert!(!decision.show_reason);

        assert!(narrow_decision.show_state_glyph);
        assert!(narrow_decision.show_subject);
        assert!(narrow_decision.subject_truncated);
        assert!(!narrow_decision.show_reason);
    }

    #[test]
    fn signal_truth_priority_forbids_timer_rotation_and_keeps_fuel_secondary() {
        let signal = SignalViewModel {
            primary_task_id: Some("waiting-primary".to_owned()),
            state: SignalState::Waiting,
            overflow_count: 3,
            primary_route_label: Some(RouteActionLabel::OpenWorkspace),
        };

        let decision =
            SignalTruthPriorityDecision::from_signal(&signal, FuelThreadCandidate::TrustworthyLow);

        assert_eq!(decision.primary_task_id.as_deref(), Some("waiting-primary"));
        assert_eq!(decision.primary_state, SignalState::Waiting);
        assert_eq!(
            decision.primary_story_source,
            PrimaryStorySource::PresentationPlanPrimary
        );
        assert_eq!(decision.fuel_thread_role, FuelThreadRole::Secondary);
        assert!(decision.fuel_thread_visible);
        assert!(!decision.timer_rotation_allowed);
        assert!(!decision.fuel_can_override_primary_state);
    }

    #[test]
    fn measurement_policy_exports_gate_a_targets_without_task_content() {
        let policy = MeasurementPolicy::gate_a();
        let metric_names = policy
            .metrics
            .iter()
            .map(|metric| metric.name)
            .collect::<Vec<_>>();

        assert!(policy.diagnostic_metadata_only);
        assert!(!policy.includes_task_content);
        assert!(metric_names.contains(&MeasurementMetricName::CompactIdleMemoryP95Mb));
        assert!(metric_names.contains(&MeasurementMetricName::FocusCardMemoryP95Mb));
        assert!(metric_names.contains(&MeasurementMetricName::IdleAverageCpuPercent));
        assert!(metric_names.contains(&MeasurementMetricName::StateUpdateLatencyP95Ms));
        assert!(metric_names.contains(&MeasurementMetricName::PaletteShortcutLatencyP95Ms));
    }

    #[test]
    fn measurement_report_evaluates_gate_a_samples_and_requires_every_metric() {
        let policy = MeasurementPolicy::gate_a();
        let report = MeasurementReport::from_samples(
            policy.clone(),
            vec![
                MeasurementSample::new(MeasurementMetricName::CompactIdleMemoryP95Mb, 42.0),
                MeasurementSample::new(MeasurementMetricName::CompactIdleMemoryP95Mb, 45.0),
                MeasurementSample::new(MeasurementMetricName::FocusCardMemoryP95Mb, 80.0),
                MeasurementSample::new(MeasurementMetricName::FocusCardMemoryP95Mb, 84.0),
                MeasurementSample::new(MeasurementMetricName::ProcessTreeCeilingMb, 99.0),
                MeasurementSample::new(MeasurementMetricName::IdleAverageCpuPercent, 0.08),
                MeasurementSample::new(MeasurementMetricName::RunningAverageCpuPercent, 0.30),
                MeasurementSample::new(MeasurementMetricName::StateUpdateLatencyP95Ms, 100.0),
                MeasurementSample::new(MeasurementMetricName::PaletteShortcutLatencyP95Ms, 75.0),
                MeasurementSample::new(MeasurementMetricName::SteadyStateMemoryGrowthMb, 1.5),
                MeasurementSample::new(MeasurementMetricName::StaticStateFrameLoop, 0.0),
            ],
        );
        let failing_report = MeasurementReport::from_samples(
            policy,
            vec![MeasurementSample::new(
                MeasurementMetricName::CompactIdleMemoryP95Mb,
                46.0,
            )],
        );

        assert!(report.passed);
        assert_eq!(report.results.len(), 9);
        assert!(report.results.iter().all(|result| result.passed));
        assert!(failing_report.missing_metrics.len() > 1);
        assert!(!failing_report.passed);
    }

    #[test]
    fn performance_overlay_is_diagnostics_only_and_content_free() {
        let normal = PerformanceOverlayViewModel::from_report(
            OverlayMode::Normal,
            MeasurementReport::from_samples(MeasurementPolicy::gate_a(), Vec::new()),
        );
        let diagnostics = PerformanceOverlayViewModel::from_report(
            OverlayMode::Diagnostics,
            MeasurementReport::from_samples(
                MeasurementPolicy::gate_a(),
                vec![
                    MeasurementSample::new(MeasurementMetricName::CompactIdleMemoryP95Mb, 42.0),
                    MeasurementSample::new(MeasurementMetricName::FocusCardMemoryP95Mb, 84.0),
                    MeasurementSample::new(MeasurementMetricName::ProcessTreeCeilingMb, 99.0),
                    MeasurementSample::new(MeasurementMetricName::IdleAverageCpuPercent, 0.08),
                    MeasurementSample::new(MeasurementMetricName::RunningAverageCpuPercent, 0.30),
                    MeasurementSample::new(MeasurementMetricName::StateUpdateLatencyP95Ms, 100.0),
                    MeasurementSample::new(
                        MeasurementMetricName::PaletteShortcutLatencyP95Ms,
                        75.0,
                    ),
                    MeasurementSample::new(MeasurementMetricName::SteadyStateMemoryGrowthMb, 1.5),
                    MeasurementSample::new(MeasurementMetricName::StaticStateFrameLoop, 0.0),
                ],
            ),
        );

        assert!(!normal.visible);
        assert!(normal.rows.is_empty());
        assert!(!normal.includes_task_content);
        assert!(diagnostics.visible);
        assert!(diagnostics.diagnostic_metadata_only);
        assert!(!diagnostics.includes_task_content);
        assert_eq!(diagnostics.rows.len(), 9);
        assert!(diagnostics.rows.iter().all(|row| row.label != "task"));
        assert!(diagnostics.rows.iter().all(|row| row.passed));
    }

    #[test]
    fn static_render_policy_forbids_app_side_frame_loop_when_quiet() {
        let idle = StaticRenderPolicy::for_shell(
            &ShellViewModel::from_plan(
                &PresentationPlan {
                    primary: None,
                    peek: Vec::new(),
                    generated_at: TimestampMs(1),
                },
                ShellEnvironment::default(),
            ),
            MotionPolicy::Normal,
        );
        let transitioning = StaticRenderPolicy::for_shell(
            &ShellViewModel {
                signal: SignalViewModel {
                    primary_task_id: Some("task-a".to_owned()),
                    state: SignalState::Running,
                    overflow_count: 0,
                    primary_route_label: None,
                },
                peek: PeekViewModel {
                    rows: Vec::new(),
                    hidden_count: 0,
                },
                focus_card: None,
                compact_visible: true,
                palette_visible: false,
                motion_policy: MotionPolicy::Normal,
                high_contrast: false,
            },
            MotionPolicy::Normal,
        );

        assert!(!idle.app_side_frame_loop_allowed);
        assert_eq!(idle.redraw_reason, RedrawReason::None);
        assert!(!transitioning.app_side_frame_loop_allowed);
        assert_eq!(transitioning.redraw_reason, RedrawReason::CompositorOnly);
    }

    #[test]
    fn render_cache_policy_invalidates_required_bounded_caches() {
        let policy = RenderCachePolicy::w2();
        let dpi = policy.invalidate(RenderCacheInvalidation::DpiChanged);
        let theme = policy.invalidate(RenderCacheInvalidation::ThemeChanged);
        let font = policy.invalidate(RenderCacheInvalidation::FontChanged);
        let state_layout = policy.invalidate(RenderCacheInvalidation::StateLayoutChanged);

        assert!(policy.text_layout_cache_bounded);
        assert_eq!(policy.max_cached_text_layouts, 32);
        assert!(!policy.caches_task_content);
        assert!(dpi.invalidates_geometry);
        assert!(dpi.invalidates_text_layouts);
        assert!(theme.invalidates_brushes);
        assert!(font.invalidates_text_layouts);
        assert!(state_layout.invalidates_geometry);
        assert!(state_layout.invalidates_text_layouts);
        assert!(!state_layout.allows_unbounded_growth);
    }

    #[test]
    fn render_resource_policy_forbids_per_task_surfaces_and_detects_growth() {
        let policy = RenderResourcePolicy::w2();
        let stable = RenderResourceReport::from_snapshots(
            policy,
            RenderResourceSnapshot::new(1, 3, 4, 24),
            RenderResourceSnapshot::new(1, 3, 4, 24),
            1_000,
        );
        let leaking = RenderResourceReport::from_snapshots(
            policy,
            RenderResourceSnapshot::new(1, 3, 4, 24),
            RenderResourceSnapshot::new(1, 4, 7, 32),
            1_000,
        );

        assert!(policy.shared_d3d_device_required);
        assert!(!policy.per_task_full_render_surfaces_allowed);
        assert!(policy.focus_card_rows_virtualized);
        assert_eq!(policy.max_shared_d3d_devices, 1);
        assert_eq!(policy.max_state_transition_count, 1_000);
        assert!(stable.passed);
        assert_eq!(stable.device_growth, 0);
        assert_eq!(stable.surface_growth, 0);
        assert_eq!(stable.d3d_resource_growth, 0);
        assert_eq!(stable.handle_growth, 0);
        assert!(!leaking.passed);
        assert!(leaking.surface_growth > 0);
        assert!(leaking.d3d_resource_growth > 0);
        assert!(leaking.handle_growth > 0);
    }
}
