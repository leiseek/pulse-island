//! Spike A scenario harness for the mocked native Island shell.
#![deny(missing_docs)]

use pulse_island_ui::{
    mock_scenario_catalog, AccessibleSignalViewModel, AnimationPolicy, CommandPaletteViewModel,
    CompactSignalLayoutDecision, CompactSignalLayoutPolicy, CompositorAnimationClass,
    CompositorAnimationPlan, FuelThreadCandidate, KeyboardCommand, KeyboardNavigationState,
    MeasurementPolicy, MeasurementReport, MeasurementSample, MockScenario, MockScenarioId,
    MotionPolicy, OpenSurface, OverlayMode, PaletteInvocationPolicy, PerformanceOverlayViewModel,
    RenderCacheInvalidation, RenderCachePolicy, RenderResourcePolicy, RenderResourceReport,
    RenderResourceSnapshot, ShellEnvironment, ShellInteractionState, ShellSurfaceLifecycle,
    ShellUserEvent, ShellViewModel, SignalState, SignalTruthPriorityDecision, SignalViewModel,
    StaticRenderPolicy, SurfaceHandleSnapshot, SurfaceHandleStabilityReport,
    VisualAccessibilityPolicy,
};
use pulse_win32::{
    CompactWindowStylePolicy, DisplayTopology, DpiScale, GlobalHotkeyPolicy, HitTarget,
    HitTestLayout, ImmersiveState, ImmersiveWindowPolicy, LogicalPoint, LogicalSize, MonitorId,
    MonitorWorkArea, NativeWindowAdapterAction, NativeWindowAdapterInput, NativeWindowAdapterPlan,
    NativeWindowAdapterState, PointPx, RectPx, RememberedLogicalPlacement, SizePx,
    TextScalePercent, Win32HitTestCode, WindowPlacement,
};

/// Error returned by the Spike A scenario harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpikeHarnessError {
    /// Mock catalog construction failed.
    Catalog(pulse_domain::DomainError),
    /// Requested scenario does not exist in the catalog.
    ScenarioNotFound(MockScenarioId),
    /// Command-line argument was not recognized by the spike runner.
    InvalidArgument(String),
}

impl core::fmt::Display for SpikeHarnessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Catalog(error) => write!(f, "mock scenario catalog failed: {error}"),
            Self::ScenarioNotFound(id) => write!(f, "mock scenario not found: {id:?}"),
            Self::InvalidArgument(argument) => write!(f, "invalid spike argument: {argument}"),
        }
    }
}

impl std::error::Error for SpikeHarnessError {}

/// One deterministic scenario frame exported by the Spike A harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioFrame {
    /// Scenario id that produced this frame.
    pub id: MockScenarioId,
    /// Zero-based frame index within the replay.
    pub frame_index: usize,
    /// Shell view model for this frame.
    pub shell: ShellViewModel,
    /// Stable logical window generation. W2 replay must not recreate windows.
    pub window_generation: u32,
}

/// Deterministic Spike A scenario harness backed only by mock presentation plans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpikeScenarioHarness {
    scenarios: Vec<MockScenario>,
}

impl SpikeScenarioHarness {
    /// Load the deterministic Spike A scenario catalog.
    pub fn new() -> Result<Self, SpikeHarnessError> {
        let scenarios = mock_scenario_catalog().map_err(SpikeHarnessError::Catalog)?;
        Ok(Self { scenarios })
    }

    /// Return scenario ids in catalog order.
    pub fn scenario_ids(&self) -> Vec<MockScenarioId> {
        self.scenarios
            .iter()
            .map(|scenario| scenario.id)
            .collect::<Vec<_>>()
    }

    /// Return deterministic scenario listing lines.
    pub fn scenario_listing(&self) -> Vec<String> {
        self.scenarios
            .iter()
            .map(|scenario| format!("{} {}", scenario_code(scenario.id), scenario.name))
            .collect::<Vec<_>>()
    }

    /// Run one scenario and return its initial shell frame.
    pub fn run(&self, id: MockScenarioId) -> Result<ScenarioFrame, SpikeHarnessError> {
        let scenario = self.scenario(id)?;
        Ok(ScenarioFrame {
            id,
            frame_index: 0,
            shell: ShellViewModel::from_scenario(scenario),
            window_generation: 1,
        })
    }

    /// Replay a scenario's deterministic transition sequence.
    pub fn replay(&self, id: MockScenarioId) -> Result<Vec<ScenarioFrame>, SpikeHarnessError> {
        let scenario = self.scenario(id)?;
        if scenario.transitions.is_empty() {
            return self.run(id).map(|frame| vec![frame]);
        }
        Ok(scenario
            .transitions
            .iter()
            .enumerate()
            .map(|(frame_index, plan)| ScenarioFrame {
                id,
                frame_index,
                shell: ShellViewModel::from_plan(plan, scenario.environment),
                window_generation: 1,
            })
            .collect::<Vec<_>>())
    }

    fn scenario(&self, id: MockScenarioId) -> Result<&MockScenario, SpikeHarnessError> {
        self.scenarios
            .iter()
            .find(|scenario| scenario.id == id)
            .ok_or(SpikeHarnessError::ScenarioNotFound(id))
    }
}

/// Run the disposable Spike A command-line harness and return deterministic text output.
pub fn run_cli<I, S>(args: I) -> Result<String, SpikeHarnessError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut scenario: Option<MockScenarioId> = None;
    let mut replay = false;
    let mut focus_policy = false;
    let mut palette_policy = false;
    let mut layout_policy = false;
    let mut route_policy = false;
    let mut lifecycle_policy = false;
    let mut accessibility_policy = false;
    let mut measurement_policy = false;
    let mut window_policy = false;
    let mut adapter_plan_policy = false;
    let mut adapter_state_policy = false;
    let mut adapter_action_policy = false;
    let mut adapter_replay_policy = false;
    let mut hit_test_policy = false;
    let mut hotkey_policy = false;
    let mut dpi_policy = false;
    let mut cache_policy = false;
    let mut animation_policy = false;
    let mut resource_policy = false;
    let mut overlay_policy = false;
    let mut truth_priority_policy = false;
    let mut surface_handle_policy = false;
    let mut architecture_policy = false;
    let mut gate_audit = false;
    let mut w2_review_ready = false;
    let mut w2_review_manifest = false;
    let mut provider_probe_manifest = false;
    let mut provider_probe_report: Option<ProbeProvider> = None;
    let mut provider_config_fixture: Option<ProbeProvider> = None;
    let mut provider_probe_scorecard = false;
    let mut provider_resource_fixture: Option<ProbeProvider> = None;
    let mut provider_evidence_register: Option<ProbeProvider> = None;
    let mut provider_official_evidence_source_locator: Option<ProbeProvider> = None;
    let mut provider_probe_summary_fixture: Option<ProbeProvider> = None;
    let mut provider_probe_scorecard_fixture = false;
    let mut provider_hard_disqualifier_fixture = false;
    let mut provider_probe_audit = false;
    let mut provider_evidence_gap_summary = false;
    let mut provider_surface_inventory = false;
    let mut provider_probe_readiness = false;
    let mut provider_w5_start_preflight = false;
    let mut provider_local_environment_manifest = false;
    let mut provider_live_probe_dry_run = false;
    let mut provider_live_probe_run = false;
    let mut provider_live_probe_summary = false;
    let mut provider_resource_measurement_plan = false;
    let mut provider_direct_gate_packet: Option<ProbeProvider> = None;
    let mut provider_direct_evidence_import_checklist: Option<ProbeProvider> = None;
    let mut provider_authorized_evidence_runbook: Option<ProbeProvider> = None;
    let mut provider_sanitized_evidence_output_template: Option<ProbeProvider> = None;
    let mut provider_sanitized_evidence_bundle_validator: Option<ProbeProvider> = None;
    let mut provider_release_elevation_preflight: Option<ProbeProvider> = None;
    let mut provider_w5_observe_adapter_contract: Option<ProbeProvider> = None;
    let mut provider_probe_card_execution_plan: Option<ProbeProvider> = None;
    let mut provider_evidence_retention_policy = false;
    let mut provider_live_authorization_preflight: Option<ProbeProvider> = None;
    let mut provider_missing_capability_rationale: Option<ProbeProvider> = None;
    let mut provider_release_decision_log: Option<ProbeProvider> = None;
    let mut provider_capability_matrix: Option<ProbeProvider> = None;
    let mut provider_probe_phase_status: Option<ProbeProvider> = None;
    let mut provider_release_label_evaluation_fixture = false;
    let mut provider_w4_completion_gate = false;
    for (index, arg) in args.into_iter().enumerate() {
        if index == 0 {
            continue;
        }
        let value = arg.as_ref();
        if value == "--window-policy" {
            window_policy = true;
        } else if value == "--adapter-plan-policy" {
            adapter_plan_policy = true;
        } else if value == "--adapter-state-policy" {
            adapter_state_policy = true;
        } else if value == "--adapter-action-policy" {
            adapter_action_policy = true;
        } else if value == "--adapter-replay-policy" {
            adapter_replay_policy = true;
        } else if value == "--measurement-policy" {
            measurement_policy = true;
        } else if value == "--accessibility-policy" {
            accessibility_policy = true;
        } else if value == "--hit-test-policy" {
            hit_test_policy = true;
        } else if value == "--hotkey-policy" {
            hotkey_policy = true;
        } else if value == "--dpi-policy" {
            dpi_policy = true;
        } else if value == "--cache-policy" {
            cache_policy = true;
        } else if value == "--animation-policy" {
            animation_policy = true;
        } else if value == "--resource-policy" {
            resource_policy = true;
        } else if value == "--overlay-policy" {
            overlay_policy = true;
        } else if value == "--truth-priority-policy" {
            truth_priority_policy = true;
        } else if value == "--surface-handle-policy" {
            surface_handle_policy = true;
        } else if value == "--architecture-policy" {
            architecture_policy = true;
        } else if value == "--gate-audit" {
            gate_audit = true;
        } else if value == "--w2-review-ready" {
            w2_review_ready = true;
        } else if value == "--w2-review-manifest" {
            w2_review_manifest = true;
        } else if value == "--provider-probe-manifest" {
            provider_probe_manifest = true;
        } else if let Some(provider_id) = value.strip_prefix("--provider-probe-report=") {
            provider_probe_report = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-config-transaction-fixture=")
        {
            provider_config_fixture = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if value == "--provider-probe-scorecard" {
            provider_probe_scorecard = true;
        } else if value == "--provider-probe-scorecard=sanitized-fixture" {
            provider_probe_scorecard_fixture = true;
        } else if value == "--provider-hard-disqualifiers=sanitized-fixture" {
            provider_hard_disqualifier_fixture = true;
        } else if value == "--provider-probe-audit" {
            provider_probe_audit = true;
        } else if value == "--provider-evidence-gap-summary" {
            provider_evidence_gap_summary = true;
        } else if value == "--provider-surface-inventory" {
            provider_surface_inventory = true;
        } else if value == "--provider-probe-readiness" {
            provider_probe_readiness = true;
        } else if value == "--provider-w5-start-preflight" {
            provider_w5_start_preflight = true;
        } else if value == "--provider-local-environment-manifest=read-only-fixture" {
            provider_local_environment_manifest = true;
        } else if value == "--provider-live-probe-dry-run=read-only-fixture" {
            provider_live_probe_dry_run = true;
        } else if value == "--provider-live-probe-run=read-only-local" {
            provider_live_probe_run = true;
        } else if value == "--provider-live-probe-summary=read-only-local" {
            provider_live_probe_summary = true;
        } else if value == "--provider-resource-measurement-plan=read-only-local" {
            provider_resource_measurement_plan = true;
        } else if let Some(provider_id) = value.strip_prefix("--provider-direct-gate-packet=") {
            provider_direct_gate_packet = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-direct-evidence-import-checklist=")
        {
            provider_direct_evidence_import_checklist = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-authorized-evidence-runbook=")
        {
            provider_authorized_evidence_runbook = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-sanitized-evidence-output-template=")
        {
            provider_sanitized_evidence_output_template = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-sanitized-evidence-bundle-validator=")
        {
            provider_sanitized_evidence_bundle_validator = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-release-elevation-preflight=")
        {
            provider_release_elevation_preflight = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-w5-observe-adapter-contract=")
        {
            provider_w5_observe_adapter_contract = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-probe-card-execution-plan=")
        {
            provider_probe_card_execution_plan = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if value == "--provider-evidence-retention-policy" {
            provider_evidence_retention_policy = true;
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-live-authorization-preflight=")
        {
            provider_live_authorization_preflight = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-missing-capability-rationale=")
        {
            provider_missing_capability_rationale = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) = value.strip_prefix("--provider-release-decision-log=") {
            provider_release_decision_log = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) = value.strip_prefix("--provider-capability-matrix=") {
            provider_capability_matrix = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) = value.strip_prefix("--provider-probe-phase-status=") {
            provider_probe_phase_status = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if value == "--provider-release-label-evaluation=sanitized-fixture" {
            provider_release_label_evaluation_fixture = true;
        } else if value == "--provider-w4-completion-gate" {
            provider_w4_completion_gate = true;
        } else if let Some(provider_id) = value.strip_prefix("--provider-resource-fixture=") {
            provider_resource_fixture = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) = value.strip_prefix("--provider-evidence-register=") {
            provider_evidence_register = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) =
            value.strip_prefix("--provider-official-evidence-source-locator=")
        {
            provider_official_evidence_source_locator = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if let Some(provider_id) = value.strip_prefix("--provider-probe-summary-fixture=") {
            provider_probe_summary_fixture = Some(
                ProbeProvider::from_id(provider_id)
                    .ok_or_else(|| SpikeHarnessError::InvalidArgument(value.to_owned()))?,
            );
        } else if value == "--palette-policy" {
            palette_policy = true;
        } else if value == "--layout-policy" {
            layout_policy = true;
        } else if value == "--route-policy" {
            route_policy = true;
        } else if value == "--lifecycle-policy" {
            lifecycle_policy = true;
        } else if value == "--focus-policy" {
            focus_policy = true;
        } else if value == "--replay" {
            replay = true;
        } else if scenario.is_none() {
            scenario = Some(parse_scenario_id(value)?);
        } else {
            return Err(SpikeHarnessError::InvalidArgument(value.to_owned()));
        }
    }

    if focus_policy {
        return Ok(focus_policy_diagnostics());
    }
    if palette_policy {
        return Ok(palette_policy_diagnostics());
    }
    if layout_policy {
        return Ok(layout_policy_diagnostics());
    }
    if route_policy {
        return route_policy_diagnostics();
    }
    if lifecycle_policy {
        return Ok(lifecycle_policy_diagnostics());
    }
    if accessibility_policy {
        return Ok(accessibility_policy_diagnostics());
    }
    if measurement_policy {
        return Ok(measurement_policy_diagnostics());
    }
    if window_policy {
        return Ok(window_policy_diagnostics());
    }
    if adapter_plan_policy {
        return Ok(adapter_plan_policy_diagnostics());
    }
    if adapter_state_policy {
        return Ok(adapter_state_policy_diagnostics());
    }
    if adapter_action_policy {
        return Ok(adapter_action_policy_diagnostics());
    }
    if adapter_replay_policy {
        return adapter_replay_policy_diagnostics();
    }
    if hit_test_policy {
        return Ok(hit_test_policy_diagnostics());
    }
    if hotkey_policy {
        return Ok(hotkey_policy_diagnostics());
    }
    if dpi_policy {
        return Ok(dpi_policy_diagnostics());
    }
    if cache_policy {
        return Ok(cache_policy_diagnostics());
    }
    if animation_policy {
        return Ok(animation_policy_diagnostics());
    }
    if resource_policy {
        return Ok(resource_policy_diagnostics());
    }
    if overlay_policy {
        return Ok(overlay_policy_diagnostics());
    }
    if truth_priority_policy {
        return Ok(truth_priority_policy_diagnostics());
    }
    if surface_handle_policy {
        return Ok(surface_handle_policy_diagnostics());
    }
    if architecture_policy {
        return Ok(architecture_policy_diagnostics());
    }
    if gate_audit {
        return Ok(gate_audit_diagnostics());
    }
    if w2_review_ready {
        return Ok(w2_review_ready_diagnostics());
    }
    if w2_review_manifest {
        return Ok(w2_review_manifest_diagnostics());
    }
    if provider_probe_manifest {
        return Ok(provider_probe_manifest_diagnostics());
    }
    if let Some(provider) = provider_probe_report {
        return Ok(provider_probe_report_diagnostics(provider));
    }
    if let Some(provider) = provider_config_fixture {
        return Ok(provider_config_transaction_fixture_diagnostics(provider));
    }
    if provider_probe_scorecard {
        return Ok(provider_probe_scorecard_diagnostics());
    }
    if provider_probe_scorecard_fixture {
        return Ok(provider_probe_scorecard_fixture_diagnostics());
    }
    if provider_hard_disqualifier_fixture {
        return Ok(provider_hard_disqualifier_fixture_diagnostics());
    }
    if provider_probe_audit {
        return Ok(provider_probe_audit_diagnostics());
    }
    if provider_evidence_gap_summary {
        return Ok(provider_evidence_gap_summary_diagnostics());
    }
    if provider_surface_inventory {
        return Ok(provider_surface_inventory_diagnostics());
    }
    if provider_probe_readiness {
        return Ok(provider_probe_readiness_diagnostics());
    }
    if provider_w5_start_preflight {
        return Ok(provider_w5_start_preflight_diagnostics());
    }
    if provider_local_environment_manifest {
        return Ok(provider_local_environment_manifest_diagnostics());
    }
    if provider_live_probe_dry_run {
        return Ok(provider_live_probe_dry_run_diagnostics());
    }
    if provider_live_probe_run {
        return Ok(provider_live_probe_run_diagnostics());
    }
    if provider_live_probe_summary {
        return Ok(provider_live_probe_summary_diagnostics());
    }
    if provider_resource_measurement_plan {
        return Ok(provider_resource_measurement_plan_diagnostics());
    }
    if let Some(provider) = provider_direct_gate_packet {
        return Ok(provider_direct_gate_packet_diagnostics(provider));
    }
    if let Some(provider) = provider_direct_evidence_import_checklist {
        return Ok(provider_direct_evidence_import_checklist_diagnostics(
            provider,
        ));
    }
    if let Some(provider) = provider_authorized_evidence_runbook {
        return Ok(provider_authorized_evidence_runbook_diagnostics(provider));
    }
    if let Some(provider) = provider_sanitized_evidence_output_template {
        return Ok(provider_sanitized_evidence_output_template_diagnostics(
            provider,
        ));
    }
    if let Some(provider) = provider_sanitized_evidence_bundle_validator {
        return Ok(provider_sanitized_evidence_bundle_validator_diagnostics(
            provider,
        ));
    }
    if let Some(provider) = provider_release_elevation_preflight {
        return Ok(provider_release_elevation_preflight_diagnostics(provider));
    }
    if let Some(provider) = provider_w5_observe_adapter_contract {
        return Ok(provider_w5_observe_adapter_contract_diagnostics(provider));
    }
    if let Some(provider) = provider_probe_card_execution_plan {
        return Ok(provider_probe_card_execution_plan_diagnostics(provider));
    }
    if provider_evidence_retention_policy {
        return Ok(provider_evidence_retention_policy_diagnostics());
    }
    if let Some(provider) = provider_live_authorization_preflight {
        return Ok(provider_live_authorization_preflight_diagnostics(provider));
    }
    if let Some(provider) = provider_missing_capability_rationale {
        return Ok(provider_missing_capability_rationale_diagnostics(provider));
    }
    if let Some(provider) = provider_release_decision_log {
        return Ok(provider_release_decision_log_diagnostics(provider));
    }
    if let Some(provider) = provider_capability_matrix {
        return Ok(provider_capability_matrix_diagnostics(provider));
    }
    if let Some(provider) = provider_probe_phase_status {
        return Ok(provider_probe_phase_status_diagnostics(provider));
    }
    if provider_release_label_evaluation_fixture {
        return Ok(provider_release_label_evaluation_fixture_diagnostics());
    }
    if provider_w4_completion_gate {
        return Ok(provider_w4_completion_gate_diagnostics());
    }
    if let Some(provider) = provider_resource_fixture {
        return Ok(provider_resource_fixture_diagnostics(provider));
    }
    if let Some(provider) = provider_evidence_register {
        return Ok(provider_evidence_register_diagnostics(provider));
    }
    if let Some(provider) = provider_official_evidence_source_locator {
        return Ok(provider_official_evidence_source_locator_diagnostics(
            provider,
        ));
    }
    if let Some(provider) = provider_probe_summary_fixture {
        return Ok(provider_probe_summary_fixture_diagnostics(provider));
    }

    let harness = SpikeScenarioHarness::new()?;
    let Some(id) = scenario else {
        return Ok(harness.scenario_listing().join("\n"));
    };
    let frames = if replay {
        harness.replay(id)?
    } else {
        vec![harness.run(id)?]
    };
    Ok(frames
        .iter()
        .map(format_frame)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn window_policy_diagnostics() -> String {
    let style = CompactWindowStylePolicy::default();
    let normal = ImmersiveWindowPolicy::for_state(ImmersiveState::Normal);
    let fullscreen = ImmersiveWindowPolicy::for_state(ImmersiveState::Fullscreen);
    [
        format!(
            "style popup={} topmost={} toolwindow={} noactivate={}",
            style.popup, style.topmost, style.tool_window, style.no_activate
        ),
        format!(
            "alt_tab_visible={} permanently_click_through={}",
            style.alt_tab_visible, style.permanently_click_through
        ),
        format!(
            "normal visibility={:?} topmost={} replay_missed={}",
            normal.visibility, normal.keep_topmost, normal.replay_missed_animations_on_restore
        ),
        format!(
            "fullscreen visibility={:?} topmost={} replay_missed={}",
            fullscreen.visibility,
            fullscreen.keep_topmost,
            fullscreen.replay_missed_animations_on_restore
        ),
    ]
    .join("\n")
}

fn adapter_plan_policy_diagnostics() -> String {
    let visible = NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Normal));
    let immersive =
        NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Fullscreen));
    [
        format_adapter_plan("visible", visible),
        format_adapter_plan("immersive", immersive),
    ]
    .join("\n")
}

fn adapter_state_policy_diagnostics() -> String {
    let plans = [
        NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Normal)),
        NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Fullscreen)),
        NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Normal)),
    ];
    let state = plans
        .iter()
        .copied()
        .fold(NativeWindowAdapterState::default(), |state, plan| {
            state.apply(plan)
        });
    format!(
        "adapter_state frames={} generation={} create_calls={} destroy_calls={} hotkey_registered={} register_calls={} unregister_calls={} activation_attempts={} final_visibility={:?}",
        plans.len(),
        state.window_generation,
        state.create_window_calls,
        state.destroy_window_calls,
        state.hotkey_registered,
        state.hotkey_register_calls,
        state.hotkey_unregister_calls,
        state.activation_attempts,
        state.visibility
    )
}

fn adapter_action_policy_diagnostics() -> String {
    let visible = NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Normal));
    let immersive =
        NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Fullscreen));
    let mut hit_test_input = adapter_input(true, ImmersiveState::Normal);
    hit_test_input.hit_test.transparent_margin_px = 12;
    hit_test_input.hit_test.drag_grip_width_px = 40;
    let hit_test_changed = NativeWindowAdapterPlan::from_input(hit_test_input);
    let mut placement_input = adapter_input(true, ImmersiveState::Normal);
    placement_input.placement.origin.x = 140;
    let placement_changed = NativeWindowAdapterPlan::from_input(placement_input);
    let mut style_input = adapter_input(true, ImmersiveState::Normal);
    style_input.style_policy.topmost = false;
    let style_changed = NativeWindowAdapterPlan::from_input(style_input);
    let mut hotkey_input = adapter_input(true, ImmersiveState::Normal);
    hotkey_input.hotkey_policy.enabled = false;
    let hotkey_disabled = NativeWindowAdapterPlan::from_input(hotkey_input);
    let initial = NativeWindowAdapterState::default();
    let visible_state = initial.apply(visible);
    [
        format_adapter_actions("initial", initial.actions_for(visible)),
        format_adapter_actions("repeated", visible_state.actions_for(visible)),
        format_adapter_actions("immersive", visible_state.actions_for(immersive)),
        format_adapter_actions(
            "hit_test_changed",
            visible_state.actions_for(hit_test_changed),
        ),
        format_adapter_actions(
            "placement_changed",
            visible_state.actions_for(placement_changed),
        ),
        format_adapter_actions("style_changed", visible_state.actions_for(style_changed)),
        format_adapter_actions(
            "hotkey_disabled",
            visible_state.actions_for(hotkey_disabled),
        ),
    ]
    .join("\n")
}

fn adapter_replay_policy_diagnostics() -> Result<String, SpikeHarnessError> {
    let harness = SpikeScenarioHarness::new()?;
    let s7_frames = harness.replay(MockScenarioId::S7RapidStateChanges)?;
    let (s7_state, total_actions, max_actions_per_frame) = replay_adapter_frames(&s7_frames);
    let s8_frame = harness.run(MockScenarioId::S8ImmersiveSimulation)?;
    let visible = NativeWindowAdapterPlan::from_input(adapter_input(true, ImmersiveState::Normal));
    let s8_plan = adapter_plan_for_shell(&s8_frame.shell);
    let visible_state = NativeWindowAdapterState::default().apply(visible);
    let immersive_actions = visible_state.actions_for(s8_plan);
    let s8_state = visible_state.apply(s8_plan);
    Ok([
        format!(
            "adapter_replay s7 frames={} generation={} create_calls={} destroy_calls={} hotkey_register_calls={} hotkey_unregister_calls={} activation_attempts={} total_actions={} max_actions_per_frame={} final_visibility={:?}",
            s7_frames.len(),
            s7_state.window_generation,
            s7_state.create_window_calls,
            s7_state.destroy_window_calls,
            s7_state.hotkey_register_calls,
            s7_state.hotkey_unregister_calls,
            s7_state.activation_attempts,
            total_actions,
            max_actions_per_frame,
            s7_state.visibility
        ),
        format!(
            "adapter_replay s8 final_visibility={:?} immersive_actions={}",
            s8_state.visibility,
            format_action_list(immersive_actions)
        ),
    ]
    .join("\n"))
}

fn replay_adapter_frames(frames: &[ScenarioFrame]) -> (NativeWindowAdapterState, usize, usize) {
    frames
        .iter()
        .map(|frame| adapter_plan_for_shell(&frame.shell))
        .fold(
            (NativeWindowAdapterState::default(), 0, 0),
            |(state, total_actions, max_actions_per_frame), plan| {
                let actions = state.actions_for(plan);
                let action_count = actions.len();
                (
                    state.apply(plan),
                    total_actions + action_count,
                    max_actions_per_frame.max(action_count),
                )
            },
        )
}

fn adapter_plan_for_shell(shell: &ShellViewModel) -> NativeWindowAdapterPlan {
    let immersive = if shell.motion_policy == MotionPolicy::Stopped && !shell.compact_visible {
        ImmersiveState::Fullscreen
    } else {
        ImmersiveState::Normal
    };
    NativeWindowAdapterPlan::from_input(adapter_input(shell.compact_visible, immersive))
}

fn format_adapter_actions(label: &str, actions: Vec<NativeWindowAdapterAction>) -> String {
    format!("adapter_actions {label}={}", format_action_list(actions))
}

fn format_action_list(actions: Vec<NativeWindowAdapterAction>) -> String {
    if actions.is_empty() {
        "<none>".to_owned()
    } else {
        actions
            .iter()
            .map(|action| format!("{action:?}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn adapter_input(
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

fn format_adapter_plan(label: &str, plan: NativeWindowAdapterPlan) -> String {
    format!(
        "adapter_plan {label} visibility={:?} topmost={} create={} recreate={} destroy_hidden={} activate={} style_noactivate={} style_transparent={} hotkey={:?} replay_missed={}",
        plan.visibility,
        plan.keep_topmost,
        plan.create_compact_window_if_missing,
        plan.recreate_compact_window,
        plan.destroy_compact_window_when_hidden,
        plan.activate_on_show,
        plan.style_bits.has_no_activate(),
        plan.style_bits.has_transparent(),
        plan.hotkey.map(|hotkey| hotkey.id),
        plan.replay_missed_animations_on_restore
    )
}

fn hit_test_policy_diagnostics() -> String {
    let layout = HitTestLayout {
        width_px: 240,
        height_px: 56,
        transparent_margin_px: 8,
        drag_grip_width_px: 32,
    };
    [
        format_hit_test("margin", layout.hit_test(PointPx { x: 2, y: 20 })),
        format_hit_test("drag", layout.hit_test(PointPx { x: 16, y: 20 })),
        format_hit_test("client", layout.hit_test(PointPx { x: 80, y: 20 })),
        format_hit_test("outside", layout.hit_test(PointPx { x: 300, y: 20 })),
    ]
    .join("\n")
}

fn hotkey_policy_diagnostics() -> String {
    let hotkey = GlobalHotkeyPolicy::palette_default();
    let Some(chord) = hotkey.registration_chord() else {
        return "hotkey enabled=false".to_owned();
    };
    let normal = ShellInteractionState::default().apply_palette_invocation(
        PaletteInvocationPolicy::default(),
        ShellEnvironment::default(),
    );
    let immersive_allowed = ShellInteractionState::default().apply_palette_invocation(
        PaletteInvocationPolicy {
            global_shortcut_enabled: true,
            allow_during_immersive: true,
        },
        ShellEnvironment {
            immersive_active: true,
            reduced_motion: false,
            high_contrast: false,
        },
    );
    let immersive_blocked = ShellInteractionState::default().apply_palette_invocation(
        PaletteInvocationPolicy {
            global_shortcut_enabled: true,
            allow_during_immersive: false,
        },
        ShellEnvironment {
            immersive_active: true,
            reduced_motion: false,
            high_contrast: false,
        },
    );
    [
        format!(
            "hotkey enabled=true id={} modifiers={} virtual_key={}",
            chord.id, chord.modifiers, chord.virtual_key
        ),
        format_interaction("normal", &normal),
        format_interaction("immersive_allowed", &immersive_allowed),
        format_interaction("immersive_blocked", &immersive_blocked),
    ]
    .join("\n")
}

fn dpi_policy_diagnostics() -> String {
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
    let mut lines = Vec::new();
    if let Some(resolved) = remembered.resolve(&topology) {
        lines.push(format!(
            "resolved_monitor={} origin=({},{}) size=({},{})",
            resolved.monitor_id.0,
            resolved.placement.origin.x,
            resolved.placement.origin.y,
            resolved.placement.size.width,
            resolved.placement.size.height
        ));
    }
    if let Some(fallback) = missing_monitor.resolve(&topology) {
        lines.push(format!(
            "fallback_monitor={} origin=({},{})",
            fallback.monitor_id.0, fallback.placement.origin.x, fallback.placement.origin.y
        ));
    }
    lines.push(format!(
        "text logical=12 dpi=144 scale=150 physical={}",
        DpiScale::new(144).scale_text_px(12, TextScalePercent::new(150))
    ));
    lines.push(format!(
        "text logical=1 dpi=48 scale=50 physical={}",
        DpiScale::new(48).scale_text_px(1, TextScalePercent::new(50))
    ));
    lines.join("\n")
}

fn cache_policy_diagnostics() -> String {
    let policy = RenderCachePolicy::w2();
    let mut lines = vec![format!(
        "text_layout_cache_bounded={} max={} task_content={}",
        policy.text_layout_cache_bounded,
        policy.max_cached_text_layouts,
        policy.caches_task_content
    )];
    for trigger in [
        RenderCacheInvalidation::DpiChanged,
        RenderCacheInvalidation::ThemeChanged,
        RenderCacheInvalidation::FontChanged,
        RenderCacheInvalidation::StateLayoutChanged,
    ] {
        let plan = policy.invalidate(trigger);
        lines.push(format!(
            "invalidation={:?} geometry={} text={} brushes={} unbounded={}",
            trigger,
            plan.invalidates_geometry,
            plan.invalidates_text_layouts,
            plan.invalidates_brushes,
            plan.allows_unbounded_growth
        ));
    }
    lines.join("\n")
}

fn animation_policy_diagnostics() -> String {
    [
        CompositorAnimationClass::Arrival,
        CompositorAnimationClass::StateTransition,
        CompositorAnimationClass::AttentionPulse,
        CompositorAnimationClass::Expansion,
        CompositorAnimationClass::FuelCue,
        CompositorAnimationClass::Completion,
    ]
    .iter()
    .map(|animation_class| {
        let plan = CompositorAnimationPlan::for_class(*animation_class, MotionPolicy::Normal);
        format!(
            "animation={:?} compositor={} frame_loop={} duration_ms={} interruptible={} repetitions={:?} settles={} pulse={} scale={}",
            animation_class,
            plan.compositor_owned,
            plan.app_side_frame_loop_allowed,
            plan.duration_ms,
            plan.interruptible,
            plan.max_repetitions,
            plan.settles_to_static,
            plan.pulse_allowed,
            plan.scale_allowed
        )
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn resource_policy_diagnostics() -> String {
    let policy = RenderResourcePolicy::w2();
    let stable = RenderResourceReport::from_snapshots(
        policy,
        RenderResourceSnapshot::new(1, 3, 4, 24),
        RenderResourceSnapshot::new(1, 3, 4, 24),
        1_000,
    );
    [
        format!(
            "resource_policy shared_device={} max_devices={} per_task_surfaces={} virtualized_rows={}",
            policy.shared_d3d_device_required,
            policy.max_shared_d3d_devices,
            policy.per_task_full_render_surfaces_allowed,
            policy.focus_card_rows_virtualized
        ),
        format!(
            "resource_report transitions={} passed={} device_growth={} surface_growth={} d3d_growth={} handle_growth={}",
            stable.state_transition_count,
            stable.passed,
            stable.device_growth,
            stable.surface_growth,
            stable.d3d_resource_growth,
            stable.handle_growth
        ),
    ]
    .join("\n")
}

fn overlay_policy_diagnostics() -> String {
    let hidden = PerformanceOverlayViewModel::from_report(
        OverlayMode::Normal,
        MeasurementReport::from_samples(
            MeasurementPolicy::gate_a(),
            deterministic_gate_a_samples(),
        ),
    );
    let visible = PerformanceOverlayViewModel::from_report(
        OverlayMode::Diagnostics,
        MeasurementReport::from_samples(
            MeasurementPolicy::gate_a(),
            deterministic_gate_a_samples(),
        ),
    );
    [
        format!(
            "overlay normal visible={} rows={} task_content={}",
            hidden.visible,
            hidden.rows.len(),
            hidden.includes_task_content
        ),
        format!(
            "overlay diagnostics visible={} rows={} metadata_only={} task_content={}",
            visible.visible,
            visible.rows.len(),
            visible.diagnostic_metadata_only,
            visible.includes_task_content
        ),
        format!("overlay diagnostics passed={}", visible.passed),
    ]
    .join("\n")
}

fn truth_priority_policy_diagnostics() -> String {
    let signal = SignalViewModel {
        primary_task_id: Some("waiting-primary".to_owned()),
        state: SignalState::Waiting,
        overflow_count: 3,
        primary_route_label: None,
    };
    let decision =
        SignalTruthPriorityDecision::from_signal(&signal, FuelThreadCandidate::TrustworthyLow);
    [
        format!(
            "truth_priority primary={:?} state={:?} source={:?}",
            decision.primary_task_id, decision.primary_state, decision.primary_story_source
        ),
        format!(
            "truth_priority timer_rotation={} fuel_role={:?} fuel_visible={} fuel_override={}",
            decision.timer_rotation_allowed,
            decision.fuel_thread_role,
            decision.fuel_thread_visible,
            decision.fuel_can_override_primary_state
        ),
    ]
    .join("\n")
}

fn surface_handle_policy_diagnostics() -> String {
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
    let report = SurfaceHandleStabilityReport::from_lifecycle(
        lifecycle,
        SurfaceHandleSnapshot::new(12, 8),
        SurfaceHandleSnapshot::new(12, 8),
    );
    [
        format!(
            "surface_handles cycles={} required={} passed={}",
            report.actual_open_close_cycles, report.required_open_close_cycles, report.passed
        ),
        format!(
            "surface_handles user_growth={} gdi_growth={}",
            report.user_handle_growth, report.gdi_handle_growth
        ),
    ]
    .join("\n")
}

fn architecture_policy_manifests() -> [(&'static str, &'static str); 5] {
    [
        (
            "pulse-island-ui",
            include_str!("../../../crates/pulse-island-ui/Cargo.toml"),
        ),
        (
            "pulse-win32",
            include_str!("../../../crates/pulse-win32/Cargo.toml"),
        ),
        (
            "pulse-win32-hwnd",
            include_str!("../../../crates/pulse-win32-hwnd/Cargo.toml"),
        ),
        (
            "pulse-win32-link",
            include_str!("../../../crates/pulse-win32-link/Cargo.toml"),
        ),
        ("pulse-island-spike", include_str!("../Cargo.toml")),
    ]
}

fn architecture_policy_manifest_blob() -> String {
    architecture_policy_manifests()
        .iter()
        .map(|(_, manifest)| *manifest)
        .collect::<Vec<_>>()
        .join("\n")
}

fn architecture_policy_diagnostics() -> String {
    let manifests = architecture_policy_manifests();
    let manifest_blob = architecture_policy_manifest_blob();
    let forbidden_dependency_hits = architecture_forbidden_dependency_hits();
    let ui_source = include_str!("../../../crates/pulse-island-ui/src/lib.rs");
    let mock_plan_replaceable = ui_source.contains("pub trait PresentationPlanSource")
        && ui_source.contains("fn current_plan(&self) -> PresentationPlan")
        && ui_source.contains("fn subscribe(&self, callback: PlanChangedCallback<'_>)")
        && ui_source.contains("pub struct MockPresentationPlanSource");
    let hwnd_boundary_manifest = manifests
        .iter()
        .any(|(name, _)| *name == "pulse-win32-hwnd");
    let link_transport_boundary_manifest = manifests
        .iter()
        .any(|(name, _)| *name == "pulse-win32-link");
    let hwnd_source = include_str!("../../../crates/pulse-win32-hwnd/src/lib.rs");
    let hwnd_native_api_adapter = hwnd_source.contains("pub struct WindowsSysHwndApi")
        && hwnd_source.contains("impl HwndNativeApi for WindowsSysHwndApi")
        && hwnd_source.contains("SetWindowLongPtrW")
        && hwnd_source.contains("RegisterHotKey");
    let hwnd_create_window_factory = hwnd_source
        .contains("pub struct WindowsSysCompactWindowFactory")
        && hwnd_source.contains("CreateWindowExW")
        && hwnd_source.contains("RegisterClassW")
        && hwnd_source.contains("DestroyWindow");
    let hwnd_message_pump = hwnd_source.contains("pub struct WindowsSysMessagePump")
        && hwnd_source.contains("PeekMessageW")
        && hwnd_source.contains("DispatchMessageW")
        && hwnd_source.contains("PM_REMOVE");
    let hwnd_wndproc_hit_test = hwnd_source.contains("pub struct HwndHitTestBridge")
        && hwnd_source.contains("WM_NCHITTEST")
        && hwnd_source.contains("ScreenToClient")
        && hwnd_source.contains("Win32HitTestCode");
    let hwnd_wndproc_mouse_activate =
        hwnd_source.contains("WM_MOUSEACTIVATE") && hwnd_source.contains("MA_NOACTIVATE");
    let hwnd_wndproc_mouse_dispatch = hwnd_source.contains("HwndMouseInputBridge")
        && hwnd_source.contains("HwndMouseInputEvent")
        && hwnd_source.contains("WM_LBUTTONUP")
        && hwnd_source.contains("CompactPrimaryClick");
    let hwnd_wndproc_paint_dispatch = hwnd_source.contains("HwndPaintBridge")
        && hwnd_source.contains("HwndRenderEvent")
        && hwnd_source.contains("WM_PAINT")
        && hwnd_source.contains("CompactRepaintRequested");
    let browser_runtime = forbidden_manifest_hits(
        &manifest_blob,
        &[
            "tauri", "electron", "webview", "web-view", "web_view", "wry", "cef", "chromium",
        ],
    )
    .next()
    .is_some();
    let sqlite = forbidden_manifest_hits(&manifest_blob, &["sqlite", "rusqlite", "sqlx"])
        .next()
        .is_some();
    let provider_adapter =
        forbidden_manifest_hits(&manifest_blob, &["provider-adapter", "provider_adapter"])
            .next()
            .is_some();
    let passed = forbidden_dependency_hits == 0 && mock_plan_replaceable;

    [
        format!(
            "architecture_policy checked_manifests={} passed={}",
            manifests.len(),
            passed
        ),
        format!("forbidden_dependency_hits={forbidden_dependency_hits}"),
        format!("mock_plan_replaceable={mock_plan_replaceable}"),
        format!("hwnd_boundary_manifest={hwnd_boundary_manifest}"),
        format!("link_transport_boundary_manifest={link_transport_boundary_manifest}"),
        format!("hwnd_native_api_adapter={hwnd_native_api_adapter}"),
        format!("hwnd_create_window_factory={hwnd_create_window_factory}"),
        format!("hwnd_message_pump={hwnd_message_pump}"),
        format!("hwnd_wndproc_hit_test={hwnd_wndproc_hit_test}"),
        format!("hwnd_wndproc_mouse_activate={hwnd_wndproc_mouse_activate}"),
        format!("hwnd_wndproc_mouse_dispatch={hwnd_wndproc_mouse_dispatch}"),
        format!("hwnd_wndproc_paint_dispatch={hwnd_wndproc_paint_dispatch}"),
        format!(
            "browser_runtime={} sqlite={} provider_adapter={}",
            browser_runtime, sqlite, provider_adapter
        ),
    ]
    .join("\n")
}

fn architecture_forbidden_dependency_hits() -> usize {
    const FORBIDDEN_DEPENDENCIES: [&str; 13] = [
        "sqlite",
        "rusqlite",
        "sqlx",
        "tauri",
        "electron",
        "webview",
        "web-view",
        "web_view",
        "wry",
        "cef",
        "chromium",
        "provider-adapter",
        "provider_adapter",
    ];
    architecture_policy_manifests()
        .iter()
        .map(|(_, manifest)| *manifest)
        .flat_map(|manifest| forbidden_manifest_hits(manifest, &FORBIDDEN_DEPENDENCIES))
        .count()
}

fn forbidden_manifest_hits<'a>(
    manifest: &'a str,
    forbidden_dependencies: &'a [&str],
) -> impl Iterator<Item = &'a str> {
    forbidden_dependencies
        .iter()
        .copied()
        .filter(move |dependency| manifest_contains_dependency(manifest, dependency))
}

fn manifest_contains_dependency(manifest: &str, dependency: &str) -> bool {
    manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .any(|line| {
            line.strip_prefix(dependency)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
}

fn gate_audit_diagnostics() -> String {
    [
        "gate_audit checklist_items=21 evidence_items=21".to_owned(),
        "functional=6/6 window=6/6 performance=5/5 architecture=4/4".to_owned(),
        "w3_ready=true w2_review=accepted".to_owned(),
        "w4_ready=true w3_review=accepted".to_owned(),
        "active_work=W4_Provider_Probe_Harness".to_owned(),
        "scope=mock_presentation_plan_only".to_owned(),
    ]
    .join("\n")
}

fn w2_review_ready_diagnostics() -> String {
    let manifest = W2ReviewManifest::current();
    [
        format!("w2_review_ready={}", manifest.w2_review_ready),
        format!(
            "gate_audit={}/{}",
            manifest.gate_audit_evidence, manifest.gate_audit_checklist
        ),
        format!("adapter_readiness={}", manifest.adapter_readiness.join(",")),
        format!("evidence_doc={}", manifest.evidence_doc),
        format!("scope={}", manifest.scope),
        format!("w3_ready={}", manifest.w3_ready),
    ]
    .join("\n")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct W2ReviewManifest {
    manifest_version: u8,
    package: &'static str,
    scope: &'static str,
    review_status: &'static str,
    w2_review_ready: bool,
    w3_ready: bool,
    w3_authorized_scope: &'static str,
    gate_audit_checklist: u8,
    gate_audit_evidence: u8,
    adapter_readiness: [&'static str; 4],
    evidence_doc: &'static str,
    forbidden_dependency_hits: usize,
    later_gated_work: [&'static str; 4],
}

impl W2ReviewManifest {
    fn current() -> Self {
        Self {
            manifest_version: 1,
            package: "W2 Native Signal Shell",
            scope: "mock_presentation_plan_only",
            review_status: "accepted_for_w3",
            w2_review_ready: true,
            w3_ready: true,
            w3_authorized_scope: "link_shim_drop_mode_synthetic_only",
            gate_audit_checklist: 21,
            gate_audit_evidence: 21,
            adapter_readiness: ["plan", "state", "action", "replay"],
            evidence_doc: "docs/pulse-island/W2-GATE-AUDIT.md",
            forbidden_dependency_hits: architecture_forbidden_dependency_hits(),
            later_gated_work: [
                "live_provider_hooks",
                "provider_adapters",
                "provider_config",
                "route_activation",
            ],
        }
    }

    fn diagnostics(self) -> String {
        [
            format!("manifest_version={}", self.manifest_version),
            format!("package={}", self.package),
            format!("scope={}", self.scope),
            format!("review_status={}", self.review_status),
            format!("w2_review_ready={}", self.w2_review_ready),
            format!("w3_ready={}", self.w3_ready),
            format!("w3_authorized_scope={}", self.w3_authorized_scope),
            format!("gate_audit_checklist={}", self.gate_audit_checklist),
            format!("gate_audit_evidence={}", self.gate_audit_evidence),
            format!("adapter_readiness={}", self.adapter_readiness.join(",")),
            format!("evidence_doc={}", self.evidence_doc),
            format!(
                "forbidden_dependency_hits={}",
                self.forbidden_dependency_hits
            ),
            format!("later_gated_work={}", self.later_gated_work.join(",")),
        ]
        .join("\n")
    }
}

fn w2_review_manifest_diagnostics() -> String {
    W2ReviewManifest::current().diagnostics()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeProvider {
    CodexCli,
    ClaudeCode,
    Antigravity,
}

impl ProbeProvider {
    fn from_id(value: &str) -> Option<Self> {
        match value {
            "codex_cli" => Some(Self::CodexCli),
            "claude_code" => Some(Self::ClaudeCode),
            "antigravity" => Some(Self::Antigravity),
            _ => None,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::CodexCli => "codex_cli",
            Self::ClaudeCode => "claude_code",
            Self::Antigravity => "antigravity",
        }
    }

    fn probe_card(self) -> &'static str {
        match self {
            Self::CodexCli => "docs/pulse-island/16-codex-cli-probe-card.md",
            Self::ClaudeCode => "docs/pulse-island/17-claude-code-probe-card.md",
            Self::Antigravity => "docs/pulse-island/19-antigravity-probe-card.md",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProviderProbeManifest {
    manifest_version: u8,
    package: &'static str,
    mode: &'static str,
    active_work: &'static str,
    protocol_doc: &'static str,
    providers: [ProbeProvider; 3],
    required_fields: [&'static str; 7],
    forbidden_actions: [&'static str; 6],
}

impl ProviderProbeManifest {
    fn current() -> Self {
        Self {
            manifest_version: 1,
            package: "W4 Provider Probe Harness",
            mode: "read_only_capability_discovery",
            active_work: "W4_Provider_Probe_Harness",
            protocol_doc: "docs/pulse-island/15-provider-capability-probe.md",
            providers: [
                ProbeProvider::CodexCli,
                ProbeProvider::ClaudeCode,
                ProbeProvider::Antigravity,
            ],
            required_fields: [
                "version",
                "environment_category",
                "integration_mode",
                "capability_matrix",
                "known_limitations",
                "resource_figures",
                "release_recommendation",
            ],
            forbidden_actions: [
                "live_hook_install",
                "provider_config_mutation",
                "provider_adapter_creation",
                "network_query",
                "transcript_or_session_file_parsing",
                "production_route_activation",
            ],
        }
    }

    fn diagnostics(self) -> String {
        let mut lines = vec![
            format!("manifest_version={}", self.manifest_version),
            format!("package={}", self.package),
            format!("mode={}", self.mode),
            format!("active_work={}", self.active_work),
            format!("protocol_doc={}", self.protocol_doc),
            format!("required_fields={}", self.required_fields.join(",")),
            format!("forbidden_actions={}", self.forbidden_actions.join(",")),
            "raw_provider_content=false".to_owned(),
            "raw_provider_configuration=false".to_owned(),
        ];
        lines.extend(self.providers.iter().copied().map(|provider| {
            format!(
                "provider={} release=not_probed probe_card={}",
                provider.id(),
                provider.probe_card()
            )
        }));
        lines.join("\n")
    }
}

fn provider_probe_manifest_diagnostics() -> String {
    ProviderProbeManifest::current().diagnostics()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCapability {
    DiscoverProcess,
    DiscoverSession,
    ObserveRunning,
    ObserveWaiting,
    ObserveCompletion,
    ObserveFailure,
    ObserveSafeTitle,
    OpenWorkspace,
    OpenExactContext,
    OpenOfficialUsage,
    ObserveSessionTokens,
    ObserveQuotaSnapshot,
    ObserveQuotaLimit,
    ControlDecision,
    ControlStopSteerResume,
}

impl ProbeCapability {
    const ALL: [Self; 15] = [
        Self::DiscoverProcess,
        Self::DiscoverSession,
        Self::ObserveRunning,
        Self::ObserveWaiting,
        Self::ObserveCompletion,
        Self::ObserveFailure,
        Self::ObserveSafeTitle,
        Self::OpenExactContext,
        Self::OpenWorkspace,
        Self::OpenOfficialUsage,
        Self::ObserveSessionTokens,
        Self::ObserveQuotaSnapshot,
        Self::ObserveQuotaLimit,
        Self::ControlDecision,
        Self::ControlStopSteerResume,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::DiscoverProcess => "discover_process",
            Self::DiscoverSession => "discover_session",
            Self::ObserveRunning => "observe_running",
            Self::ObserveWaiting => "observe_waiting",
            Self::ObserveCompletion => "observe_completion",
            Self::ObserveFailure => "observe_failure",
            Self::ObserveSafeTitle => "observe_safe_title",
            Self::OpenWorkspace => "open_workspace",
            Self::OpenExactContext => "open_exact_context",
            Self::OpenOfficialUsage => "open_official_usage",
            Self::ObserveSessionTokens => "observe_session_tokens",
            Self::ObserveQuotaSnapshot => "observe_quota_snapshot",
            Self::ObserveQuotaLimit => "observe_quota_limit",
            Self::ControlDecision => "control_decision",
            Self::ControlStopSteerResume => "control_stop_steer_resume",
        }
    }

    fn missing_reason(self) -> &'static str {
        match self {
            Self::DiscoverProcess
            | Self::DiscoverSession
            | Self::ObserveRunning
            | Self::ObserveWaiting
            | Self::ObserveSafeTitle => "direct_gate_missing",
            Self::ObserveCompletion | Self::ObserveFailure => "terminal_truth_missing",
            Self::OpenWorkspace | Self::OpenExactContext => "context_route_missing",
            Self::OpenOfficialUsage
            | Self::ObserveSessionTokens
            | Self::ObserveQuotaSnapshot
            | Self::ObserveQuotaLimit => "fuel_source_missing",
            Self::ControlDecision | Self::ControlStopSteerResume => "control_safety_review_missing",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProviderProbeReport {
    report_version: u8,
    provider: ProbeProvider,
    tested_version: &'static str,
    environment_category: &'static str,
    integration_mode: &'static str,
    release_recommendation: &'static str,
    capabilities: [ProbeCapability; 15],
    known_limitations: [&'static str; 4],
    resource_figures: &'static str,
    synthetic_config_transactions: &'static str,
}

impl ProviderProbeReport {
    fn read_only_inventory(provider: ProbeProvider) -> Self {
        Self {
            report_version: 1,
            provider,
            tested_version: "not_collected",
            environment_category: "not_collected",
            integration_mode: "read_only_inventory",
            release_recommendation: "not_probed",
            capabilities: ProbeCapability::ALL,
            known_limitations: [
                "live_probe_not_run",
                "install_rollback_not_run",
                "late_attach_not_run",
                "resource_measurement_not_run",
            ],
            resource_figures: "not_measured",
            synthetic_config_transactions: "fixture_only",
        }
    }

    fn diagnostics(self) -> String {
        let mut lines = vec![
            format!("report_version={}", self.report_version),
            format!("provider={}", self.provider.id()),
            format!("probe_card={}", self.provider.probe_card()),
            format!("tested_version={}", self.tested_version),
            format!("environment_category={}", self.environment_category),
            format!("integration_mode={}", self.integration_mode),
            format!("release_recommendation={}", self.release_recommendation),
            format!("known_limitations={}", self.known_limitations.join(",")),
            format!("resource_figures={}", self.resource_figures),
            format!(
                "synthetic_config_transactions={}",
                self.synthetic_config_transactions
            ),
            "raw_provider_content=false".to_owned(),
            "raw_provider_configuration=false".to_owned(),
        ];
        lines.extend(
            self.capabilities
                .iter()
                .copied()
                .map(|capability| format!("capability={} result=not_probed", capability.id())),
        );
        lines.join("\n")
    }
}

fn provider_probe_report_diagnostics(provider: ProbeProvider) -> String {
    ProviderProbeReport::read_only_inventory(provider).diagnostics()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SyntheticConfigTransactionFixture {
    fixture_version: u8,
    provider: ProbeProvider,
    fixture_scope: &'static str,
    real_provider_config_read: bool,
    real_provider_config_written: bool,
    unrelated_entries_preserved: bool,
    ordering_preserved: bool,
    pulse_signature_only: bool,
}

impl SyntheticConfigTransactionFixture {
    fn passing(provider: ProbeProvider) -> Self {
        Self {
            fixture_version: 1,
            provider,
            fixture_scope: "synthetic_user_config",
            real_provider_config_read: false,
            real_provider_config_written: false,
            unrelated_entries_preserved: true,
            ordering_preserved: true,
            pulse_signature_only: true,
        }
    }

    fn diagnostics(self) -> String {
        [
            format!("fixture_version={}", self.fixture_version),
            format!("provider={}", self.provider.id()),
            format!("fixture_scope={}", self.fixture_scope),
            format!(
                "real_provider_config_read={}",
                self.real_provider_config_read
            ),
            format!(
                "real_provider_config_written={}",
                self.real_provider_config_written
            ),
            "install_pulse_entry=pass".to_owned(),
            "update_pulse_entry=pass".to_owned(),
            "uninstall_pulse_entry=pass".to_owned(),
            format!(
                "unrelated_entries_preserved={}",
                self.unrelated_entries_preserved
            ),
            format!("ordering_preserved={}", self.ordering_preserved),
            format!("pulse_signature_only={}", self.pulse_signature_only),
            "rollback_after_interrupted_install=pass".to_owned(),
        ]
        .join("\n")
    }
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_config_transaction_fixture_diagnostics(provider: ProbeProvider) -> String {
    SyntheticConfigTransactionFixture::passing(provider).diagnostics()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProviderProbeScorecard {
    scorecard_version: u8,
    providers: [ProbeProvider; 3],
    hard_disqualifiers: [&'static str; 5],
}

impl ProviderProbeScorecard {
    fn empty() -> Self {
        Self {
            scorecard_version: 1,
            providers: [
                ProbeProvider::CodexCli,
                ProbeProvider::ClaudeCode,
                ProbeProvider::Antigravity,
            ],
            hard_disqualifiers: [
                "user_level_integration_unproven",
                "late_attach_unproven",
                "terminal_truth_unproven",
                "privacy_boundary_unproven",
                "resource_budget_unproven",
            ],
        }
    }

    fn diagnostics(self) -> String {
        let mut lines = vec![
            format!("scorecard_version={}", self.scorecard_version),
            "selection_status=no_adapter_selected".to_owned(),
            "selection_reason=no_provider_has_probe_evidence".to_owned(),
            format!("hard_disqualifiers={}", self.hard_disqualifiers.join(",")),
            "first_adapter_candidate=none".to_owned(),
        ];
        lines.extend(self.providers.iter().copied().map(|provider| {
            format!(
                "provider={} total_score=0 release=not_probed",
                provider.id()
            )
        }));
        lines.join("\n")
    }
}

fn provider_probe_scorecard_diagnostics() -> String {
    ProviderProbeScorecard::empty().diagnostics()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceMeasurementCategory {
    DropModeMemoryCpu,
    ActiveLinkMemoryCpu,
    EventToSnapshotLatency,
    AdapterEventRate,
    BreadcrumbSize,
    LinkExitBehavior,
}

impl ResourceMeasurementCategory {
    fn id(self) -> &'static str {
        match self {
            Self::DropModeMemoryCpu => "drop_mode_memory_cpu",
            Self::ActiveLinkMemoryCpu => "active_link_memory_cpu",
            Self::EventToSnapshotLatency => "event_to_snapshot_latency",
            Self::AdapterEventRate => "adapter_event_rate",
            Self::BreadcrumbSize => "breadcrumb_size",
            Self::LinkExitBehavior => "link_exit_behavior",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProviderResourceFixture {
    fixture_version: u8,
    provider: ProbeProvider,
    categories: [ResourceMeasurementCategory; 6],
}

impl ProviderResourceFixture {
    fn synthetic(provider: ProbeProvider) -> Self {
        Self {
            fixture_version: 1,
            provider,
            categories: [
                ResourceMeasurementCategory::DropModeMemoryCpu,
                ResourceMeasurementCategory::ActiveLinkMemoryCpu,
                ResourceMeasurementCategory::EventToSnapshotLatency,
                ResourceMeasurementCategory::AdapterEventRate,
                ResourceMeasurementCategory::BreadcrumbSize,
                ResourceMeasurementCategory::LinkExitBehavior,
            ],
        }
    }

    fn diagnostics(self) -> String {
        let mut lines = vec![
            format!("resource_fixture_version={}", self.fixture_version),
            format!("provider={}", self.provider.id()),
            "fixture_scope=synthetic_measurement_categories".to_owned(),
            "live_provider_probe=false".to_owned(),
            "raw_provider_content=false".to_owned(),
            "raw_provider_configuration=false".to_owned(),
            "resource_budget_claim=not_measured".to_owned(),
        ];
        lines.extend(
            self.categories
                .iter()
                .copied()
                .map(|category| format!("measurement={} status=category_only", category.id())),
        );
        lines.join("\n")
    }
}

fn provider_resource_fixture_diagnostics(provider: ProbeProvider) -> String {
    ProviderResourceFixture::synthetic(provider).diagnostics()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvidenceCategory {
    OfficialSurfaceInventory,
    SyntheticFixture,
}

impl EvidenceCategory {
    fn id(self) -> &'static str {
        match self {
            Self::OfficialSurfaceInventory => "official_surface_inventory",
            Self::SyntheticFixture => "synthetic_fixture",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SanitizedEvidenceEntry {
    id: &'static str,
    category: EvidenceCategory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProviderEvidenceRegister {
    register_version: u8,
    provider: ProbeProvider,
    entries: [SanitizedEvidenceEntry; 3],
}

impl ProviderEvidenceRegister {
    fn sanitized_summary(provider: ProbeProvider) -> Self {
        Self {
            register_version: 1,
            provider,
            entries: [
                SanitizedEvidenceEntry {
                    id: "official_hooks",
                    category: EvidenceCategory::OfficialSurfaceInventory,
                },
                SanitizedEvidenceEntry {
                    id: "app_server",
                    category: EvidenceCategory::OfficialSurfaceInventory,
                },
                SanitizedEvidenceEntry {
                    id: "install_rollback",
                    category: EvidenceCategory::SyntheticFixture,
                },
            ],
        }
    }

    fn diagnostics(self) -> String {
        let mut lines = vec![
            format!("evidence_register_version={}", self.register_version),
            format!("provider={}", self.provider.id()),
            "register_scope=sanitized_probe_summary".to_owned(),
            "raw_provider_content=false".to_owned(),
            "raw_provider_configuration=false".to_owned(),
            "raw_source_location=false".to_owned(),
            "capability_claims_enabled=false".to_owned(),
            "release_recommendation=not_probed".to_owned(),
        ];
        lines.extend(self.entries.iter().map(|entry| {
            format!(
                "evidence={} category={} status=summary_only",
                entry.id,
                entry.category.id()
            )
        }));
        lines.join("\n")
    }
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_evidence_register_diagnostics(provider: ProbeProvider) -> String {
    ProviderEvidenceRegister::sanitized_summary(provider).diagnostics()
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_official_evidence_source_locator_diagnostics(provider: ProbeProvider) -> String {
    [
        "official_evidence_source_locator_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "locator_status=scaffold_only".to_owned(),
        "source_location_retained=false".to_owned(),
        "raw_documentation_retained=false".to_owned(),
        "required_field=source_type".to_owned(),
        "required_field=source_location_redacted_id".to_owned(),
        "required_field=published_or_updated_date".to_owned(),
        "required_field=provider_version_tested".to_owned(),
        "required_field=capability_claim_supported".to_owned(),
        "required_field=known_constraints".to_owned(),
        "source_candidate=hook_reference type=official_documentation".to_owned(),
        "source_candidate=local_api_reference type=official_documentation".to_owned(),
        "source_candidate=cli_behavior type=verified_official_cli_behavior".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "release_recommendation=not_probed".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCapabilityResult {
    NotProbed,
    ProbedCandidate,
}

impl ProbeCapabilityResult {
    fn id(self) -> &'static str {
        match self {
            Self::NotProbed => "not_probed",
            Self::ProbedCandidate => "probed_candidate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SanitizedProbeCapabilitySummary {
    capability: ProbeCapability,
    result: ProbeCapabilityResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SanitizedProbeSummaryFixture {
    fixture_version: u8,
    provider: ProbeProvider,
    capabilities: [SanitizedProbeCapabilitySummary; 5],
}

impl SanitizedProbeSummaryFixture {
    fn for_provider(provider: ProbeProvider) -> Self {
        Self {
            fixture_version: 1,
            provider,
            capabilities: [
                SanitizedProbeCapabilitySummary {
                    capability: ProbeCapability::DiscoverSession,
                    result: ProbeCapabilityResult::ProbedCandidate,
                },
                SanitizedProbeCapabilitySummary {
                    capability: ProbeCapability::ObserveWaiting,
                    result: ProbeCapabilityResult::ProbedCandidate,
                },
                SanitizedProbeCapabilitySummary {
                    capability: ProbeCapability::OpenWorkspace,
                    result: ProbeCapabilityResult::ProbedCandidate,
                },
                SanitizedProbeCapabilitySummary {
                    capability: ProbeCapability::OpenExactContext,
                    result: ProbeCapabilityResult::NotProbed,
                },
                SanitizedProbeCapabilitySummary {
                    capability: ProbeCapability::ObserveQuotaSnapshot,
                    result: ProbeCapabilityResult::NotProbed,
                },
            ],
        }
    }

    fn diagnostics(self) -> String {
        let mut lines = vec![
            format!("summary_fixture_version={}", self.fixture_version),
            format!("provider={}", self.provider.id()),
            "summary_scope=sanitized_probe_result".to_owned(),
            "raw_provider_content=false".to_owned(),
            "raw_provider_configuration=false".to_owned(),
            "raw_payload_retained=false".to_owned(),
            "release_recommendation=not_probed".to_owned(),
            "w5_adapter_creation_authorized=false".to_owned(),
        ];
        lines.extend(self.capabilities.iter().map(|summary| {
            format!(
                "capability={} result={}",
                summary.capability.id(),
                summary.result.id()
            )
        }));
        lines.join("\n")
    }
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_probe_summary_fixture_diagnostics(provider: ProbeProvider) -> String {
    SanitizedProbeSummaryFixture::for_provider(provider).diagnostics()
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_probe_scorecard_fixture_diagnostics() -> String {
    [
        "scorecard_version=1".to_owned(),
        "score_source=sanitized_fixture".to_owned(),
        "provider=codex_cli total_score=7 release=not_probed".to_owned(),
        "provider=claude_code total_score=8 release=not_probed".to_owned(),
        "provider=antigravity total_score=1 release=not_probed".to_owned(),
        "dimension=user_level_ingress codex_cli=2 claude_code=2 antigravity=0".to_owned(),
        "dimension=waiting_truth codex_cli=2 claude_code=3 antigravity=0".to_owned(),
        "selection_status=no_adapter_selected".to_owned(),
        "selection_reason=observe_release_not_earned".to_owned(),
        "first_adapter_candidate=none".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_release_label_evaluation_fixture_diagnostics() -> String {
    [
        "release_label_evaluation_version=1".to_owned(),
        "source=sanitized_fixture".to_owned(),
        "current_release_label=not_probed".to_owned(),
        "evaluation_policy=direct_gate_required".to_owned(),
        "label=process_observed eligible=false reason=direct_process_evidence_missing".to_owned(),
        "label=experimental_attached eligible=false reason=install_late_attach_or_truth_gate_missing"
            .to_owned(),
        "label=supported_observe eligible=false reason=observe_gate_missing".to_owned(),
        "label=supported_fuel eligible=false reason=fuel_source_gate_missing".to_owned(),
        "label=supported_control eligible=false reason=control_safety_review_missing".to_owned(),
        "blocking_gate=official_evidence_register".to_owned(),
        "blocking_gate=install_update_uninstall_real_fixture".to_owned(),
        "blocking_gate=live_lifecycle_mapping".to_owned(),
        "blocking_gate=late_attach_real_result".to_owned(),
        "blocking_gate=terminal_truth_real_result".to_owned(),
        "blocking_gate=live_resource_measurement".to_owned(),
        "release_recommendation=not_probed".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_probe_phase_status_diagnostics(provider: ProbeProvider) -> String {
    [
        "probe_phase_status_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "protocol=docs/pulse-island/15-provider-capability-probe.md".to_owned(),
        "phase=p0_official_surface_inventory status=scaffolded execution=read_only".to_owned(),
        "phase=p1_passive_process_discovery status=not_executed execution=requires_authorization"
            .to_owned(),
        "phase=p2_installation_rollback status=not_executed execution=requires_authorization"
            .to_owned(),
        "phase=p3_lifecycle_semantics status=not_executed execution=requires_authorization"
            .to_owned(),
        "phase=p4_late_attach status=not_executed execution=requires_authorization".to_owned(),
        "phase=p5_context_routing status=not_executed execution=requires_authorization".to_owned(),
        "phase=p6_fuel_telemetry status=not_executed execution=requires_authorization".to_owned(),
        "phase=p7_fail_open_fault_injection status=not_executed execution=requires_authorization"
            .to_owned(),
        "phase=p8_performance_retention status=not_executed execution=requires_authorization"
            .to_owned(),
        "live_provider_probe=false".to_owned(),
        "provider_task_started=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "raw_provider_content=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_hard_disqualifier_fixture_diagnostics() -> String {
    [
        "hard_disqualifier_version=1".to_owned(),
        "source=sanitized_fixture".to_owned(),
        "provider=codex_cli".to_owned(),
        "provider=claude_code".to_owned(),
        "provider=antigravity".to_owned(),
        "gate=user_level_install_rollback passed=false".to_owned(),
        "gate=late_attach passed=false".to_owned(),
        "gate=terminal_truth passed=false".to_owned(),
        "gate=privacy_boundary passed=true".to_owned(),
        "gate=resource_budget passed=false".to_owned(),
        "w5_blocked_by_hard_disqualifier=true".to_owned(),
        "adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_evidence_gap_summary_diagnostics() -> String {
    let providers = [
        ProbeProvider::CodexCli,
        ProbeProvider::ClaudeCode,
        ProbeProvider::Antigravity,
    ];
    let gates = provider_direct_gates();
    let provider_ids = providers
        .iter()
        .copied()
        .map(ProbeProvider::id)
        .collect::<Vec<_>>()
        .join(",");
    let mut lines = vec![
        "evidence_gap_summary_version=1".to_owned(),
        format!("provider_count={}", providers.len()),
        format!("direct_gate_count={}", gates.len()),
        format!(
            "total_missing_direct_gates={}",
            providers.len() * gates.len()
        ),
    ];
    lines.extend(providers.iter().copied().map(|provider| {
        format!(
            "provider={} missing_direct_gates={}",
            provider.id(),
            gates.len()
        )
    }));
    lines.extend(
        gates
            .iter()
            .copied()
            .map(|gate| format!("missing_gate={} providers={}", gate.id(), provider_ids)),
    );
    lines.extend([
        "w4_complete=false".to_owned(),
        "w5_start_allowed=false".to_owned(),
        "next_allowed_work=collect_authorized_direct_gate_evidence".to_owned(),
        "live_provider_probe=false".to_owned(),
        "provider_task_started=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
    ]);
    lines.join("\n")
}

fn provider_w5_start_preflight_diagnostics() -> String {
    let providers = [
        ProbeProvider::CodexCli,
        ProbeProvider::ClaudeCode,
        ProbeProvider::Antigravity,
    ];
    let missing_direct_gate_count = providers.len() * provider_direct_gates().len();
    let mut lines = vec![
        "w5_start_preflight_version=1".to_owned(),
        "w4_complete=false".to_owned(),
        "w5_start_allowed=false".to_owned(),
        "selected_provider=none".to_owned(),
        "required_release_label=supported_observe".to_owned(),
    ];
    lines.extend(providers.iter().copied().map(|provider| {
        format!(
            "provider={} eligible=false reason=missing_direct_gates",
            provider.id()
        )
    }));
    lines.extend([
        "blocking_condition=no_provider_has_supported_observe".to_owned(),
        format!("blocking_condition=total_missing_direct_gates:{missing_direct_gate_count}"),
        "blocking_condition=hard_disqualifiers_present".to_owned(),
        "blocked_next_work=provider_adapter_creation".to_owned(),
        "allowed_next_work=collect_authorized_direct_gate_evidence".to_owned(),
        "live_provider_probe=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

fn provider_probe_audit_diagnostics() -> String {
    [
        "w4_probe_audit_version=1".to_owned(),
        "active_work=W4_Provider_Probe_Harness".to_owned(),
        "manifest=present".to_owned(),
        "provider_reports=present".to_owned(),
        "synthetic_config_transactions=present".to_owned(),
        "resource_fixtures=present".to_owned(),
        "evidence_registers=sanitized_only".to_owned(),
        "official_evidence_source_locators=scaffold_only".to_owned(),
        "probe_summaries=sanitized_fixture_only".to_owned(),
        "capability_matrix=present".to_owned(),
        "scorecard=sanitized_fixture_only".to_owned(),
        "hard_disqualifiers=blocking".to_owned(),
        "evidence_gap_summary=present".to_owned(),
        "live_provider_probe=false".to_owned(),
        "read_only_local_probe_run=present".to_owned(),
        "read_only_local_probe_summary=present".to_owned(),
        "read_only_resource_measurement_plan=present".to_owned(),
        "probe_card_execution_plans=present".to_owned(),
        "evidence_retention_policy=present".to_owned(),
        "live_authorization_preflight=blocking".to_owned(),
        "missing_capability_rationale=present".to_owned(),
        "release_decision_logs=present".to_owned(),
        "direct_gate_packets=present".to_owned(),
        "direct_evidence_import_checklist=present".to_owned(),
        "authorized_evidence_runbooks=present".to_owned(),
        "sanitized_evidence_output_template=present".to_owned(),
        "sanitized_evidence_bundle_validator=present".to_owned(),
        "release_elevation_preflight=blocking".to_owned(),
        "w5_observe_adapter_contract=scaffold_only".to_owned(),
        "release_label_evaluation=blocking".to_owned(),
        "probe_phase_status=present".to_owned(),
        "w5_start_preflight=blocking".to_owned(),
        "w4_completion_gate=blocking".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
        "next_work=direct_gate_evidence_when_authorized".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_surface_inventory_diagnostics() -> String {
    [
        "surface_inventory_version=1".to_owned(),
        "source=probe_cards".to_owned(),
        "verification_status=card_declared_only".to_owned(),
        "provider=codex_cli surface=command_hooks status=candidate".to_owned(),
        "provider=codex_cli surface=app_server status=candidate".to_owned(),
        "provider=codex_cli surface=passive_process status=fallback_candidate".to_owned(),
        "provider=claude_code surface=command_hooks status=candidate".to_owned(),
        "provider=claude_code surface=user_settings status=candidate".to_owned(),
        "provider=claude_code surface=passive_process status=fallback_candidate".to_owned(),
        "provider=antigravity surface=passive_process status=fallback_candidate".to_owned(),
        "provider=antigravity surface=formal_integration status=unverified".to_owned(),
        "live_provider_probe=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_probe_readiness_diagnostics() -> String {
    [
        "probe_readiness_version=1".to_owned(),
        "w4_scaffold_ready=true".to_owned(),
        "manifest_ready=true".to_owned(),
        "surface_inventory_ready=true".to_owned(),
        "provider_reports_ready=true".to_owned(),
        "evidence_register_ready=true".to_owned(),
        "official_evidence_source_locator_ready=true".to_owned(),
        "scorecard_ready=true".to_owned(),
        "hard_disqualifier_ready=true".to_owned(),
        "evidence_gap_summary_ready=true".to_owned(),
        "probe_card_execution_plan_ready=true".to_owned(),
        "evidence_retention_policy_ready=true".to_owned(),
        "live_authorization_preflight_ready=true".to_owned(),
        "missing_capability_rationale_ready=true".to_owned(),
        "release_decision_log_ready=true".to_owned(),
        "capability_matrix_ready=true".to_owned(),
        "release_label_evaluation_ready=true".to_owned(),
        "probe_phase_status_ready=true".to_owned(),
        "w5_start_preflight_ready=true".to_owned(),
        "direct_gate_packet_ready=true".to_owned(),
        "direct_evidence_import_checklist_ready=true".to_owned(),
        "authorized_evidence_runbook_ready=true".to_owned(),
        "sanitized_evidence_output_template_ready=true".to_owned(),
        "sanitized_evidence_bundle_validator_ready=true".to_owned(),
        "release_elevation_preflight_ready=true".to_owned(),
        "w5_observe_adapter_contract_ready=true".to_owned(),
        "w4_completion_gate_ready=true".to_owned(),
        "live_probe_ready=false".to_owned(),
        "w5_ready=false".to_owned(),
        "remaining_gate=live_provider_probe_execution".to_owned(),
        "remaining_gate=live_resource_measurement".to_owned(),
        "remaining_gate=install_rollback_real_fixture".to_owned(),
        "remaining_gate=late_attach_real_result".to_owned(),
        "remaining_gate=terminal_truth_real_result".to_owned(),
        "next_allowed_work=collect_direct_gate_evidence_when_authorized".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_local_environment_manifest_diagnostics() -> String {
    [
        "local_environment_manifest_version=1".to_owned(),
        "source=read_only_cli_preflight".to_owned(),
        "provider=codex_cli command_present=true version_status=observed".to_owned(),
        "provider=claude_code command_present=true version_status=observed".to_owned(),
        "provider=antigravity command_present=false version_status=not_found".to_owned(),
        "raw_command_path_retained=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "account_data_read=false".to_owned(),
        "network_query=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "release_recommendation=not_probed".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_live_probe_dry_run_diagnostics() -> String {
    [
        "live_probe_dry_run_version=1".to_owned(),
        "source=sanitized_environment_manifest".to_owned(),
        "executed=false".to_owned(),
        "action=collect_version_category mode=read_only".to_owned(),
        "action=collect_environment_category mode=read_only".to_owned(),
        "action=confirm_probe_card_surface_inventory mode=read_only".to_owned(),
        "action=prepare_sanitized_evidence_summary mode=read_only".to_owned(),
        "action=prepare_resource_measurement_plan mode=read_only".to_owned(),
        "forbidden_action=install_hook".to_owned(),
        "forbidden_action=mutate_provider_config".to_owned(),
        "forbidden_action=start_provider_task".to_owned(),
        "forbidden_action=query_network_or_app_server".to_owned(),
        "forbidden_action=parse_transcript_or_session_file".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadOnlyVersionCategory {
    Observed,
    NotFound,
    Failed,
}

impl ReadOnlyVersionCategory {
    fn id(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::NotFound => "not_found",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadOnlyLocalProbeTarget {
    provider: ProbeProvider,
    command_name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadOnlyLocalProbeObservation {
    target: ReadOnlyLocalProbeTarget,
    version_category: ReadOnlyVersionCategory,
}

impl ReadOnlyLocalProbeObservation {
    fn collect(target: ReadOnlyLocalProbeTarget) -> Self {
        let version_category = match probe_command_output(target.command_name) {
            Ok(output) if output.status.success() && has_version_bytes(&output) => {
                ReadOnlyVersionCategory::Observed
            }
            Ok(_) => ReadOnlyVersionCategory::Failed,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ReadOnlyVersionCategory::NotFound
            }
            Err(_) => ReadOnlyVersionCategory::Failed,
        };

        Self {
            target,
            version_category,
        }
    }

    fn diagnostic_line(self) -> String {
        format!(
            "provider={} command_name={} version_category={} environment_category=local_cli_preflight",
            self.target.provider.id(),
            self.target.command_name,
            self.version_category.id()
        )
    }
}

fn probe_command_output(command_name: &str) -> std::io::Result<std::process::Output> {
    let candidates: &[&str] = if cfg!(windows) {
        match command_name {
            "codex" => &["codex", "codex.cmd", "codex.exe"],
            "claude" => &["claude", "claude.cmd", "claude.exe"],
            "antigravity" => &["antigravity", "antigravity.cmd", "antigravity.exe"],
            _ => &[command_name],
        }
    } else {
        &[command_name]
    };
    let mut last_error = None;
    for candidate in candidates {
        match std::process::Command::new(candidate)
            .arg("--version")
            .output()
        {
            Ok(output) => return Ok(output),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => last_error = Some(error),
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound)))
}

fn has_version_bytes(output: &std::process::Output) -> bool {
    !output.stdout.is_empty() || !output.stderr.is_empty()
}

fn read_only_local_probe_targets() -> [ReadOnlyLocalProbeTarget; 3] {
    [
        ReadOnlyLocalProbeTarget {
            provider: ProbeProvider::CodexCli,
            command_name: "codex",
        },
        ReadOnlyLocalProbeTarget {
            provider: ProbeProvider::ClaudeCode,
            command_name: "claude",
        },
        ReadOnlyLocalProbeTarget {
            provider: ProbeProvider::Antigravity,
            command_name: "antigravity",
        },
    ]
}

fn provider_live_probe_run_diagnostics() -> String {
    let mut lines = vec![
        "live_probe_run_version=1".to_owned(),
        "mode=read_only_local".to_owned(),
        "source=sanitized_environment_manifest".to_owned(),
        "executed=true".to_owned(),
        "action=collect_version_category mode=read_only".to_owned(),
    ];
    lines.extend(
        read_only_local_probe_targets()
            .into_iter()
            .map(ReadOnlyLocalProbeObservation::collect)
            .map(ReadOnlyLocalProbeObservation::diagnostic_line),
    );
    lines.push(format!(
        "provider=codex_cli surface=exec_help status={}",
        if probe_command_output_with_args("codex", &["exec", "--help"]) {
            "observed"
        } else {
            "not_observed"
        }
    ));
    lines.push(format!(
        "provider=codex_cli surface=app_server_help status={}",
        if probe_command_output_with_args("codex", &["app-server", "--help"]) {
            "observed"
        } else {
            "not_observed"
        }
    ));
    lines.extend([
        "raw_version_retained=false".to_owned(),
        "raw_command_path_retained=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "network_query=false".to_owned(),
        "provider_task_started=false".to_owned(),
        "transcript_or_session_file_parsed=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "release_recommendation=not_probed".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

fn probe_command_output_with_args(command_name: &str, args: &[&str]) -> bool {
    let candidates: &[&str] = if cfg!(windows) && command_name == "codex" {
        &["codex", "codex.cmd", "codex.exe"]
    } else {
        &[command_name]
    };
    candidates.iter().any(|candidate| {
        std::process::Command::new(candidate)
            .args(args)
            .output()
            .is_ok_and(|output| output.status.success())
    })
}

fn provider_live_probe_summary_diagnostics() -> String {
    let mut lines = vec![
        "live_probe_summary_version=1".to_owned(),
        "summary_scope=sanitized_read_only_local_probe".to_owned(),
        "source=read_only_local_probe_run".to_owned(),
    ];
    lines.extend(
        read_only_local_probe_targets()
            .into_iter()
            .map(ReadOnlyLocalProbeObservation::collect)
            .flat_map(|observation| {
                [
                    format!(
                        "provider={} evidence=version_category status={}",
                        observation.target.provider.id(),
                        observation.version_category.id()
                    ),
                    format!(
                        "provider={} evidence=environment_category status=local_cli_preflight",
                        observation.target.provider.id()
                    ),
                ]
            }),
    );
    lines.extend([
        "raw_version_retained=false".to_owned(),
        "raw_command_path_retained=false".to_owned(),
        "raw_provider_content=false".to_owned(),
        "raw_provider_configuration=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "network_query=false".to_owned(),
        "provider_task_started=false".to_owned(),
        "transcript_or_session_file_parsed=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "release_recommendation=not_probed".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_resource_measurement_plan_diagnostics() -> String {
    let mut lines = vec![
        "resource_measurement_plan_version=1".to_owned(),
        "source=sanitized_read_only_local_probe_summary".to_owned(),
        "executed=false".to_owned(),
    ];
    lines.extend(
        ProviderResourceFixture::synthetic(ProbeProvider::CodexCli)
            .categories
            .iter()
            .copied()
            .map(|category| format!("measurement={} mode=planned_read_only", category.id())),
    );
    lines.extend([
        "provider_task_started=false".to_owned(),
        "network_query=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "raw_provider_content=false".to_owned(),
        "raw_provider_configuration=false".to_owned(),
        "resource_budget_claim=not_measured".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProviderDirectGate {
    OfficialEvidenceRegister,
    InstallUpdateUninstallRealFixture,
    LiveLifecycleMapping,
    LateAttachRealResult,
    ContextRouteRealResult,
    FaultPrivacyRealResult,
    LiveResourceMeasurement,
}

impl ProviderDirectGate {
    fn id(self) -> &'static str {
        match self {
            Self::OfficialEvidenceRegister => "official_evidence_register",
            Self::InstallUpdateUninstallRealFixture => "install_update_uninstall_real_fixture",
            Self::LiveLifecycleMapping => "live_lifecycle_mapping",
            Self::LateAttachRealResult => "late_attach_real_result",
            Self::ContextRouteRealResult => "context_route_real_result",
            Self::FaultPrivacyRealResult => "fault_privacy_real_result",
            Self::LiveResourceMeasurement => "live_resource_measurement",
        }
    }
}

fn provider_direct_gates() -> [ProviderDirectGate; 7] {
    [
        ProviderDirectGate::OfficialEvidenceRegister,
        ProviderDirectGate::InstallUpdateUninstallRealFixture,
        ProviderDirectGate::LiveLifecycleMapping,
        ProviderDirectGate::LateAttachRealResult,
        ProviderDirectGate::ContextRouteRealResult,
        ProviderDirectGate::FaultPrivacyRealResult,
        ProviderDirectGate::LiveResourceMeasurement,
    ]
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_direct_gate_packet_diagnostics(provider: ProbeProvider) -> String {
    let mut lines = vec![
        "direct_gate_packet_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "scope=w4_direct_gate_evidence_packet".to_owned(),
        "execution_status=not_executed".to_owned(),
        "requires_explicit_authorization=true".to_owned(),
    ];
    lines.extend(
        provider_direct_gates()
            .iter()
            .copied()
            .map(|gate| format!("gate={} status=missing", gate.id())),
    );
    lines.extend([
        "raw_provider_content=false".to_owned(),
        "raw_provider_configuration=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
        "forbidden_without_authorization=install_live_hook".to_owned(),
        "forbidden_without_authorization=mutate_provider_config".to_owned(),
        "forbidden_without_authorization=start_provider_task".to_owned(),
        "forbidden_without_authorization=query_network_or_app_server".to_owned(),
        "forbidden_without_authorization=parse_transcript_or_session_file".to_owned(),
    ]);
    lines.join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_direct_evidence_import_checklist_diagnostics(provider: ProbeProvider) -> String {
    let mut lines = vec![
        "direct_evidence_import_checklist_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "source=authorized_local_direct_evidence_only".to_owned(),
        "import_execution=false".to_owned(),
    ];
    lines.extend(
        provider_direct_gates()
            .iter()
            .copied()
            .map(|gate| format!("gate={} requires_authorized_artifact=true", gate.id())),
    );
    lines.extend([
        "reject_if=sanitized_fixture_only".to_owned(),
        "reject_if=read_only_version_category_only".to_owned(),
        "reject_if=raw_provider_content_present".to_owned(),
        "reject_if=raw_provider_configuration_present".to_owned(),
        "release_elevation_allowed=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_authorized_evidence_runbook_diagnostics(provider: ProbeProvider) -> String {
    let mut lines = vec![
        "authorized_evidence_runbook_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "runbook_status=scaffold_only".to_owned(),
        "requires_explicit_authorization=true".to_owned(),
        "local_raw_evidence_retention=local_only".to_owned(),
        "repo_output=sanitized_artifacts_only".to_owned(),
        "step=prepare_synthetic_workspace action=manual_authorized".to_owned(),
        "step=backup_user_level_provider_config action=local_only".to_owned(),
        "step=execute_probe_card_phases action=manual_authorized".to_owned(),
        "step=collect_direct_gate_artifacts action=manual_authorized".to_owned(),
        "step=redact_to_sanitized_outputs action=required_before_repo".to_owned(),
        "step=run_release_elevation_preflight action=blocking_check".to_owned(),
    ];
    lines.extend(provider_direct_gates().iter().copied().map(|gate| {
        let output = match gate {
            ProviderDirectGate::OfficialEvidenceRegister => "sanitized_summary",
            ProviderDirectGate::InstallUpdateUninstallRealFixture => "category_result",
            ProviderDirectGate::LiveLifecycleMapping => "sanitized_event_mapping",
            ProviderDirectGate::LateAttachRealResult => "category_result",
            ProviderDirectGate::ContextRouteRealResult => "route_strength_matrix",
            ProviderDirectGate::FaultPrivacyRealResult => "category_result",
            ProviderDirectGate::LiveResourceMeasurement => "category_metrics",
        };
        format!("gate={} output={output}", gate.id())
    }));
    lines.extend([
        "provider_task_started=false".to_owned(),
        "provider_config_read=false".to_owned(),
        "provider_config_written=false".to_owned(),
        "raw_provider_content=false".to_owned(),
        "raw_provider_configuration=false".to_owned(),
        "release_elevation_allowed=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_sanitized_evidence_output_template_diagnostics(provider: ProbeProvider) -> String {
    [
        "sanitized_evidence_output_template_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "source=authorized_direct_evidence_after_redaction".to_owned(),
        "repo_artifact=sanitized_probe_report required=true raw_content=false".to_owned(),
        "repo_artifact=capability_matrix required=true raw_content=false".to_owned(),
        "repo_artifact=sanitized_event_mapping_fixtures required=true raw_content=false".to_owned(),
        "repo_artifact=test_harness_category_results required=true raw_content=false".to_owned(),
        "repo_artifact=known_limitations required=true raw_content=false".to_owned(),
        "repo_artifact=release_decision required=true raw_content=false".to_owned(),
        "repo_forbidden=raw_prompts_or_transcripts".to_owned(),
        "repo_forbidden=raw_provider_configuration".to_owned(),
        "repo_forbidden=customer_project_source".to_owned(),
        "repo_forbidden=credentials_cookies_or_tokens".to_owned(),
        "release_elevation_allowed=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_sanitized_evidence_bundle_validator_diagnostics(provider: ProbeProvider) -> String {
    [
        "sanitized_evidence_bundle_validator_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "validator_status=scaffold_only".to_owned(),
        "validation_execution=false".to_owned(),
        "source=authorized_direct_evidence_after_redaction".to_owned(),
        "requires_artifact=sanitized_probe_report".to_owned(),
        "requires_artifact=capability_matrix".to_owned(),
        "requires_artifact=sanitized_event_mapping_fixtures".to_owned(),
        "requires_artifact=test_harness_category_results".to_owned(),
        "requires_artifact=known_limitations".to_owned(),
        "requires_artifact=release_decision".to_owned(),
        "reject_if=raw_prompts_or_transcripts_present".to_owned(),
        "reject_if=raw_provider_configuration_present".to_owned(),
        "reject_if=customer_project_source_present".to_owned(),
        "reject_if=credentials_cookies_or_tokens_present".to_owned(),
        "reject_if=raw_terminal_buffers_present".to_owned(),
        "reject_if=private_endpoint_traffic_present".to_owned(),
        "direct_evidence_claimed=false".to_owned(),
        "release_elevation_allowed=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_release_elevation_preflight_diagnostics(provider: ProbeProvider) -> String {
    [
        "release_elevation_preflight_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "current_release_label=not_probed".to_owned(),
        "target_release_label=supported_observe".to_owned(),
        "preflight_status=blocked".to_owned(),
        "requirement=direct_gate_packet status=ready".to_owned(),
        "requirement=direct_gate_evidence status=missing".to_owned(),
        "requirement=sanitized_output_template status=ready".to_owned(),
        "requirement=hard_disqualifiers_clear status=false".to_owned(),
        "release_elevation_allowed=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
        "next_allowed_work=collect_authorized_direct_gate_evidence".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_w5_observe_adapter_contract_diagnostics(provider: ProbeProvider) -> String {
    [
        "w5_observe_adapter_contract_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "contract_status=scaffold_only".to_owned(),
        "requires_release_label=supported_observe".to_owned(),
        "allowed_capability=formal_user_level_ingress".to_owned(),
        "allowed_capability=stable_session_identity_where_proved".to_owned(),
        "allowed_capability=workspace_association".to_owned(),
        "allowed_capability=running_freshness_where_proved".to_owned(),
        "allowed_capability=waiting_signal_where_proved".to_owned(),
        "allowed_capability=late_island_attach_after_link_breadcrumb".to_owned(),
        "allowed_capability=workspace_ready_route".to_owned(),
        "allowed_capability=observed_degraded_fallback".to_owned(),
        "excluded_capability=arbitrary_external_session_control".to_owned(),
        "excluded_capability=approval_or_deny_ui".to_owned(),
        "excluded_capability=transcript_or_history_parsing".to_owned(),
        "excluded_capability=task_title_from_raw_prompt".to_owned(),
        "excluded_capability=exact_route_without_exact_evidence".to_owned(),
        "excluded_capability=completion_failure_without_terminal_evidence".to_owned(),
        "excluded_capability=fuel_without_scoped_source".to_owned(),
        "provider_adapter_created=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCardExecutionPhase {
    EnvironmentManifest,
    OfficialSurfaceInventory,
    InstallUpdateUninstall,
    LiveLifecycleMapping,
    LateAttach,
    ContextRoute,
    FuelSource,
    FaultPrivacy,
    ResourceMeasurement,
    ReleaseDecision,
}

impl ProbeCardExecutionPhase {
    fn id(self) -> &'static str {
        match self {
            Self::EnvironmentManifest => "environment_manifest",
            Self::OfficialSurfaceInventory => "official_surface_inventory",
            Self::InstallUpdateUninstall => "install_update_uninstall",
            Self::LiveLifecycleMapping => "live_lifecycle_mapping",
            Self::LateAttach => "late_attach",
            Self::ContextRoute => "context_route",
            Self::FuelSource => "fuel_source",
            Self::FaultPrivacy => "fault_privacy",
            Self::ResourceMeasurement => "resource_measurement",
            Self::ReleaseDecision => "release_decision",
        }
    }
}

fn probe_card_execution_phases() -> [ProbeCardExecutionPhase; 10] {
    [
        ProbeCardExecutionPhase::EnvironmentManifest,
        ProbeCardExecutionPhase::OfficialSurfaceInventory,
        ProbeCardExecutionPhase::InstallUpdateUninstall,
        ProbeCardExecutionPhase::LiveLifecycleMapping,
        ProbeCardExecutionPhase::LateAttach,
        ProbeCardExecutionPhase::ContextRoute,
        ProbeCardExecutionPhase::FuelSource,
        ProbeCardExecutionPhase::FaultPrivacy,
        ProbeCardExecutionPhase::ResourceMeasurement,
        ProbeCardExecutionPhase::ReleaseDecision,
    ]
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_probe_card_execution_plan_diagnostics(provider: ProbeProvider) -> String {
    let mut lines = vec![
        "probe_card_execution_plan_version=1".to_owned(),
        format!("provider={}", provider.id()),
        format!("probe_card={}", provider.probe_card()),
        "execution_status=not_executed".to_owned(),
    ];
    lines.extend(
        probe_card_execution_phases()
            .iter()
            .copied()
            .map(|phase| format!("phase={} status=planned", phase.id())),
    );
    lines.extend([
        "raw_provider_content=false".to_owned(),
        "raw_provider_configuration=false".to_owned(),
        "provider_task_started=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_evidence_retention_policy_diagnostics() -> String {
    [
        "evidence_retention_policy_version=1".to_owned(),
        "repo_allowed=sanitized_probe_report".to_owned(),
        "repo_allowed=capability_matrix".to_owned(),
        "repo_allowed=sanitized_event_mapping_fixtures".to_owned(),
        "repo_allowed=test_harness_category_results".to_owned(),
        "repo_allowed=known_limitation_statements".to_owned(),
        "repo_allowed=release_decision".to_owned(),
        "repo_forbidden=customer_project_source".to_owned(),
        "repo_forbidden=full_prompts_or_transcripts".to_owned(),
        "repo_forbidden=credentials_cookies_or_tokens".to_owned(),
        "repo_forbidden=raw_terminal_buffers".to_owned(),
        "repo_forbidden=private_endpoint_traffic".to_owned(),
        "local_only=provider_documentation_exports".to_owned(),
        "local_only=synthetic_test_run_recordings".to_owned(),
        "local_only=provider_configuration_backups".to_owned(),
        "local_only=redacted_error_captures".to_owned(),
        "raw_provider_content_in_repo=false".to_owned(),
        "raw_provider_configuration_in_repo=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_live_authorization_preflight_diagnostics(provider: ProbeProvider) -> String {
    [
        "live_authorization_preflight_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "authorization_status=not_authorized".to_owned(),
        "live_actions_allowed=false".to_owned(),
        "requires_user_approved_test_workspace=true".to_owned(),
        "requires_disposable_or_authorized_account=true".to_owned(),
        "requires_local_raw_evidence_retention_policy=true".to_owned(),
        "requires_sanitized_report_destination=true".to_owned(),
        "blocked_action=install_live_hook".to_owned(),
        "blocked_action=mutate_provider_config".to_owned(),
        "blocked_action=start_provider_task".to_owned(),
        "blocked_action=query_network_or_app_server".to_owned(),
        "blocked_action=parse_transcript_or_session_file".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_missing_capability_rationale_diagnostics(provider: ProbeProvider) -> String {
    let mut lines = vec![
        "missing_capability_rationale_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "scope=w4_absent_capability_explanations".to_owned(),
    ];
    lines.extend(ProbeCapability::ALL.iter().copied().map(|capability| {
        format!(
            "capability={} release=not_probed reason={}",
            capability.id(),
            capability.missing_reason()
        )
    }));
    lines.extend([
        "blank_cells_allowed=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

fn provider_capability_matrix_diagnostics(provider: ProbeProvider) -> String {
    let mut lines = vec![
        "capability_matrix_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "matrix_template=docs/pulse-island/15-provider-capability-probe.md".to_owned(),
        "integration_mode=read_only_inventory".to_owned(),
    ];
    lines.extend(ProbeCapability::ALL.iter().copied().map(|capability| {
        format!(
            "capability={} evidence_source=missing probe_result=not_probed identity_strength=none health_ceiling=unavailable release=not_probed user_wording=not_available_yet reason={}",
            capability.id(),
            capability.missing_reason()
        )
    }));
    lines.extend([
        "blank_cells_allowed=false".to_owned(),
        "raw_provider_content=false".to_owned(),
        "raw_provider_configuration=false".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]);
    lines.join("\n")
}

#[deprecated(note = "W4 scopedown: meta-info scaffold. Use direct probe evidence instead.")]
fn provider_release_decision_log_diagnostics(provider: ProbeProvider) -> String {
    [
        "release_decision_log_version=1".to_owned(),
        format!("provider={}", provider.id()),
        "decision_scope=w4_probe_report_release_decision".to_owned(),
        "current_release_label=not_probed".to_owned(),
        "decision=defer_provider_pending_direct_evidence".to_owned(),
        "outcome_candidate=proceed_to_narrow_adapter status=blocked".to_owned(),
        "outcome_candidate=process_observed_only status=blocked".to_owned(),
        "outcome_candidate=experimental_observation status=blocked".to_owned(),
        "outcome_candidate=defer_provider status=current".to_owned(),
        "outcome_candidate=reject_integration status=not_decided".to_owned(),
        "blocking_reason=official_evidence_register_missing".to_owned(),
        "blocking_reason=install_update_uninstall_real_fixture_missing".to_owned(),
        "blocking_reason=live_lifecycle_mapping_missing".to_owned(),
        "blocking_reason=late_attach_real_result_missing".to_owned(),
        "blocking_reason=terminal_truth_real_result_missing".to_owned(),
        "decision_log_sanitized=true".to_owned(),
        "capability_claims_enabled=false".to_owned(),
        "w5_adapter_creation_authorized=false".to_owned(),
    ]
    .join("\n")
}

fn provider_w4_completion_gate_diagnostics() -> String {
    let mut lines = vec![
        "w4_completion_gate_version=1".to_owned(),
        "w4_scaffold_ready=true".to_owned(),
        "direct_gate_packets_ready=true".to_owned(),
        "w4_complete=false".to_owned(),
        "w5_start_allowed=false".to_owned(),
    ];
    lines.extend(
        provider_direct_gates()
            .iter()
            .copied()
            .map(|gate| format!("remaining_gate={}", gate.id())),
    );
    lines.push("next_allowed_work=collect_authorized_direct_gate_evidence".to_owned());
    lines.join("\n")
}

fn format_hit_test(point: &str, target: HitTarget) -> String {
    let code = Win32HitTestCode::from_target(target);
    format!(
        "point={} target={:?} win32={:?} value={}",
        point,
        target,
        code,
        code.value()
    )
}

fn measurement_policy_diagnostics() -> String {
    let policy = MeasurementPolicy::gate_a();
    let static_shell = ShellViewModel {
        signal: SignalViewModel {
            primary_task_id: None,
            state: SignalState::Idle,
            overflow_count: 0,
            primary_route_label: None,
        },
        peek: pulse_island_ui::PeekViewModel {
            rows: Vec::new(),
            hidden_count: 0,
        },
        focus_card: None,
        compact_visible: false,
        palette_visible: false,
        motion_policy: MotionPolicy::Normal,
        high_contrast: false,
    };
    let render_policy = StaticRenderPolicy::for_shell(&static_shell, MotionPolicy::Normal);
    let mut lines = vec![format!(
        "diagnostic_metadata_only={} task_content={}",
        policy.diagnostic_metadata_only, policy.includes_task_content
    )];
    lines.extend(policy.metrics.iter().map(|metric| {
        format!(
            "metric={:?} target={} comparator={:?}",
            metric.name, metric.target, metric.comparator
        )
    }));
    lines.push(format!(
        "static_render app_side_frame_loop={} redraw={:?}",
        render_policy.app_side_frame_loop_allowed, render_policy.redraw_reason
    ));
    let report = MeasurementReport::from_samples(policy, deterministic_gate_a_samples());
    lines.push(format!(
        "measurement_report passed={} missing={}",
        report.passed,
        report.missing_metrics.len()
    ));
    lines.extend(report.results.iter().map(|result| {
        format!(
            "result={:?} actual={} target={} comparator={:?} passed={}",
            result.name, result.actual, result.target, result.comparator, result.passed
        )
    }));
    lines.join("\n")
}

fn deterministic_gate_a_samples() -> Vec<MeasurementSample> {
    vec![
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::CompactIdleMemoryP95Mb,
            42.0,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::FocusCardMemoryP95Mb,
            84.0,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::ProcessTreeCeilingMb,
            99.0,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::IdleAverageCpuPercent,
            0.08,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::RunningAverageCpuPercent,
            0.30,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::StateUpdateLatencyP95Ms,
            100.0,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::PaletteShortcutLatencyP95Ms,
            75.0,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::SteadyStateMemoryGrowthMb,
            1.5,
        ),
        MeasurementSample::new(
            pulse_island_ui::MeasurementMetricName::StaticStateFrameLoop,
            0.0,
        ),
    ]
}

fn focus_policy_diagnostics() -> String {
    let passive = ShellInteractionState::default().apply(ShellUserEvent::PassivePlanUpdate);
    let compact = ShellInteractionState::default().apply(ShellUserEvent::CompactClicked);
    let palette = ShellInteractionState::default().apply(ShellUserEvent::PaletteShortcut);
    let escape = palette.clone().apply(ShellUserEvent::Escape);
    [
        format_interaction("passive", &passive),
        format_interaction("compact-click", &compact),
        format_interaction("palette-shortcut", &palette),
        format_interaction("escape", &escape),
    ]
    .join("\n")
}

fn palette_policy_diagnostics() -> String {
    let palette = CommandPaletteViewModel::p0();
    let mut lines = vec![format!("palette_commands={}", palette.commands.len())];
    lines.extend(palette.commands.iter().map(|command| {
        format!(
            "command={} provider_control={} high_risk={}",
            command.label, command.provider_control, command.high_risk
        )
    }));
    lines.join("\n")
}

fn layout_policy_diagnostics() -> String {
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
    [
        format_layout_decision(policy.available_width_px, policy.evaluate()),
        format_layout_decision(narrow.available_width_px, narrow.evaluate()),
    ]
    .join("\n")
}

fn format_layout_decision(width: u32, decision: CompactSignalLayoutDecision) -> String {
    format!(
        "layout width={} glyph={} subject={} subject_truncated={} reason={} active_count={} fuel={}",
        width,
        decision.show_state_glyph,
        decision.show_subject,
        decision.subject_truncated,
        decision.show_reason,
        decision.show_active_count,
        decision.show_secondary_fuel
    )
}

fn route_policy_diagnostics() -> Result<String, SpikeHarnessError> {
    let harness = SpikeScenarioHarness::new()?;
    let mut labels = Vec::new();
    for id in harness.scenario_ids() {
        let frame = harness.run(id)?;
        if let Some(label) = frame.shell.signal.primary_route_label {
            labels.push(label);
        }
        labels.extend(
            frame
                .shell
                .peek
                .rows
                .iter()
                .filter_map(|row| row.route_label),
        );
        if let Some(label) = frame
            .shell
            .focus_card
            .as_ref()
            .and_then(|focus| focus.route_label)
        {
            labels.push(label);
        }
    }
    labels.sort_by_key(|label| format!("{label:?}"));
    labels.dedup();
    Ok(labels
        .iter()
        .map(|label| format!("route_label={label:?}"))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn lifecycle_policy_diagnostics() -> String {
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
    [
        format!("window_generation={}", lifecycle.window_generation),
        format!(
            "active_transient_surfaces={}",
            lifecycle.active_transient_surfaces
        ),
        format!(
            "max_active_transient_surfaces={}",
            lifecycle.max_active_transient_surfaces
        ),
        format!("open_close_cycles={}", lifecycle.open_close_cycles),
    ]
    .join("\n")
}

fn format_interaction(event: &str, state: &ShellInteractionState) -> String {
    format!(
        "event={} open={:?} focus={:?}",
        event, state.open_surface, state.focus_owner
    )
}

fn accessibility_policy_diagnostics() -> String {
    let signal = SignalViewModel {
        primary_task_id: Some("task-a".to_owned()),
        state: SignalState::Waiting,
        overflow_count: 2,
        primary_route_label: None,
    };
    let accessible = AccessibleSignalViewModel::from_signal(&signal);
    let reduced = AnimationPolicy::for_state(SignalState::Waiting, MotionPolicy::Reduced);
    let contrast = VisualAccessibilityPolicy::from_environment(ShellEnvironment {
        immersive_active: false,
        reduced_motion: true,
        high_contrast: true,
    });
    let peek_keyboard = KeyboardNavigationState::new(OpenSurface::Peek, 3)
        .apply(KeyboardCommand::ArrowDown)
        .apply(KeyboardCommand::ArrowDown);
    let palette_keyboard =
        KeyboardNavigationState::new(OpenSurface::CommandPalette, 9).apply(KeyboardCommand::Enter);

    [
        format!(
            "accessible_state={} color_only={}",
            accessible.state_label, accessible.uses_color_as_sole_indicator
        ),
        format!(
            "reduced_motion pulse={} scale={} state_change={}",
            reduced.pulse_allowed, reduced.scale_allowed, reduced.state_change_visible
        ),
        format!(
            "contrast={:?} system_tokens={}",
            contrast.contrast_mode, contrast.uses_system_contrast_tokens
        ),
        format!(
            "peek_keyboard selected={} activated={:?}",
            format_index(peek_keyboard.selected_index),
            peek_keyboard.activated_index
        ),
        format!(
            "palette_keyboard selected={} activated={:?}",
            format_index(palette_keyboard.selected_index),
            palette_keyboard.activated_index
        ),
    ]
    .join("\n")
}

fn format_index(index: Option<usize>) -> String {
    match index {
        Some(index) => index.to_string(),
        None => "None".to_owned(),
    }
}

fn parse_scenario_id(value: &str) -> Result<MockScenarioId, SpikeHarnessError> {
    match value {
        "S0" => Ok(MockScenarioId::S0IdleParked),
        "S1" => Ok(MockScenarioId::S1OneRunningTask),
        "S2" => Ok(MockScenarioId::S2WaitingWithBackgroundWork),
        "S3" => Ok(MockScenarioId::S3FailedTask),
        "S4" => Ok(MockScenarioId::S4AggregateActiveFuelLow),
        "S5" => Ok(MockScenarioId::S5DegradedObserved),
        "S6" => Ok(MockScenarioId::S6CompletionSettleOut),
        "S7" => Ok(MockScenarioId::S7RapidStateChanges),
        "S8" => Ok(MockScenarioId::S8ImmersiveSimulation),
        _ => Err(SpikeHarnessError::InvalidArgument(value.to_owned())),
    }
}

fn scenario_code(id: MockScenarioId) -> &'static str {
    match id {
        MockScenarioId::S0IdleParked => "S0",
        MockScenarioId::S1OneRunningTask => "S1",
        MockScenarioId::S2WaitingWithBackgroundWork => "S2",
        MockScenarioId::S3FailedTask => "S3",
        MockScenarioId::S4AggregateActiveFuelLow => "S4",
        MockScenarioId::S5DegradedObserved => "S5",
        MockScenarioId::S6CompletionSettleOut => "S6",
        MockScenarioId::S7RapidStateChanges => "S7",
        MockScenarioId::S8ImmersiveSimulation => "S8",
    }
}

fn format_frame(frame: &ScenarioFrame) -> String {
    format!(
        "scenario={:?} frame={} state={:?} compact_visible={} peek_rows={} motion={:?} window_generation={}",
        frame.id,
        frame.frame_index,
        frame.shell.signal.state,
        frame.shell.compact_visible,
        frame.shell.peek.rows.len(),
        frame.shell.motion_policy,
        frame.window_generation
    )
}

#[cfg(test)]
mod tests {
    use pulse_island_ui::{MockScenarioId, MotionPolicy, SignalState};

    use super::*;

    #[test]
    fn scenario_harness_lists_spike_a_scenarios_in_catalog_order(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let harness = SpikeScenarioHarness::new()?;

        assert_eq!(
            harness.scenario_ids(),
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
    fn scenario_harness_runs_waiting_and_immersive_scenarios(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let harness = SpikeScenarioHarness::new()?;

        let waiting = harness.run(MockScenarioId::S2WaitingWithBackgroundWork)?;
        assert_eq!(waiting.id, MockScenarioId::S2WaitingWithBackgroundWork);
        assert_eq!(waiting.shell.signal.state, SignalState::Waiting);
        assert!(waiting.shell.compact_visible);
        assert_eq!(waiting.shell.peek.rows.len(), 2);

        let immersive = harness.run(MockScenarioId::S8ImmersiveSimulation)?;
        assert_eq!(immersive.id, MockScenarioId::S8ImmersiveSimulation);
        assert_eq!(immersive.shell.signal.state, SignalState::Running);
        assert!(!immersive.shell.compact_visible);
        assert_eq!(immersive.shell.motion_policy, MotionPolicy::Stopped);
        Ok(())
    }

    #[test]
    fn scenario_harness_replays_rapid_state_changes_without_window_recreation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let harness = SpikeScenarioHarness::new()?;

        let frames = harness.replay(MockScenarioId::S7RapidStateChanges)?;
        let states = frames
            .iter()
            .map(|frame| frame.shell.signal.state)
            .collect::<Vec<_>>();
        let window_generations = frames
            .iter()
            .map(|frame| frame.window_generation)
            .collect::<Vec<_>>();

        assert_eq!(
            states,
            vec![
                SignalState::Running,
                SignalState::Waiting,
                SignalState::Running,
                SignalState::Failed,
                SignalState::Completed,
                SignalState::Idle,
            ]
        );
        assert!(window_generations.iter().all(|generation| *generation == 1));
        Ok(())
    }

    #[test]
    fn cli_lists_scenarios_when_no_scenario_argument_is_supplied(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike"])?;

        assert!(output.contains("S0 idle / parked"));
        assert!(output.contains("S8 immersive mode simulation"));
        Ok(())
    }

    #[test]
    fn cli_runs_named_scenario_as_deterministic_snapshot() -> Result<(), Box<dyn std::error::Error>>
    {
        let output = run_cli(["pulse-island-spike", "S2"])?;

        assert!(output.contains("scenario=S2WaitingWithBackgroundWork"));
        assert!(output.contains("frame=0"));
        assert!(output.contains("state=Waiting"));
        assert!(output.contains("compact_visible=true"));
        assert!(output.contains("peek_rows=2"));
        Ok(())
    }

    #[test]
    fn cli_replays_named_scenario_without_window_recreation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "S7", "--replay"])?;

        assert!(output.contains("frame=0 state=Running"));
        assert!(output.contains("frame=5 state=Idle"));
        assert!(output
            .lines()
            .all(|line| !line.contains("window_generation=2")));
        Ok(())
    }

    #[test]
    fn cli_exports_focus_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--focus-policy"])?;

        assert!(output.contains("event=passive open=None focus=ExternalApp"));
        assert!(output.contains("event=compact-click open=Peek focus=ExternalApp"));
        assert!(output.contains("event=palette-shortcut open=CommandPalette focus=CommandPalette"));
        assert!(output.contains("event=escape open=None focus=ExternalApp"));
        Ok(())
    }

    #[test]
    fn cli_exports_palette_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--palette-policy"])?;

        assert!(output.contains("palette_commands=9"));
        assert!(output.contains("command=Open active task provider_control=false high_risk=false"));
        assert!(
            output.contains("command=Open Pulse settings provider_control=false high_risk=false")
        );
        assert!(!output.contains("Approve"));
        assert!(!output.contains("Reject"));
        assert!(!output.contains("Resume"));
        assert!(!output.contains("Stop"));
        Ok(())
    }

    #[test]
    fn cli_exports_layout_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--layout-policy"])?;

        assert!(output.contains("layout width=128 glyph=true subject=true subject_truncated=false"));
        assert!(output.contains("reason=false active_count=false fuel=false"));
        assert!(output.contains("layout width=72 glyph=true subject=true subject_truncated=true"));
        Ok(())
    }

    #[test]
    fn cli_exports_route_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--route-policy"])?;

        assert!(output.contains("route_label=OpenOriginalTask"));
        assert!(output.contains("route_label=FocusAgentWindow"));
        assert!(output.contains("route_label=OpenWorkspace"));
        assert!(output.contains("route_label=ShowProcessDetails"));
        Ok(())
    }

    #[test]
    fn cli_exports_lifecycle_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--lifecycle-policy"])?;

        assert!(output.contains("window_generation=1"));
        assert!(output.contains("active_transient_surfaces=0"));
        assert!(output.contains("max_active_transient_surfaces=1"));
        assert!(output.contains("open_close_cycles=3000"));
        Ok(())
    }

    #[test]
    fn cli_exports_accessibility_and_motion_diagnostics() -> Result<(), Box<dyn std::error::Error>>
    {
        let output = run_cli(["pulse-island-spike", "--accessibility-policy"])?;

        assert!(output.contains("accessible_state=Waiting for user"));
        assert!(output.contains("color_only=false"));
        assert!(output.contains("reduced_motion pulse=false scale=false state_change=true"));
        assert!(output.contains("contrast=HighContrast system_tokens=true"));
        assert!(output.contains("peek_keyboard selected=2 activated=None"));
        assert!(output.contains("palette_keyboard selected=0 activated=Some(0)"));
        Ok(())
    }

    #[test]
    fn cli_exports_measurement_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--measurement-policy"])?;

        assert!(output.contains("diagnostic_metadata_only=true task_content=false"));
        assert!(
            output.contains("metric=CompactIdleMemoryP95Mb target=45 comparator=LessThanOrEqual")
        );
        assert!(output.contains("metric=FocusCardMemoryP95Mb target=85 comparator=LessThanOrEqual"));
        assert!(
            output.contains("metric=IdleAverageCpuPercent target=0.1 comparator=LessThanOrEqual")
        );
        assert!(output
            .contains("metric=PaletteShortcutLatencyP95Ms target=80 comparator=LessThanOrEqual"));
        assert!(output.contains("static_render app_side_frame_loop=false redraw=None"));
        assert!(output.contains("measurement_report passed=true missing=0"));
        assert!(output.contains(
            "result=StaticStateFrameLoop actual=0 target=0 comparator=Equal passed=true"
        ));
        Ok(())
    }

    #[test]
    fn cli_exports_window_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--window-policy"])?;

        assert!(output.contains("style popup=true topmost=true toolwindow=true noactivate=true"));
        assert!(output.contains("alt_tab_visible=false permanently_click_through=false"));
        assert!(output.contains("normal visibility=Visible topmost=true replay_missed=false"));
        assert!(output.contains("fullscreen visibility=Hidden topmost=false replay_missed=false"));
        Ok(())
    }

    #[test]
    fn cli_exports_adapter_plan_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--adapter-plan-policy"])?;

        assert!(output.contains("adapter_plan visible visibility=Visible topmost=true"));
        assert!(output.contains("create=true recreate=false destroy_hidden=false activate=false"));
        assert!(output.contains("style_noactivate=true style_transparent=false"));
        assert!(output.contains("hotkey=Some(1)"));
        assert!(output.contains("adapter_plan immersive visibility=Hidden topmost=false"));
        assert!(output.contains("replay_missed=false"));
        Ok(())
    }

    #[test]
    fn cli_exports_adapter_state_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--adapter-state-policy"])?;

        assert!(output.contains("adapter_state frames=3 generation=1"));
        assert!(output.contains("create_calls=1 destroy_calls=0"));
        assert!(output.contains("hotkey_registered=true register_calls=1 unregister_calls=0"));
        assert!(output.contains("activation_attempts=0 final_visibility=Visible"));
        Ok(())
    }

    #[test]
    fn cli_exports_adapter_action_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--adapter-action-policy"])?;

        assert!(output.contains(
            "adapter_actions initial=CreateCompactWindow,ApplyWindowStyles,UpdateHitTestLayout,MoveResize,RegisterHotkey,ShowNoActivate,SetTopmost"
        ));
        assert!(output.contains("adapter_actions repeated=<none>"));
        assert!(output.contains("adapter_actions immersive=HideCompactWindow,ClearTopmost"));
        assert!(output.contains("adapter_actions hit_test_changed=UpdateHitTestLayout"));
        assert!(output.contains("adapter_actions placement_changed=MoveResize"));
        assert!(output.contains("adapter_actions style_changed=ApplyWindowStyles"));
        assert!(output.contains("adapter_actions hotkey_disabled=UnregisterHotkey"));
        Ok(())
    }

    #[test]
    fn cli_exports_adapter_replay_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--adapter-replay-policy"])?;

        assert!(output.contains("adapter_replay s7 frames=6 generation=1"));
        assert!(output.contains("create_calls=1 destroy_calls=0"));
        assert!(output.contains("hotkey_register_calls=1 hotkey_unregister_calls=0"));
        assert!(output.contains("activation_attempts=0"));
        assert!(output.contains("total_actions=9 max_actions_per_frame=7"));
        assert!(output.contains("adapter_replay s8 final_visibility=Hidden"));
        assert!(output.contains("immersive_actions=HideCompactWindow,ClearTopmost"));
        Ok(())
    }

    #[test]
    fn cli_exports_hit_test_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--hit-test-policy"])?;

        assert!(output.contains("point=margin target=Transparent win32=Transparent value=-1"));
        assert!(output.contains("point=drag target=Drag win32=Caption value=2"));
        assert!(output.contains("point=client target=Client win32=Client value=1"));
        assert!(output.contains("point=outside target=Outside win32=Nowhere value=0"));
        Ok(())
    }

    #[test]
    fn cli_exports_hotkey_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--hotkey-policy"])?;

        assert!(output.contains("hotkey enabled=true id=1 modifiers=6 virtual_key=32"));
        assert!(output.contains("normal open=CommandPalette focus=CommandPalette"));
        assert!(output.contains("immersive_allowed open=CommandPalette focus=CommandPalette"));
        assert!(output.contains("immersive_blocked open=None focus=ExternalApp"));
        Ok(())
    }

    #[test]
    fn cli_exports_dpi_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--dpi-policy"])?;

        assert!(output.contains("resolved_monitor=7 origin=(2810,1344) size=(390,96)"));
        assert!(output.contains("fallback_monitor=1 origin=(100,100)"));
        assert!(output.contains("text logical=12 dpi=144 scale=150 physical=27"));
        assert!(output.contains("text logical=1 dpi=48 scale=50 physical=1"));
        Ok(())
    }

    #[test]
    fn cli_exports_cache_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--cache-policy"])?;

        assert!(output.contains("text_layout_cache_bounded=true max=32 task_content=false"));
        assert!(output.contains(
            "invalidation=DpiChanged geometry=true text=true brushes=false unbounded=false"
        ));
        assert!(output.contains(
            "invalidation=ThemeChanged geometry=false text=false brushes=true unbounded=false"
        ));
        assert!(output.contains(
            "invalidation=FontChanged geometry=false text=true brushes=false unbounded=false"
        ));
        assert!(output.contains(
            "invalidation=StateLayoutChanged geometry=true text=true brushes=false unbounded=false"
        ));
        Ok(())
    }

    #[test]
    fn cli_exports_animation_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--animation-policy"])?;

        assert!(output.contains("animation=Arrival compositor=true frame_loop=false"));
        assert!(output.contains("animation=AttentionPulse"));
        assert!(output.contains("repetitions=Some(3) settles=true pulse=true"));
        assert!(output.contains("animation=Expansion"));
        assert!(output.contains("interruptible=true"));
        assert!(output.contains("animation=Completion"));
        Ok(())
    }

    #[test]
    fn cli_exports_resource_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--resource-policy"])?;

        assert!(output
            .contains("resource_policy shared_device=true max_devices=1 per_task_surfaces=false"));
        assert!(output.contains("virtualized_rows=true"));
        assert!(output.contains("resource_report transitions=1000 passed=true"));
        assert!(output.contains("device_growth=0 surface_growth=0 d3d_growth=0 handle_growth=0"));
        Ok(())
    }

    #[test]
    fn cli_exports_overlay_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--overlay-policy"])?;

        assert!(output.contains("overlay normal visible=false rows=0 task_content=false"));
        assert!(output.contains(
            "overlay diagnostics visible=true rows=9 metadata_only=true task_content=false"
        ));
        assert!(output.contains("overlay diagnostics passed=true"));
        Ok(())
    }

    #[test]
    fn cli_exports_truth_priority_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--truth-priority-policy"])?;

        assert!(output.contains("primary=Some(\"waiting-primary\") state=Waiting"));
        assert!(output.contains("source=PresentationPlanPrimary"));
        assert!(output.contains(
            "timer_rotation=false fuel_role=Secondary fuel_visible=true fuel_override=false"
        ));
        Ok(())
    }

    #[test]
    fn cli_exports_surface_handle_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--surface-handle-policy"])?;

        assert!(output.contains("surface_handles cycles=2000 required=2000 passed=true"));
        assert!(output.contains("surface_handles user_growth=0 gdi_growth=0"));
        Ok(())
    }

    #[test]
    fn cli_exports_architecture_policy_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--architecture-policy"])?;

        assert!(output.contains("architecture_policy checked_manifests=5 passed=true"));
        assert!(output.contains("forbidden_dependency_hits=0"));
        assert!(output.contains("mock_plan_replaceable=true"));
        assert!(output.contains("hwnd_boundary_manifest=true"));
        assert!(output.contains("link_transport_boundary_manifest=true"));
        assert!(output.contains("hwnd_native_api_adapter=true"));
        assert!(output.contains("hwnd_create_window_factory=true"));
        assert!(output.contains("hwnd_message_pump=true"));
        assert!(output.contains("hwnd_wndproc_hit_test=true"));
        assert!(output.contains("hwnd_wndproc_mouse_activate=true"));
        assert!(output.contains("hwnd_wndproc_mouse_dispatch=true"));
        assert!(output.contains("hwnd_wndproc_paint_dispatch=true"));
        assert!(output.contains("browser_runtime=false sqlite=false provider_adapter=false"));
        Ok(())
    }

    #[test]
    fn cli_exports_gate_audit_authorizing_w3() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--gate-audit"])?;

        assert!(output.contains("gate_audit checklist_items=21 evidence_items=21"));
        assert!(output.contains("functional=6/6 window=6/6 performance=5/5 architecture=4/4"));
        assert!(output.contains("w3_ready=true w2_review=accepted"));
        assert!(output.contains("w4_ready=true w3_review=accepted"));
        assert!(output.contains("active_work=W4_Provider_Probe_Harness"));
        assert!(!output.contains("active_work=W3_Link_Shim_Drop_Mode"));
        assert!(output.contains("scope=mock_presentation_plan_only"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_provider_probe_manifest_as_read_only_capability_scaffold(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-probe-manifest"])?;

        assert!(output.contains("manifest_version=1"));
        assert!(output.contains("package=W4 Provider Probe Harness"));
        assert!(output.contains("mode=read_only_capability_discovery"));
        assert!(output.contains("active_work=W4_Provider_Probe_Harness"));
        assert!(output.contains("provider=codex_cli release=not_probed"));
        assert!(output.contains("provider=claude_code release=not_probed"));
        assert!(output.contains("provider=antigravity release=not_probed"));
        assert!(output.contains("required_fields=version,environment_category,integration_mode,capability_matrix,known_limitations,resource_figures,release_recommendation"));
        assert!(output.contains("forbidden_actions=live_hook_install,provider_config_mutation,provider_adapter_creation,network_query,transcript_or_session_file_parsing,production_route_activation"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("supported_fuel"));
        assert!(!output.contains("supported_control"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_provider_probe_report_without_support_claims_or_provider_mutation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-probe-report=codex_cli"])?;

        assert!(output.contains("report_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("tested_version=not_collected"));
        assert!(output.contains("environment_category=not_collected"));
        assert!(output.contains("integration_mode=read_only_inventory"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("capability=discover_process result=not_probed"));
        assert!(output.contains("capability=observe_waiting result=not_probed"));
        assert!(output.contains("capability=open_exact_context result=not_probed"));
        assert!(output.contains("capability=observe_quota_snapshot result=not_probed"));
        assert!(output.contains("known_limitations=live_probe_not_run,install_rollback_not_run,late_attach_not_run,resource_measurement_not_run"));
        assert!(output.contains("resource_figures=not_measured"));
        assert!(output.contains("synthetic_config_transactions=fixture_only"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("supported_fuel"));
        assert!(!output.contains("supported_control"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_synthetic_config_transaction_fixture_without_real_provider_config(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-config-transaction-fixture=claude_code",
        ])?;

        assert!(output.contains("fixture_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("fixture_scope=synthetic_user_config"));
        assert!(output.contains("real_provider_config_read=false"));
        assert!(output.contains("real_provider_config_written=false"));
        assert!(output.contains("install_pulse_entry=pass"));
        assert!(output.contains("update_pulse_entry=pass"));
        assert!(output.contains("uninstall_pulse_entry=pass"));
        assert!(output.contains("unrelated_entries_preserved=true"));
        assert!(output.contains("ordering_preserved=true"));
        assert!(output.contains("pulse_signature_only=true"));
        assert!(output.contains("rollback_after_interrupted_install=pass"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_scorecard_without_selecting_adapter_before_probe_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-probe-scorecard"])?;

        assert!(output.contains("scorecard_version=1"));
        assert!(output.contains("selection_status=no_adapter_selected"));
        assert!(output.contains("selection_reason=no_provider_has_probe_evidence"));
        assert!(output.contains("provider=codex_cli total_score=0 release=not_probed"));
        assert!(output.contains("provider=claude_code total_score=0 release=not_probed"));
        assert!(output.contains("provider=antigravity total_score=0 release=not_probed"));
        assert!(output.contains("hard_disqualifiers=user_level_integration_unproven,late_attach_unproven,terminal_truth_unproven,privacy_boundary_unproven,resource_budget_unproven"));
        assert!(output.contains("first_adapter_candidate=none"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_resource_measurement_fixture_categories_without_live_probe(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-resource-fixture=codex_cli",
        ])?;

        assert!(output.contains("resource_fixture_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("fixture_scope=synthetic_measurement_categories"));
        assert!(output.contains("live_provider_probe=false"));
        assert!(output.contains("measurement=drop_mode_memory_cpu status=category_only"));
        assert!(output.contains("measurement=active_link_memory_cpu status=category_only"));
        assert!(output.contains("measurement=event_to_snapshot_latency status=category_only"));
        assert!(output.contains("measurement=adapter_event_rate status=category_only"));
        assert!(output.contains("measurement=breadcrumb_size status=category_only"));
        assert!(output.contains("measurement=link_exit_behavior status=category_only"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("resource_budget_claim=not_measured"));
        assert!(!output.contains("resource_budget_passed=true"));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_sanitized_evidence_register_without_raw_provider_fields(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-evidence-register=codex_cli",
        ])?;

        assert!(output.contains("evidence_register_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("register_scope=sanitized_probe_summary"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("raw_source_location=false"));
        assert!(output.contains(
            "evidence=official_hooks category=official_surface_inventory status=summary_only"
        ));
        assert!(output.contains(
            "evidence=app_server category=official_surface_inventory status=summary_only"
        ));
        assert!(output
            .contains("evidence=install_rollback category=synthetic_fixture status=summary_only"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(!output.contains("transcript"));
        assert!(!output.contains("prompt"));
        assert!(!output.contains("token="));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_official_evidence_source_locator_without_raw_docs_or_claims(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-official-evidence-source-locator=codex_cli",
        ])?;

        assert!(output.contains("official_evidence_source_locator_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("locator_status=scaffold_only"));
        assert!(output.contains("source_location_retained=false"));
        assert!(output.contains("raw_documentation_retained=false"));
        assert!(output.contains("required_field=source_type"));
        assert!(output.contains("required_field=source_location_redacted_id"));
        assert!(output.contains("required_field=published_or_updated_date"));
        assert!(output.contains("required_field=provider_version_tested"));
        assert!(output.contains("required_field=capability_claim_supported"));
        assert!(output.contains("required_field=known_constraints"));
        assert!(output.contains("source_candidate=hook_reference type=official_documentation"));
        assert!(output.contains("source_candidate=local_api_reference type=official_documentation"));
        assert!(
            output.contains("source_candidate=cli_behavior type=verified_official_cli_behavior")
        );
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("https://"));
        assert!(!output.contains("raw_provider_content=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_sanitized_probe_summary_fixture_without_raw_payloads(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-probe-summary-fixture=claude_code",
        ])?;

        assert!(output.contains("summary_fixture_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("summary_scope=sanitized_probe_result"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("raw_payload_retained=false"));
        assert!(output.contains("capability=discover_session result=probed_candidate"));
        assert!(output.contains("capability=observe_waiting result=probed_candidate"));
        assert!(output.contains("capability=open_workspace result=probed_candidate"));
        assert!(output.contains("capability=open_exact_context result=not_probed"));
        assert!(output.contains("capability=observe_quota_snapshot result=not_probed"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("transcript"));
        assert!(!output.contains("prompt"));
        assert!(!output.contains("tool_input"));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_scores_w4_sanitized_probe_summaries_without_selecting_w5_adapter(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-probe-scorecard=sanitized-fixture",
        ])?;

        assert!(output.contains("scorecard_version=1"));
        assert!(output.contains("score_source=sanitized_fixture"));
        assert!(output.contains("provider=codex_cli total_score=7 release=not_probed"));
        assert!(output.contains("provider=claude_code total_score=8 release=not_probed"));
        assert!(output.contains("provider=antigravity total_score=1 release=not_probed"));
        assert!(
            output.contains("dimension=user_level_ingress codex_cli=2 claude_code=2 antigravity=0")
        );
        assert!(output.contains("dimension=waiting_truth codex_cli=2 claude_code=3 antigravity=0"));
        assert!(output.contains("selection_status=no_adapter_selected"));
        assert!(output.contains("selection_reason=observe_release_not_earned"));
        assert!(output.contains("first_adapter_candidate=none"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("winner="));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_hard_disqualifier_evaluation_from_sanitized_fixture(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-hard-disqualifiers=sanitized-fixture",
        ])?;

        assert!(output.contains("hard_disqualifier_version=1"));
        assert!(output.contains("source=sanitized_fixture"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("provider=antigravity"));
        assert!(output.contains("gate=user_level_install_rollback passed=false"));
        assert!(output.contains("gate=late_attach passed=false"));
        assert!(output.contains("gate=terminal_truth passed=false"));
        assert!(output.contains("gate=privacy_boundary passed=true"));
        assert!(output.contains("gate=resource_budget passed=false"));
        assert!(output.contains("w5_blocked_by_hard_disqualifier=true"));
        assert!(output.contains("adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_provider_probe_audit_aggregate_without_w5_authorization(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-probe-audit"])?;

        assert!(output.contains("w4_probe_audit_version=1"));
        assert!(output.contains("active_work=W4_Provider_Probe_Harness"));
        assert!(output.contains("manifest=present"));
        assert!(output.contains("provider_reports=present"));
        assert!(output.contains("synthetic_config_transactions=present"));
        assert!(output.contains("resource_fixtures=present"));
        assert!(output.contains("evidence_registers=sanitized_only"));
        assert!(output.contains("official_evidence_source_locators=scaffold_only"));
        assert!(output.contains("probe_summaries=sanitized_fixture_only"));
        assert!(output.contains("capability_matrix=present"));
        assert!(output.contains("scorecard=sanitized_fixture_only"));
        assert!(output.contains("hard_disqualifiers=blocking"));
        assert!(output.contains("evidence_gap_summary=present"));
        assert!(output.contains("live_provider_probe=false"));
        assert!(output.contains("read_only_local_probe_run=present"));
        assert!(output.contains("read_only_local_probe_summary=present"));
        assert!(output.contains("read_only_resource_measurement_plan=present"));
        assert!(output.contains("probe_card_execution_plans=present"));
        assert!(output.contains("evidence_retention_policy=present"));
        assert!(output.contains("live_authorization_preflight=blocking"));
        assert!(output.contains("missing_capability_rationale=present"));
        assert!(output.contains("release_decision_logs=present"));
        assert!(output.contains("direct_gate_packets=present"));
        assert!(output.contains("direct_evidence_import_checklist=present"));
        assert!(output.contains("authorized_evidence_runbooks=present"));
        assert!(output.contains("sanitized_evidence_output_template=present"));
        assert!(output.contains("sanitized_evidence_bundle_validator=present"));
        assert!(output.contains("release_elevation_preflight=blocking"));
        assert!(output.contains("w5_observe_adapter_contract=scaffold_only"));
        assert!(output.contains("release_label_evaluation=blocking"));
        assert!(output.contains("probe_phase_status=present"));
        assert!(output.contains("w5_start_preflight=blocking"));
        assert!(output.contains("w4_completion_gate=blocking"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(output.contains("next_work=direct_gate_evidence_when_authorized"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_evidence_gap_summary_across_providers_without_live_actions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-evidence-gap-summary"])?;

        assert!(output.contains("evidence_gap_summary_version=1"));
        assert!(output.contains("provider_count=3"));
        assert!(output.contains("direct_gate_count=7"));
        assert!(output.contains("total_missing_direct_gates=21"));
        assert!(output.contains("provider=codex_cli missing_direct_gates=7"));
        assert!(output.contains("provider=claude_code missing_direct_gates=7"));
        assert!(output.contains("provider=antigravity missing_direct_gates=7"));
        assert!(output.contains(
            "missing_gate=official_evidence_register providers=codex_cli,claude_code,antigravity"
        ));
        assert!(output.contains("missing_gate=install_update_uninstall_real_fixture providers=codex_cli,claude_code,antigravity"));
        assert!(output.contains(
            "missing_gate=live_lifecycle_mapping providers=codex_cli,claude_code,antigravity"
        ));
        assert!(output.contains(
            "missing_gate=late_attach_real_result providers=codex_cli,claude_code,antigravity"
        ));
        assert!(output.contains(
            "missing_gate=context_route_real_result providers=codex_cli,claude_code,antigravity"
        ));
        assert!(output.contains(
            "missing_gate=fault_privacy_real_result providers=codex_cli,claude_code,antigravity"
        ));
        assert!(output.contains(
            "missing_gate=live_resource_measurement providers=codex_cli,claude_code,antigravity"
        ));
        assert!(output.contains("w4_complete=false"));
        assert!(output.contains("w5_start_allowed=false"));
        assert!(output.contains("next_allowed_work=collect_authorized_direct_gate_evidence"));
        assert!(output.contains("live_provider_probe=false"));
        assert!(output.contains("provider_task_started=false"));
        assert!(output.contains("provider_config_read=false"));
        assert!(output.contains("provider_config_written=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_w5_start_preflight_that_blocks_adapter_creation_without_direct_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-w5-start-preflight"])?;

        assert!(output.contains("w5_start_preflight_version=1"));
        assert!(output.contains("w4_complete=false"));
        assert!(output.contains("w5_start_allowed=false"));
        assert!(output.contains("selected_provider=none"));
        assert!(output.contains("required_release_label=supported_observe"));
        assert!(output.contains("provider=codex_cli eligible=false reason=missing_direct_gates"));
        assert!(output.contains("provider=claude_code eligible=false reason=missing_direct_gates"));
        assert!(output.contains("provider=antigravity eligible=false reason=missing_direct_gates"));
        assert!(output.contains("blocking_condition=no_provider_has_supported_observe"));
        assert!(output.contains("blocking_condition=total_missing_direct_gates:21"));
        assert!(output.contains("blocking_condition=hard_disqualifiers_present"));
        assert!(output.contains("blocked_next_work=provider_adapter_creation"));
        assert!(output.contains("allowed_next_work=collect_authorized_direct_gate_evidence"));
        assert!(output.contains("live_provider_probe=false"));
        assert!(output.contains("provider_config_written=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe provider="));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_provider_surface_inventory_from_probe_cards_without_live_claims(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-surface-inventory"])?;

        assert!(output.contains("surface_inventory_version=1"));
        assert!(output.contains("source=probe_cards"));
        assert!(output.contains("verification_status=card_declared_only"));
        assert!(output.contains("provider=codex_cli surface=command_hooks status=candidate"));
        assert!(output.contains("provider=codex_cli surface=app_server status=candidate"));
        assert!(
            output.contains("provider=codex_cli surface=passive_process status=fallback_candidate")
        );
        assert!(output.contains("provider=claude_code surface=command_hooks status=candidate"));
        assert!(output.contains("provider=claude_code surface=user_settings status=candidate"));
        assert!(output
            .contains("provider=claude_code surface=passive_process status=fallback_candidate"));
        assert!(output
            .contains("provider=antigravity surface=passive_process status=fallback_candidate"));
        assert!(
            output.contains("provider=antigravity surface=formal_integration status=unverified")
        );
        assert!(output.contains("live_provider_probe=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_probe_readiness_with_remaining_live_gates(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-probe-readiness"])?;

        assert!(output.contains("probe_readiness_version=1"));
        assert!(output.contains("w4_scaffold_ready=true"));
        assert!(output.contains("manifest_ready=true"));
        assert!(output.contains("surface_inventory_ready=true"));
        assert!(output.contains("provider_reports_ready=true"));
        assert!(output.contains("evidence_register_ready=true"));
        assert!(output.contains("official_evidence_source_locator_ready=true"));
        assert!(output.contains("scorecard_ready=true"));
        assert!(output.contains("hard_disqualifier_ready=true"));
        assert!(output.contains("evidence_gap_summary_ready=true"));
        assert!(output.contains("probe_card_execution_plan_ready=true"));
        assert!(output.contains("evidence_retention_policy_ready=true"));
        assert!(output.contains("live_authorization_preflight_ready=true"));
        assert!(output.contains("missing_capability_rationale_ready=true"));
        assert!(output.contains("release_decision_log_ready=true"));
        assert!(output.contains("capability_matrix_ready=true"));
        assert!(output.contains("release_label_evaluation_ready=true"));
        assert!(output.contains("probe_phase_status_ready=true"));
        assert!(output.contains("w5_start_preflight_ready=true"));
        assert!(output.contains("direct_gate_packet_ready=true"));
        assert!(output.contains("direct_evidence_import_checklist_ready=true"));
        assert!(output.contains("authorized_evidence_runbook_ready=true"));
        assert!(output.contains("sanitized_evidence_output_template_ready=true"));
        assert!(output.contains("sanitized_evidence_bundle_validator_ready=true"));
        assert!(output.contains("release_elevation_preflight_ready=true"));
        assert!(output.contains("w5_observe_adapter_contract_ready=true"));
        assert!(output.contains("w4_completion_gate_ready=true"));
        assert!(output.contains("live_probe_ready=false"));
        assert!(output.contains("w5_ready=false"));
        assert!(output.contains("remaining_gate=live_provider_probe_execution"));
        assert!(output.contains("remaining_gate=live_resource_measurement"));
        assert!(output.contains("remaining_gate=install_rollback_real_fixture"));
        assert!(output.contains("remaining_gate=late_attach_real_result"));
        assert!(output.contains("remaining_gate=terminal_truth_real_result"));
        assert!(output.contains("next_allowed_work=collect_direct_gate_evidence_when_authorized"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_sanitized_local_environment_manifest_without_paths_or_accounts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-local-environment-manifest=read-only-fixture",
        ])?;

        assert!(output.contains("local_environment_manifest_version=1"));
        assert!(output.contains("source=read_only_cli_preflight"));
        assert!(output.contains("provider=codex_cli command_present=true version_status=observed"));
        assert!(
            output.contains("provider=claude_code command_present=true version_status=observed")
        );
        assert!(
            output.contains("provider=antigravity command_present=false version_status=not_found")
        );
        assert!(output.contains("raw_command_path_retained=false"));
        assert!(output.contains("provider_config_read=false"));
        assert!(output.contains("account_data_read=false"));
        assert!(output.contains("network_query=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("AppData"));
        assert!(!output.contains(".claude"));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_read_only_live_probe_dry_run_without_executing_provider_tasks(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-live-probe-dry-run=read-only-fixture",
        ])?;

        assert!(output.contains("live_probe_dry_run_version=1"));
        assert!(output.contains("source=sanitized_environment_manifest"));
        assert!(output.contains("executed=false"));
        assert!(output.contains("action=collect_version_category mode=read_only"));
        assert!(output.contains("action=collect_environment_category mode=read_only"));
        assert!(output.contains("action=confirm_probe_card_surface_inventory mode=read_only"));
        assert!(output.contains("action=prepare_sanitized_evidence_summary mode=read_only"));
        assert!(output.contains("action=prepare_resource_measurement_plan mode=read_only"));
        assert!(output.contains("forbidden_action=install_hook"));
        assert!(output.contains("forbidden_action=mutate_provider_config"));
        assert!(output.contains("forbidden_action=start_provider_task"));
        assert!(output.contains("forbidden_action=query_network_or_app_server"));
        assert!(output.contains("forbidden_action=parse_transcript_or_session_file"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_runs_w4_read_only_local_probe_without_support_claims_or_mutation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-live-probe-run=read-only-local",
        ])?;

        assert!(output.contains("live_probe_run_version=1"));
        assert!(output.contains("mode=read_only_local"));
        assert!(output.contains("executed=true"));
        assert!(output.contains("action=collect_version_category mode=read_only"));
        assert!(output.contains("provider=codex_cli command_name=codex"));
        assert!(output.contains("provider=claude_code command_name=claude"));
        assert!(output.contains("provider=antigravity command_name=antigravity"));
        assert!(output.contains("version_category="));
        assert!(output.contains("environment_category=local_cli_preflight"));
        assert!(output.contains("raw_version_retained=false"));
        assert!(output.contains("raw_command_path_retained=false"));
        assert!(output.contains("provider_config_read=false"));
        assert!(output.contains("provider_config_written=false"));
        assert!(output.contains("network_query=false"));
        assert!(output.contains("provider_task_started=false"));
        assert!(output.contains("transcript_or_session_file_parsed=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_read_only_local_probe_summary_without_capability_elevation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-live-probe-summary=read-only-local",
        ])?;

        assert!(output.contains("live_probe_summary_version=1"));
        assert!(output.contains("summary_scope=sanitized_read_only_local_probe"));
        assert!(output.contains("source=read_only_local_probe_run"));
        assert!(output.contains("provider=codex_cli evidence=version_category"));
        assert!(output.contains("provider=claude_code evidence=version_category"));
        assert!(output.contains("provider=antigravity evidence=version_category"));
        assert!(output.contains("evidence=environment_category status=local_cli_preflight"));
        assert!(output.contains("raw_version_retained=false"));
        assert!(output.contains("raw_command_path_retained=false"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("supported_fuel"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_read_only_resource_measurement_plan_without_measuring_providers(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-resource-measurement-plan=read-only-local",
        ])?;

        assert!(output.contains("resource_measurement_plan_version=1"));
        assert!(output.contains("source=sanitized_read_only_local_probe_summary"));
        assert!(output.contains("executed=false"));
        assert!(output.contains("measurement=drop_mode_memory_cpu mode=planned_read_only"));
        assert!(output.contains("measurement=active_link_memory_cpu mode=planned_read_only"));
        assert!(output.contains("measurement=event_to_snapshot_latency mode=planned_read_only"));
        assert!(output.contains("measurement=adapter_event_rate mode=planned_read_only"));
        assert!(output.contains("measurement=breadcrumb_size mode=planned_read_only"));
        assert!(output.contains("measurement=link_exit_behavior mode=planned_read_only"));
        assert!(output.contains("provider_task_started=false"));
        assert!(output.contains("network_query=false"));
        assert!(output.contains("provider_config_read=false"));
        assert!(output.contains("provider_config_written=false"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("resource_budget_claim=not_measured"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("resource_budget_passed=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_direct_gate_packet_without_executing_live_provider_actions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-direct-gate-packet=codex_cli",
        ])?;

        assert!(output.contains("direct_gate_packet_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("scope=w4_direct_gate_evidence_packet"));
        assert!(output.contains("execution_status=not_executed"));
        assert!(output.contains("requires_explicit_authorization=true"));
        assert!(output.contains("gate=official_evidence_register status=missing"));
        assert!(output.contains("gate=install_update_uninstall_real_fixture status=missing"));
        assert!(output.contains("gate=live_lifecycle_mapping status=missing"));
        assert!(output.contains("gate=late_attach_real_result status=missing"));
        assert!(output.contains("gate=context_route_real_result status=missing"));
        assert!(output.contains("gate=fault_privacy_real_result status=missing"));
        assert!(output.contains("gate=live_resource_measurement status=missing"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(output.contains("forbidden_without_authorization=install_live_hook"));
        assert!(output.contains("forbidden_without_authorization=mutate_provider_config"));
        assert!(output.contains("forbidden_without_authorization=start_provider_task"));
        assert!(output.contains("forbidden_without_authorization=query_network_or_app_server"));
        assert!(output.contains("forbidden_without_authorization=parse_transcript_or_session_file"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_direct_evidence_import_checklist_without_accepting_weak_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-direct-evidence-import-checklist=claude_code",
        ])?;

        assert!(output.contains("direct_evidence_import_checklist_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("source=authorized_local_direct_evidence_only"));
        assert!(output.contains("import_execution=false"));
        assert!(
            output.contains("gate=official_evidence_register requires_authorized_artifact=true")
        );
        assert!(output.contains(
            "gate=install_update_uninstall_real_fixture requires_authorized_artifact=true"
        ));
        assert!(output.contains("gate=live_lifecycle_mapping requires_authorized_artifact=true"));
        assert!(output.contains("gate=late_attach_real_result requires_authorized_artifact=true"));
        assert!(output.contains("gate=context_route_real_result requires_authorized_artifact=true"));
        assert!(output.contains("gate=fault_privacy_real_result requires_authorized_artifact=true"));
        assert!(output.contains("gate=live_resource_measurement requires_authorized_artifact=true"));
        assert!(output.contains("reject_if=sanitized_fixture_only"));
        assert!(output.contains("reject_if=read_only_version_category_only"));
        assert!(output.contains("reject_if=raw_provider_content_present"));
        assert!(output.contains("reject_if=raw_provider_configuration_present"));
        assert!(output.contains("release_elevation_allowed=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_authorized_evidence_runbook_without_executing_provider_actions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-authorized-evidence-runbook=codex_cli",
        ])?;

        assert!(output.contains("authorized_evidence_runbook_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("runbook_status=scaffold_only"));
        assert!(output.contains("requires_explicit_authorization=true"));
        assert!(output.contains("local_raw_evidence_retention=local_only"));
        assert!(output.contains("repo_output=sanitized_artifacts_only"));
        assert!(output.contains("step=prepare_synthetic_workspace action=manual_authorized"));
        assert!(output.contains("step=backup_user_level_provider_config action=local_only"));
        assert!(output.contains("step=execute_probe_card_phases action=manual_authorized"));
        assert!(output.contains("step=collect_direct_gate_artifacts action=manual_authorized"));
        assert!(output.contains("step=redact_to_sanitized_outputs action=required_before_repo"));
        assert!(output.contains("step=run_release_elevation_preflight action=blocking_check"));
        assert!(output.contains("gate=official_evidence_register output=sanitized_summary"));
        assert!(
            output.contains("gate=install_update_uninstall_real_fixture output=category_result")
        );
        assert!(output.contains("gate=live_lifecycle_mapping output=sanitized_event_mapping"));
        assert!(output.contains("gate=late_attach_real_result output=category_result"));
        assert!(output.contains("gate=context_route_real_result output=route_strength_matrix"));
        assert!(output.contains("gate=fault_privacy_real_result output=category_result"));
        assert!(output.contains("gate=live_resource_measurement output=category_metrics"));
        assert!(output.contains("provider_task_started=false"));
        assert!(output.contains("provider_config_read=false"));
        assert!(output.contains("provider_config_written=false"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("release_elevation_allowed=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_sanitized_evidence_output_template_without_raw_artifacts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-sanitized-evidence-output-template=codex_cli",
        ])?;

        assert!(output.contains("sanitized_evidence_output_template_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("source=authorized_direct_evidence_after_redaction"));
        assert!(
            output.contains("repo_artifact=sanitized_probe_report required=true raw_content=false")
        );
        assert!(output.contains("repo_artifact=capability_matrix required=true raw_content=false"));
        assert!(output.contains(
            "repo_artifact=sanitized_event_mapping_fixtures required=true raw_content=false"
        ));
        assert!(output.contains(
            "repo_artifact=test_harness_category_results required=true raw_content=false"
        ));
        assert!(output.contains("repo_artifact=known_limitations required=true raw_content=false"));
        assert!(output.contains("repo_artifact=release_decision required=true raw_content=false"));
        assert!(output.contains("repo_forbidden=raw_prompts_or_transcripts"));
        assert!(output.contains("repo_forbidden=raw_provider_configuration"));
        assert!(output.contains("repo_forbidden=customer_project_source"));
        assert!(output.contains("repo_forbidden=credentials_cookies_or_tokens"));
        assert!(output.contains("release_elevation_allowed=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_sanitized_evidence_bundle_validator_without_importing_artifacts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-sanitized-evidence-bundle-validator=claude_code",
        ])?;

        assert!(output.contains("sanitized_evidence_bundle_validator_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("validator_status=scaffold_only"));
        assert!(output.contains("validation_execution=false"));
        assert!(output.contains("source=authorized_direct_evidence_after_redaction"));
        assert!(output.contains("requires_artifact=sanitized_probe_report"));
        assert!(output.contains("requires_artifact=capability_matrix"));
        assert!(output.contains("requires_artifact=sanitized_event_mapping_fixtures"));
        assert!(output.contains("requires_artifact=test_harness_category_results"));
        assert!(output.contains("requires_artifact=known_limitations"));
        assert!(output.contains("requires_artifact=release_decision"));
        assert!(output.contains("reject_if=raw_prompts_or_transcripts_present"));
        assert!(output.contains("reject_if=raw_provider_configuration_present"));
        assert!(output.contains("reject_if=customer_project_source_present"));
        assert!(output.contains("reject_if=credentials_cookies_or_tokens_present"));
        assert!(output.contains("reject_if=raw_terminal_buffers_present"));
        assert!(output.contains("reject_if=private_endpoint_traffic_present"));
        assert!(output.contains("direct_evidence_claimed=false"));
        assert!(output.contains("release_elevation_allowed=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("validation_execution=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_release_elevation_preflight_without_promoting_provider(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-release-elevation-preflight=codex_cli",
        ])?;

        assert!(output.contains("release_elevation_preflight_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("current_release_label=not_probed"));
        assert!(output.contains("target_release_label=supported_observe"));
        assert!(output.contains("preflight_status=blocked"));
        assert!(output.contains("requirement=direct_gate_packet status=ready"));
        assert!(output.contains("requirement=direct_gate_evidence status=missing"));
        assert!(output.contains("requirement=sanitized_output_template status=ready"));
        assert!(output.contains("requirement=hard_disqualifiers_clear status=false"));
        assert!(output.contains("release_elevation_allowed=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(output.contains("next_allowed_work=collect_authorized_direct_gate_evidence"));
        assert!(!output.contains("current_release_label=supported_observe"));
        assert!(!output.contains("release_elevation_allowed=true"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_w5_observe_adapter_contract_without_creating_adapter(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-w5-observe-adapter-contract=claude_code",
        ])?;

        assert!(output.contains("w5_observe_adapter_contract_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("contract_status=scaffold_only"));
        assert!(output.contains("requires_release_label=supported_observe"));
        assert!(output.contains("allowed_capability=formal_user_level_ingress"));
        assert!(output.contains("allowed_capability=stable_session_identity_where_proved"));
        assert!(output.contains("allowed_capability=workspace_association"));
        assert!(output.contains("allowed_capability=running_freshness_where_proved"));
        assert!(output.contains("allowed_capability=waiting_signal_where_proved"));
        assert!(output.contains("allowed_capability=late_island_attach_after_link_breadcrumb"));
        assert!(output.contains("allowed_capability=workspace_ready_route"));
        assert!(output.contains("allowed_capability=observed_degraded_fallback"));
        assert!(output.contains("excluded_capability=arbitrary_external_session_control"));
        assert!(output.contains("excluded_capability=approval_or_deny_ui"));
        assert!(output.contains("excluded_capability=transcript_or_history_parsing"));
        assert!(output.contains("excluded_capability=task_title_from_raw_prompt"));
        assert!(output.contains("excluded_capability=exact_route_without_exact_evidence"));
        assert!(output.contains("excluded_capability=completion_failure_without_terminal_evidence"));
        assert!(output.contains("excluded_capability=fuel_without_scoped_source"));
        assert!(output.contains("provider_adapter_created=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_control"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_probe_card_execution_plan_without_running_provider(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-probe-card-execution-plan=claude_code",
        ])?;

        assert!(output.contains("probe_card_execution_plan_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("probe_card=docs/pulse-island/17-claude-code-probe-card.md"));
        assert!(output.contains("execution_status=not_executed"));
        assert!(output.contains("phase=environment_manifest status=planned"));
        assert!(output.contains("phase=official_surface_inventory status=planned"));
        assert!(output.contains("phase=install_update_uninstall status=planned"));
        assert!(output.contains("phase=live_lifecycle_mapping status=planned"));
        assert!(output.contains("phase=late_attach status=planned"));
        assert!(output.contains("phase=context_route status=planned"));
        assert!(output.contains("phase=fuel_source status=planned"));
        assert!(output.contains("phase=fault_privacy status=planned"));
        assert!(output.contains("phase=resource_measurement status=planned"));
        assert!(output.contains("phase=release_decision status=planned"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("provider_task_started=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_evidence_retention_policy_without_raw_artifact_retention(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-evidence-retention-policy"])?;

        assert!(output.contains("evidence_retention_policy_version=1"));
        assert!(output.contains("repo_allowed=sanitized_probe_report"));
        assert!(output.contains("repo_allowed=capability_matrix"));
        assert!(output.contains("repo_allowed=sanitized_event_mapping_fixtures"));
        assert!(output.contains("repo_allowed=test_harness_category_results"));
        assert!(output.contains("repo_forbidden=customer_project_source"));
        assert!(output.contains("repo_forbidden=full_prompts_or_transcripts"));
        assert!(output.contains("repo_forbidden=credentials_cookies_or_tokens"));
        assert!(output.contains("repo_forbidden=raw_terminal_buffers"));
        assert!(output.contains("repo_forbidden=private_endpoint_traffic"));
        assert!(output.contains("local_only=provider_configuration_backups"));
        assert!(output.contains("local_only=redacted_error_captures"));
        assert!(output.contains("raw_provider_content_in_repo=false"));
        assert!(output.contains("raw_provider_configuration_in_repo=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("transcript_retained=true"));
        assert!(!output.contains("token="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_live_authorization_preflight_as_not_authorized_by_default(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-live-authorization-preflight=codex_cli",
        ])?;

        assert!(output.contains("live_authorization_preflight_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("authorization_status=not_authorized"));
        assert!(output.contains("live_actions_allowed=false"));
        assert!(output.contains("requires_user_approved_test_workspace=true"));
        assert!(output.contains("requires_disposable_or_authorized_account=true"));
        assert!(output.contains("requires_local_raw_evidence_retention_policy=true"));
        assert!(output.contains("requires_sanitized_report_destination=true"));
        assert!(output.contains("blocked_action=install_live_hook"));
        assert!(output.contains("blocked_action=mutate_provider_config"));
        assert!(output.contains("blocked_action=start_provider_task"));
        assert!(output.contains("blocked_action=query_network_or_app_server"));
        assert!(output.contains("blocked_action=parse_transcript_or_session_file"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("hook_installed=true"));
        assert!(!output.contains("provider_config_mutated=true"));
        assert!(!output.contains("supported_observe"));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_missing_capability_rationale_without_blank_cells(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-missing-capability-rationale=claude_code",
        ])?;

        assert!(output.contains("missing_capability_rationale_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output.contains("scope=w4_absent_capability_explanations"));
        assert!(output
            .contains("capability=discover_process release=not_probed reason=direct_gate_missing"));
        assert!(output
            .contains("capability=discover_session release=not_probed reason=direct_gate_missing"));
        assert!(output
            .contains("capability=observe_running release=not_probed reason=direct_gate_missing"));
        assert!(output
            .contains("capability=observe_waiting release=not_probed reason=direct_gate_missing"));
        assert!(output.contains(
            "capability=observe_completion release=not_probed reason=terminal_truth_missing"
        ));
        assert!(output.contains(
            "capability=observe_failure release=not_probed reason=terminal_truth_missing"
        ));
        assert!(output
            .contains("capability=open_workspace release=not_probed reason=context_route_missing"));
        assert!(output.contains(
            "capability=open_exact_context release=not_probed reason=context_route_missing"
        ));
        assert!(output.contains(
            "capability=observe_quota_snapshot release=not_probed reason=fuel_source_missing"
        ));
        assert!(output.contains("blank_cells_allowed=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("reason=unknown"));
        assert!(!output.contains("reason=blank"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_release_decision_log_without_selecting_provider(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-release-decision-log=codex_cli",
        ])?;

        assert!(output.contains("release_decision_log_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(output.contains("decision_scope=w4_probe_report_release_decision"));
        assert!(output.contains("current_release_label=not_probed"));
        assert!(output.contains("decision=defer_provider_pending_direct_evidence"));
        assert!(output.contains("outcome_candidate=proceed_to_narrow_adapter status=blocked"));
        assert!(output.contains("outcome_candidate=process_observed_only status=blocked"));
        assert!(output.contains("outcome_candidate=experimental_observation status=blocked"));
        assert!(output.contains("outcome_candidate=defer_provider status=current"));
        assert!(output.contains("outcome_candidate=reject_integration status=not_decided"));
        assert!(output.contains("blocking_reason=official_evidence_register_missing"));
        assert!(output.contains("blocking_reason=install_update_uninstall_real_fixture_missing"));
        assert!(output.contains("blocking_reason=live_lifecycle_mapping_missing"));
        assert!(output.contains("blocking_reason=late_attach_real_result_missing"));
        assert!(output.contains("blocking_reason=terminal_truth_real_result_missing"));
        assert!(output.contains("decision_log_sanitized=true"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_full_capability_matrix_without_blank_or_elevated_rows(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-capability-matrix=codex_cli",
        ])?;

        assert!(output.contains("capability_matrix_version=1"));
        assert!(output.contains("provider=codex_cli"));
        assert!(
            output.contains("matrix_template=docs/pulse-island/15-provider-capability-probe.md")
        );
        assert!(output.contains("integration_mode=read_only_inventory"));
        assert!(output.contains("blank_cells_allowed=false"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("raw_provider_configuration=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));

        for capability in [
            "discover_process",
            "discover_session",
            "observe_running",
            "observe_waiting",
            "observe_completion",
            "observe_failure",
            "observe_safe_title",
            "open_exact_context",
            "open_workspace",
            "open_official_usage",
            "observe_session_tokens",
            "observe_quota_snapshot",
            "observe_quota_limit",
            "control_decision",
            "control_stop_steer_resume",
        ] {
            assert!(output.contains(&format!(
                "capability={capability} evidence_source=missing probe_result=not_probed identity_strength=none health_ceiling=unavailable release=not_probed"
            )));
        }

        assert!(output.contains(
            "capability=observe_safe_title evidence_source=missing probe_result=not_probed"
        ));
        assert!(output.contains(
            "capability=control_stop_steer_resume evidence_source=missing probe_result=not_probed"
        ));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_release_label_evaluation_without_elevating_sanitized_fixture(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-release-label-evaluation=sanitized-fixture",
        ])?;

        assert!(output.contains("release_label_evaluation_version=1"));
        assert!(output.contains("source=sanitized_fixture"));
        assert!(output.contains("current_release_label=not_probed"));
        assert!(output.contains("evaluation_policy=direct_gate_required"));
        assert!(output.contains(
            "label=process_observed eligible=false reason=direct_process_evidence_missing"
        ));
        assert!(output.contains("label=experimental_attached eligible=false reason=install_late_attach_or_truth_gate_missing"));
        assert!(
            output.contains("label=supported_observe eligible=false reason=observe_gate_missing")
        );
        assert!(
            output.contains("label=supported_fuel eligible=false reason=fuel_source_gate_missing")
        );
        assert!(output.contains(
            "label=supported_control eligible=false reason=control_safety_review_missing"
        ));
        assert!(output.contains("release_recommendation=not_probed"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("eligible=true"));
        assert!(!output.contains("supported_observe provider="));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_probe_phase_status_without_executing_live_phases(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli([
            "pulse-island-spike",
            "--provider-probe-phase-status=claude_code",
        ])?;

        assert!(output.contains("probe_phase_status_version=1"));
        assert!(output.contains("provider=claude_code"));
        assert!(output
            .contains("phase=p0_official_surface_inventory status=scaffolded execution=read_only"));
        assert!(output.contains("phase=p1_passive_process_discovery status=not_executed execution=requires_authorization"));
        assert!(output.contains(
            "phase=p2_installation_rollback status=not_executed execution=requires_authorization"
        ));
        assert!(output.contains(
            "phase=p3_lifecycle_semantics status=not_executed execution=requires_authorization"
        ));
        assert!(output
            .contains("phase=p4_late_attach status=not_executed execution=requires_authorization"));
        assert!(output.contains(
            "phase=p5_context_routing status=not_executed execution=requires_authorization"
        ));
        assert!(output.contains(
            "phase=p6_fuel_telemetry status=not_executed execution=requires_authorization"
        ));
        assert!(output.contains("phase=p7_fail_open_fault_injection status=not_executed execution=requires_authorization"));
        assert!(output.contains(
            "phase=p8_performance_retention status=not_executed execution=requires_authorization"
        ));
        assert!(output.contains("live_provider_probe=false"));
        assert!(output.contains("provider_task_started=false"));
        assert!(output.contains("provider_config_read=false"));
        assert!(output.contains("provider_config_written=false"));
        assert!(output.contains("raw_provider_content=false"));
        assert!(output.contains("capability_claims_enabled=false"));
        assert!(output.contains("w5_adapter_creation_authorized=false"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w4_completion_gate_that_keeps_w5_blocked_until_direct_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--provider-w4-completion-gate"])?;

        assert!(output.contains("w4_completion_gate_version=1"));
        assert!(output.contains("w4_scaffold_ready=true"));
        assert!(output.contains("direct_gate_packets_ready=true"));
        assert!(output.contains("w4_complete=false"));
        assert!(output.contains("w5_start_allowed=false"));
        assert!(output.contains("remaining_gate=official_evidence_register"));
        assert!(output.contains("remaining_gate=install_update_uninstall_real_fixture"));
        assert!(output.contains("remaining_gate=live_lifecycle_mapping"));
        assert!(output.contains("remaining_gate=late_attach_real_result"));
        assert!(output.contains("remaining_gate=context_route_real_result"));
        assert!(output.contains("remaining_gate=fault_privacy_real_result"));
        assert!(output.contains("remaining_gate=live_resource_measurement"));
        assert!(output.contains("next_allowed_work=collect_authorized_direct_gate_evidence"));
        assert!(!output.contains("supported_observe"));
        assert!(!output.contains("winner="));
        Ok(())
    }

    #[test]
    fn cli_exports_w2_review_ready_and_w3_authorized() -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--w2-review-ready"])?;

        assert!(output.contains("w2_review_ready=true"));
        assert!(output.contains("gate_audit=21/21"));
        assert!(output.contains("adapter_readiness=plan,state,action,replay"));
        assert!(output.contains("evidence_doc=docs/pulse-island/W2-GATE-AUDIT.md"));
        assert!(output.contains("scope=mock_presentation_plan_only"));
        assert!(output.contains("w3_ready=true"));
        Ok(())
    }

    #[test]
    fn cli_exports_w2_review_manifest_as_machine_checkable_contract(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = run_cli(["pulse-island-spike", "--w2-review-manifest"])?;

        assert!(output.contains("manifest_version=1"));
        assert!(output.contains("package=W2 Native Signal Shell"));
        assert!(output.contains("scope=mock_presentation_plan_only"));
        assert!(output.contains("review_status=accepted_for_w3"));
        assert!(output.contains("w2_review_ready=true"));
        assert!(output.contains("w3_ready=true"));
        assert!(output.contains("w3_authorized_scope=link_shim_drop_mode_synthetic_only"));
        assert!(output.contains("gate_audit_checklist=21"));
        assert!(output.contains("gate_audit_evidence=21"));
        assert!(output.contains("adapter_readiness=plan,state,action,replay"));
        assert!(output.contains("forbidden_dependency_hits=0"));
        assert!(output.contains(
            "later_gated_work=live_provider_hooks,provider_adapters,provider_config,route_activation"
        ));
        let stale_field_name = ["blocked", "_work="].join("");
        assert!(!output.contains(&stale_field_name));
        Ok(())
    }
}
