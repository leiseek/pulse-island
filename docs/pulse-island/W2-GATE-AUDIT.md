# Pulse Island · W2 Gate Audit

**Status:** Accepted for W4 sequencing through W3 evidence
**Scope:** W2 Native Signal Shell with mocked `PresentationPlan` input only
**Last updated:** 2026-07-07

This audit maps the Spike A checklist from `12-spike-a-native-signal-benchmark.md` and the W2 package boundary from `24-implementation-work-packages.md` to current repository evidence.

`pulse-island-spike --gate-audit` is the W2 evidence aggregate. W2 and W3 are accepted for sequencing and are now regression evidence only; W4 Provider Probe Harness is the active implementation boundary under `15-provider-capability-probe.md`.

## Verification Commands

Latest required verification set:

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets
cargo tree --workspace --depth 1
cargo run -p pulse-island-spike -- --adapter-plan-policy
cargo run -p pulse-island-spike -- --adapter-state-policy
cargo run -p pulse-island-spike -- --adapter-action-policy
cargo run -p pulse-island-spike -- --adapter-replay-policy
cargo run -p pulse-island-spike -- --w2-review-ready
cargo run -p pulse-island-spike -- --w2-review-manifest
cargo run -p pulse-island-spike -- --architecture-policy
cargo run -p pulse-island-spike -- --gate-audit
cargo test -p pulse-win32-hwnd
cmd /c "call \"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat\" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32"
cmd /c "call \"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat\" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32-hwnd"
```

Current Gate A aggregate output:

```text
gate_audit checklist_items=21 evidence_items=21
functional=6/6 window=6/6 performance=5/5 architecture=4/4
w3_ready=true w2_review=accepted
w4_ready=true w3_review=accepted
active_work=W4_Provider_Probe_Harness
scope=mock_presentation_plan_only
```

Current W2 review manifest output:

```text
manifest_version=1
package=W2 Native Signal Shell
scope=mock_presentation_plan_only
review_status=accepted_for_w3
w2_review_ready=true
w3_ready=true
w3_authorized_scope=link_shim_drop_mode_synthetic_only
gate_audit_checklist=21
gate_audit_evidence=21
adapter_readiness=plan,state,action,replay
evidence_doc=docs/pulse-island/W2-GATE-AUDIT.md
forbidden_dependency_hits=0
later_gated_work=live_provider_hooks,provider_adapters,provider_config,route_activation
```

Current architecture policy output:

```text
architecture_policy checked_manifests=5 passed=true
forbidden_dependency_hits=0
mock_plan_replaceable=true
hwnd_boundary_manifest=true
link_transport_boundary_manifest=true
hwnd_native_api_adapter=true
hwnd_create_window_factory=true
hwnd_message_pump=true
hwnd_wndproc_hit_test=true
hwnd_wndproc_mouse_activate=true
hwnd_wndproc_mouse_dispatch=true
hwnd_wndproc_paint_dispatch=true
browser_runtime=false sqlite=false provider_adapter=false
```

## Scope Guard

| Boundary | Evidence | Status |
|---|---|---|
| Mock `PresentationPlan` input only | `PresentationPlanSource`, `MockPresentationPlanSource`, and `subscribe` seam in `pulse-island-ui` | Passing |
| Native adapter remains policy-driven before runtime wiring | `NativeWindowAdapterPlan` composes style, visibility, placement, hit-test, and hotkey policies before dispatch to the isolated HWND boundary | Passing |
| HWND unsafe boundary is explicit and isolated | `pulse-win32-hwnd` is included in the workspace and architecture policy; it validates payload commands before native calls and contains the MSVC-only `windows-sys` API adapter | Passing |
| No provider adapter, live Hook install, provider config, network, browser runtime, or production route activation in W2 | `pulse-island-spike --architecture-policy`; workspace manifest review | Passing |
| W3 implementation boundary is explicitly authorized after W2 evidence | W2 evidence remains accepted; `pulse-island-spike --gate-audit` now reports `w3_ready=true w2_review=accepted`, `w4_ready=true w3_review=accepted`, and `active_work=W4_Provider_Probe_Harness` after W3 acceptance | Passing |

## Functional Checklist

| Spike A requirement | Evidence | Status |
|---|---|---|
| All S0-S8 scenarios render deterministically | `mock_scenario_catalog_covers_spike_a_s0_to_s8_in_order`; `scenario_harness_lists_spike_a_scenarios_in_catalog_order`; CLI supports `S0`-`S8` | Passing |
| Signal -> Peek -> Focus Card navigation works | `spike_a_mock_scenarios_drive_expected_signal_peek_and_focus_models`; `peek_view_model_renders_plan_rows_without_resorting_or_overflow`; `focus_card_keeps_route_label_and_exposes_no_p0_controls` | Passing |
| Compact Island never timer-rotates tasks | `signal_truth_priority_forbids_timer_rotation_and_keeps_fuel_secondary`; `pulse-island-spike --truth-priority-policy` | Passing |
| Fuel Thread is secondary to task signal | `SignalTruthPriorityDecision` keeps Fuel secondary and cannot override primary state; `pulse-island-spike --truth-priority-policy` | Passing |
| Mock route labels preserve workspace versus original-task distinction | `mock_scenarios_cover_all_w2_route_label_strengths`; `pulse-island-spike --route-policy` exports Exact/Strong/Useful/Weak labels | Passing |
| Escape and global shortcut behavior is predictable | `global_shortcut_opens_palette_with_keyboard_focus`; `escape_closes_focused_surface_and_returns_external_focus`; `pulse-island-spike --focus-policy`; `--hotkey-policy` | Passing |

## Window Behavior Checklist

| Spike A requirement | Evidence | Status |
|---|---|---|
| Transparent margins click through | `hit_test_distinguishes_transparent_margin_and_interactive_body`; `hit_targets_map_to_documented_win32_nchittest_codes`; `pulse-island-spike --hit-test-policy` | Passing |
| Visible interaction zones receive input | Hit-test model maps body to `HTCLIENT` and drag grip to `HTCAPTION`; `pulse-island-spike --hit-test-policy` | Passing |
| Passive transitions do not steal focus | `passive_plan_update_never_steals_focus_or_opens_surfaces`; `compact_click_opens_peek_without_focus_theft`; `pulse-island-spike --focus-policy` | Passing |
| Alt+Tab does not show compact island | `compact_window_style_policy_matches_non_activating_toolwindow_contract`; `compact_window_style_policy_maps_to_documented_win32_style_bits`; `pulse-island-spike --window-policy` | Passing |
| DPI/monitor changes preserve visible placement | `remembered_logical_placement_resolves_against_monitor_dpi_and_work_area`; `placement_clamps_window_to_current_work_area`; `dpi_and_text_scale_preserve_nonzero_text_dimensions`; `pulse-island-spike --dpi-policy` | Passing |
| Fullscreen policy hides Island | `immersive_window_policy_hides_without_fighting_fullscreen_surfaces`; S8 immersive scenario; `pulse-island-spike --window-policy` | Passing |

## Additional Adapter Readiness Evidence

| W2 adapter-readiness requirement | Evidence | Status |
|---|---|---|
| Native adapter plan reuses the compact window instead of recreating/destroying it | `native_adapter_plan_reuses_non_activating_compact_window`; `native_adapter_plan_hides_immersive_without_destroying_or_replaying`; `pulse-island-spike --adapter-plan-policy` | Passing |
| Native adapter state is idempotent across repeated visible and immersive frames | `native_adapter_state_creates_once_across_repeated_visible_frames`; `native_adapter_state_hides_and_restores_without_recreate_or_hotkey_churn`; `pulse-island-spike --adapter-state-policy` | Passing |
| Native adapter action diff is ordered and idempotent before real HWND wiring | `native_adapter_actions_are_ordered_and_idempotent_for_visible_plan`; `native_adapter_actions_hide_immersive_without_destroy_or_hotkey_churn`; `native_adapter_actions_update_hit_test_layout_without_window_churn`; `native_adapter_actions_diff_placement_style_and_hotkey_independently`; `pulse-island-spike --adapter-action-policy` | Passing |
| Native adapter action diffs can drive a safe backend seam before unsafe HWND code exists | `native_adapter_driver_applies_ordered_actions_and_updates_state`; `native_adapter_driver_does_not_advance_state_after_sink_failure` prove ordered sink application and no state advance after backend failure | Passing |
| Native adapter command diffs carry backend payloads before real HWND wiring | `native_adapter_commands_carry_backend_payloads_for_visible_plan`; `native_adapter_driver_applies_payload_commands_to_command_sink`; `native_adapter_driver_does_not_advance_state_after_command_sink_failure` prove style bits, hit-test layout, placement, and hotkey payloads reach the sink and state advances only after command success | Passing |
| HWND backend boundary rejects invalid command payloads before native calls | `pulse-win32-hwnd` `HwndCommandPreflightSink` accepts ordered visible-plan commands, rejects move/resize before window creation, rejects invalid hotkey payloads, and records only validated commands | Passing |
| HWND native backend calls native API only after preflight and HWND creation | `HwndNativeBackend` applies payload commands through `HwndCompactWindowFactory` and `HwndNativeApi`; tests prove fake native calls receive the created HWND only after preflight, and invalid native commands do not call the API | Passing |
| Compact HWND factory creates a real hidden native window on MSVC | `WindowsSysCompactWindowFactory` registers a content-free class, calls `CreateWindowExW`, and destroys the returned HWND; MSVC test `windows_sys_compact_factory_creates_and_destroys_hidden_compact_hwnd` passes | Passing |
| Compact HWND message pump is nonblocking and bounded | `WindowsSysMessagePump` drains pending messages with `PeekMessageW(PM_REMOVE)`, `TranslateMessage`, and `DispatchMessageW`; MSVC test posts `WM_NULL` and proves the pending message is removed and dispatched without blocking | Passing |
| Compact HWND WndProc returns hit-test codes from cached layout | `HwndHitTestBridge` stores content-free geometry and WndProc handles `WM_NCHITTEST` via `ScreenToClient`; MSVC test sends `WM_NCHITTEST` and verifies `HTCAPTION` and `HTTRANSPARENT` results | Passing |
| Compact HWND WndProc rejects mouse activation | WndProc handles `WM_MOUSEACTIVATE` by returning `MA_NOACTIVATE`; MSVC test sends `WM_MOUSEACTIVATE` and proves the compact HWND does not activate on mouse boundary input | Passing |
| Compact HWND WndProc dispatches content-free mouse input | `HwndMouseInputBridge` maps only client-body `WM_LBUTTONUP` to `CompactPrimaryClick`; transparent margin, drag grip, and outside points dispatch nothing; MSVC test sends `WM_LBUTTONUP` and drains the content-free HWND input queue | Passing |
| Compact HWND WndProc dispatches content-free paint readiness | `HwndPaintBridge` maps `WM_PAINT` to `CompactRepaintRequested`; MSVC test sends `WM_PAINT`, drains the content-free render queue, and WndProc validates the paint region | Passing |
| Native adapter replay stays stable across S7 rapid transitions and S8 immersive suppression | `pulse-island-spike --adapter-replay-policy` reports one window generation, one create, zero destroy, zero activation, one hotkey registration, bounded S7 action budget, and immersive hide/clear-topmost actions | Passing |
| W2 review readiness is machine-checkable and authorizes W3 | `pulse-island-spike --w2-review-ready` reports `w2_review_ready=true`, `gate_audit=21/21`, `adapter_readiness=plan,state,action,replay`, `scope=mock_presentation_plan_only`, and `w3_ready=true` | Passing |
| W2 review contract is machine-checkable as a stable manifest | `pulse-island-spike --w2-review-manifest` reports manifest version, W2 package/scope/status, Gate A counts, adapter readiness, forbidden dependency hits, W3 authorized scope, and later gated provider work | Passing |
| Pure Win32 constants match current Rust for Windows bindings before real HWND wiring | MSVC-only `pulse-win32` tests `style_bits_match_current_windows_sys_bindings` and `hit_test_and_hotkey_values_match_current_windows_sys_bindings` compare style, hit-test, and hotkey constants against `windows-sys 0.61.2` | Passing |

## Performance Checklist

| Spike A requirement | Evidence | Status |
|---|---|---|
| All Gate A metrics pass | `measurement_policy_exports_gate_a_targets_without_task_content`; `measurement_report_evaluates_gate_a_samples_and_requires_every_metric`; `pulse-island-spike --measurement-policy` | Passing |
| No permanent 60 Hz app-side timer exists | `static_render_policy_forbids_app_side_frame_loop_when_quiet`; `pulse-island-spike --measurement-policy` | Passing |
| No resource growth across rapid state transitions | `render_resource_policy_forbids_per_task_surfaces_and_detects_growth`; `pulse-island-spike --resource-policy` | Passing |
| Handle counts remain stable after 1,000 Peek and Focus cycles | `surface_handle_report_requires_stable_handles_after_peek_focus_cycles`; `pulse-island-spike --surface-handle-policy` | Passing |
| No D3D resource leak after 1,000 state transitions | `RenderResourceReport` requires zero device/surface/D3D/handle growth; `pulse-island-spike --resource-policy` | Passing |

## Architecture Checklist

| Spike A requirement | Evidence | Status |
|---|---|---|
| UI has no provider dependencies | `pulse-island-spike --architecture-policy` checks UI, Win32, HWND-boundary, and spike manifests for forbidden provider-adapter dependencies | Passing |
| UI has no SQLite dependency | `pulse-island-spike --architecture-policy`; `cargo tree --workspace --depth 1` | Passing |
| Mock plan source can be replaced by pipe-backed source without UI API change | `PresentationPlanSource` trait exposes `current_plan` and `subscribe`; mock implementation is isolated at the seam | Passing |
| No browser/WebView/Tauri/Electron dependency exists | `pulse-island-spike --architecture-policy`; workspace manifests | Passing |

## Native Adapter Preflight

The first production `windows-sys` HWND API adapter is now isolated in `pulse-win32-hwnd`. It wraps `SetWindowLongPtrW`, `SetWindowPos`, `ShowWindow`, `RegisterHotKey`, and `UnregisterHotKey` behind the safe `HwndNativeApi` trait and is compiled only on MSVC Windows targets.

Compact HWND creation is also isolated in `pulse-win32-hwnd`: `WindowsSysCompactWindowFactory` registers a content-free class, calls `CreateWindowExW` for a hidden popup/toolwindow/no-activate compact window, and destroys it with `DestroyWindow`. `WindowsSysMessagePump` provides a nonblocking, bounded pending-message drain for the compact HWND. WndProc now handles `WM_NCHITTEST` from cached content-free hit-test geometry, `WM_MOUSEACTIVATE` with `MA_NOACTIVATE`, client-body `WM_LBUTTONUP` as a content-free `CompactPrimaryClick` input event, and `WM_PAINT` as a content-free `CompactRepaintRequested` render event. Direct2D/DirectComposition drawing has not been wired yet.

Current docs were fetched with ctx7 using `/microsoft/windows-rs`, and binding parity is now verified through MSVC-only `pulse-win32` dev tests.

- Active toolchain is `stable-x86_64-pc-windows-gnu`.
- GNU `cargo test --workspace` remains the default verification path and does not build the MSVC-only `windows-sys` adapter.
- `stable-x86_64-pc-windows-msvc` plus `VsDevCmd.bat` successfully runs `cargo test -p pulse-win32`, including 28 tests and the two `windows-sys` parity tests.
- `stable-x86_64-pc-windows-msvc` plus `VsDevCmd.bat` successfully runs `cargo test -p pulse-win32-hwnd`, including 16 tests covering native-backend boundary behavior, real hidden HWND create/destroy, posted-message pump drain, real WndProc `WM_NCHITTEST`, real WndProc `WM_MOUSEACTIVATE -> MA_NOACTIVATE`, real WndProc content-free `WM_LBUTTONUP` dispatch, and real WndProc content-free `WM_PAINT` dispatch verification.
- A GNU route is possible only if rustup's self-contained `dlltool.exe` is added to PATH before building `windows-sys`.
- `windows-window v0.0.0` was probed as a possible safe wrapper but does not expose the documented `Window` API from the current docs; it is not used.

## Review Decision

W2 is accepted with all 21 Spike A checklist items mapped to deterministic evidence. The current implementation is still a mock `PresentationPlan` native shell proof with an isolated HWND API boundary, a minimal real hidden compact HWND creation factory, a bounded nonblocking message drain, WndProc `WM_NCHITTEST` integration, WndProc `WM_MOUSEACTIVATE -> MA_NOACTIVATE` activation-edge handling, content-free client mouse-click dispatch, and content-free paint readiness dispatch. W2 is now regression evidence only; W4 Provider Probe Harness is active after W3 acceptance. Live provider Hook installation, provider adapters, provider configuration mutation, and production route activation are later-gated work, not W2 follow-up.

Next implementation boundary:

1. Continue W4 Provider Probe Harness as read-only capability discovery.
2. Keep provider adapters, live Hook installation, provider configuration mutation, and route activation gated until W4/W5 evidence explicitly authorizes them.
3. Keep W2 evidence available as regression coverage, not as a reason to keep expanding W2.
