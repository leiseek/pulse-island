# Pulse Island W4 Provider Probe Harness Plan

Updated: 2026-07-10T23:59:00+08:00
Workspace: D:\Workspace\pulse-island
Target agent: Codex

## Current objective

Continue W4 Provider Probe Harness in canonical order:

```text
W0 Workspace Foundation
-> W1 State Truth Kernel
-> W2 Native Signal Shell
-> W3 Link / Shim / Drop Mode
-> W4 Provider Probe Harness
-> W5 First narrow supported Observe adapter
-> W6 independent Context / Fuel enhancements
```

`docs/pulse-island/24-implementation-work-packages.md` and `docs/pulse-island/25-consistency-closure.md` remain authoritative.

## Current active boundary

W0/W1/W2/W3 have gate evidence. The current implementation boundary is W4 Provider Probe Harness under `docs/pulse-island/15-provider-capability-probe.md` and the provider-specific Probe Cards.

Current W4 work includes read-only discovery plus authorized, sanitized Codex CLI smoke evidence. Hook installation/rollback and lifecycle gates remain open; no provider is release-selected.

Do not reinterpret older W0/W1/W2/W3 review notes as active tasks. W2 and W3 have no current implementation queue unless a fresh regression test fails. `docs/pulse-island/W1-GATE-AUDIT.md`, `docs/pulse-island/W2-GATE-AUDIT.md`, `docs/pulse-island/W3-GATE-AUDIT.md`, and `docs/pulse-island/25-consistency-closure.md` are the current sequencing evidence.

Do not begin live provider Hook installation, provider configuration changes, provider adapters, real provider process control, provider Fuel collection, transcript/session parsing, or exact route activation until the relevant later gate is reached.

## Immediate execution pointer

Continue inside W4 and the 1.0 delivery plan. W3 now has executable Shim/Link ingress, persistence, multi-frame Island handshake, and process-level Codex event coverage. W4 has sanitized Codex CLI execution and surface evidence; Hook installation/rollback, lifecycle, Late Attach, resource gates, and production Island host remain open. Do not mutate provider configuration until the isolated fixture is ready.

## W0 evidence

- Cargo workspace exists with provider-neutral crates only.
- `cargo tree --workspace --depth 1` shows only intra-workspace dependencies.
- Core crates have no provider adapter, network, UI, Win32, SQLite, browser, or generic utils dependency.
- Workspace lint policy forbids unsafe code and denies `unwrap`, `expect`, and `panic` via Clippy.

## W1 evidence implemented

- Separate axes exist for provider release status, task health, route capability, feature capability, lifecycle, attention, privacy profile, route strength, and safe summary.
- Protocol admission validates frame length, version, forbidden fields, and structured-source approval before mutation.
- Island-facing protocol exposes state and health messages only: `HelloAck`, `FullSnapshot`, `SnapshotDelta`, `LinkHealth`, and `ProtocolError`.
- Reducer fixtures cover lifecycle, waiting truthfulness, stale waiting recovery, terminal protection, failed terminal priority, process-exit offline semantics, freshness degradation/recovery, identity/PID safety, process-only ceiling, Fuel separation, route labels, Safe Mode Shim ingress, explicit retention decisions, resource stall priority, weak evidence not downgrading attached truth, and feature capability declaration.
- Arbitration follows the canonical order from `25`: failed, waiting, verified limit, pinned, fuel risk, resource stall, running, recent terminal, idle/observed.
- W1 end-to-end truth fixtures currently pass under `cargo test --workspace`.
- W1 gate evidence is tracked in `docs/pulse-island/W1-GATE-AUDIT.md`, currently accepted for W2/W3 sequencing.
- W2 scaffold has begun with `pulse-island-ui` mock `PresentationPlan` seam and `pulse-win32` pure hit-test/DPI primitives.

## W2 evidence implemented

- `pulse-island-ui` exposes `SignalViewModel`, `PeekViewModel`, `FocusCardViewModel`, `PresentationPlanSource`, and `MockPresentationPlanSource`.
- `PresentationPlanSource` now includes the W2 `subscribe` seam; the mock source synchronously delivers the current mocked `PresentationPlan` without Link or transport dependencies.
- Spike A mock scenario catalog covers S0-S8 in deterministic order.
- Spike A mock scenarios now cover Exact, Strong, Useful, and Weak route labels through rendered Signal/Peek/Focus view models.
- Scenario-derived shell view model keeps task truth in the `PresentationPlan`, derives Peek/Focus without re-arbitrating, hides compact Island for idle/immersive cases, and stops motion under immersive policy.
- `pulse-island-ui` includes compact Signal truth-priority policy: the primary story remains the `PresentationPlan` primary, timer-based task rotation is forbidden, and trustworthy low-Fuel treatment is visible only as a secondary Fuel Thread that cannot override task state.
- P0 Focus Card exposes no provider control actions.
- `pulse-win32` includes pure hit-test primitives for transparent margin, client area, drag grip, and outside points.
- `pulse-win32` maps pure hit-test targets to documented `WM_NCHITTEST` result codes: `HTTRANSPARENT`, `HTCLIENT`, `HTCAPTION`, and `HTNOWHERE`.
- `pulse-win32` includes a pure default Command Palette global hotkey registration chord: Ctrl+Shift+Space, with disabled-state no-registration behavior.
- `pulse-win32` includes DPI scaling and work-area placement clamp primitives without linking Windows APIs.
- `pulse-win32` includes monitor-aware logical placement resolution: remembered per-monitor logical coordinates are scaled by monitor DPI, offset to the monitor work area, and clamped after topology changes.
- `pulse-win32` includes Windows text-scale-aware sizing that combines per-monitor DPI and text scaling while preserving non-zero text dimensions.
- `apps/pulse-island-spike` is now in the workspace as the disposable Spike A runner.
- Spike runner CLI can list scenarios, run a named S0-S8 scenario, and replay deterministic transition sequences without provider/Link/Win32 API dependencies.
- S7 replay currently proves stable logical window generation across rapid state changes at the pure harness layer.
- `pulse-island-ui` includes a pure Palette/Peek/Focus interaction state machine proving no focus theft on passive updates or compact click, explicit focus for Palette shortcut and Focus Card row selection, and Escape collapse behavior.
- `pulse-island-spike --focus-policy` exports deterministic focus-policy diagnostics.
- `pulse-island-spike --truth-priority-policy` exports deterministic no-timer-rotation and Fuel-secondary diagnostics.
- `pulse-island-ui` includes accessible Signal metadata, reduced-motion animation policy, high-contrast token policy, and deterministic Peek/Palette keyboard navigation models.
- `pulse-island-spike --accessibility-policy` exports deterministic accessibility/motion diagnostics.
- `pulse-island-ui` includes the P0 Command Palette command model from `09`, with no provider control or high-risk commands.
- `pulse-island-spike --palette-policy` exports deterministic Command Palette diagnostics.
- `pulse-island-ui` includes compact Signal width-degradation policy that preserves glyph/subject first and hides lower-priority slots before truncating core subject text.
- `pulse-island-spike --layout-policy` exports deterministic compact layout diagnostics.
- `pulse-island-ui` includes a pure shell surface lifecycle model proving repeated Peek/Focus/Palette cycles do not recreate the compact logical window or leave transient surfaces active.
- `pulse-island-ui` includes focused-surface handle stability reporting: after 1,000 Peek and 1,000 Focus open/close cycles, diagnostic USER/GDI handle snapshots must show zero growth and no active transient surfaces.
- `pulse-island-ui` includes a Palette invocation policy: global shortcut can be disabled, and immersive Palette access opens only when explicitly permitted.
- `pulse-island-spike --route-policy` and `--lifecycle-policy` export deterministic route-label and surface-lifecycle diagnostics.
- `pulse-island-spike --surface-handle-policy` exports deterministic Peek/Focus handle-stability diagnostics.
- `pulse-island-spike --hit-test-policy` exports deterministic transparent/client/drag/outside hit-test diagnostics with Win32 result-code values.
- `pulse-island-spike --hotkey-policy` exports deterministic hotkey registration and Palette invocation diagnostics.
- `pulse-island-spike --dpi-policy` exports deterministic monitor fallback, DPI placement, and text-scaling diagnostics.
- `pulse-island-ui` includes Gate A measurement target schema and static render/no-app-frame-loop policy.
- `pulse-island-ui` includes a diagnostics-only performance overlay model that is hidden in normal mode and renders only Gate A metric metadata, never task content.
- `pulse-island-ui` includes pure compositor animation-class policy for Arrival, StateTransition, AttentionPulse, Expansion, FuelCue, and Completion; all W2 plans forbid app-side frame loops, bound attention pulses, settle to static state, and honor reduced/stopped motion.
- `pulse-island-ui` includes pure render-resource policy and report evaluation: one shared D3D device, no per-task full render surfaces, virtualized Focus Card rows, and zero device/surface/D3D/handle growth across the 1,000-transition W2 leak check.
- `pulse-island-ui` includes W2 render-cache invalidation policy: DPI/theme/font/state-layout changes invalidate the correct bounded caches, text layout cache is capped, and task content is not cached.
- `pulse-island-ui` includes a pure Gate A measurement fixture harness with diagnostic samples, per-metric aggregation, missing-metric failure, and pass/fail evaluation without task content.
- `pulse-island-spike --measurement-policy` exports deterministic measurement-policy and sample-report diagnostics.
- `pulse-island-spike --overlay-policy` exports deterministic diagnostics-overlay visibility and content-boundary checks.
- `pulse-island-spike --animation-policy` exports deterministic animation-class diagnostics.
- `pulse-island-spike --resource-policy` exports deterministic render-resource stability diagnostics.
- `pulse-island-spike --cache-policy` exports deterministic render-cache bounds and invalidation diagnostics.
- `pulse-win32` includes pure compact-window style policy for popup/topmost/toolwindow/noactivate, Alt+Tab exclusion, and non-permanent click-through behavior.
- `pulse-win32` includes MSVC-only `windows-sys 0.61.2` dev parity tests proving the pure style, hit-test, and hotkey constants match current Rust for Windows bindings without adding production HWND calls or unsafe code.
- `pulse-win32` includes pure immersive window policy for normal versus fullscreen suppression without fighting exclusive fullscreen surfaces.
- `pulse-island-spike --window-policy` exports deterministic window-policy diagnostics.
- `pulse-win32` includes a pure `NativeWindowAdapterPlan` that composes compact style bits, visibility, placement, hit-test layout, and Command Palette hotkey registration into a stable per-frame native adapter plan without touching Win32 APIs.
- `pulse-island-spike --adapter-plan-policy` exports deterministic native-adapter-plan diagnostics proving visible and immersive frames reuse the compact logical window, avoid activation, avoid hidden-window destruction, and avoid replaying missed animations.
- `pulse-win32` includes pure `NativeWindowAdapterState` accumulation so repeated visible frames and visible/immersive/visible transitions create one compact window generation, avoid destruction/recreation, avoid activation attempts, and register the Palette hotkey once without churn.
- `pulse-island-spike --adapter-state-policy` exports deterministic adapter-state diagnostics for the visible/immersive/visible frame sequence.
- `pulse-win32` includes pure `NativeWindowAdapterAction` diffs so the future adapter has deterministic ordered native operation sequences: create/style/update-hit-test/move/register/show/topmost on first visible frame, no actions on repeated identical frames, hide/clear-topmost under immersive suppression without destroy or hotkey churn, hit-test layout changes produce only `UpdateHitTestLayout`, placement changes produce only `MoveResize`, style changes produce only `ApplyWindowStyles`, and disabled hotkey policy produces only `UnregisterHotkey`.
- `pulse-win32` includes a safe `NativeWindowAdapterDriver` and `NativeWindowActionSink` seam that applies ordered action diffs through a backend and advances pure state only after all sink actions succeed.
- `pulse-win32` includes payload-bearing `NativeWindowAdapterCommand` diffs and a `NativeWindowCommandSink` seam so the future HWND backend receives style bits, hit-test layout, placement, and hotkey payloads, not just action names.
- `pulse-win32-hwnd` is in the workspace as the explicit HWND unsafe boundary crate; it contains safe preflight logic plus an MSVC-only `windows-sys` native API adapter.
- `pulse-win32-hwnd` includes `HwndCommandPreflightSink`, which validates ordered backend commands before native calls, rejects commands requiring a missing compact window, rejects duplicate compact-window creation, rejects non-positive placement sizes, rejects invalid hotkey payloads, and records only validated commands.
- `pulse-win32-hwnd` includes `HwndNativeBackend`, `HwndCompactWindowFactory`, `HwndNativeApi`, and `RawHwnd` so payload commands can reach native APIs only after preflight and HWND creation.
- `pulse-win32-hwnd` includes `WindowsSysHwndApi` on MSVC targets, wrapping `SetWindowLongPtrW`, `SetWindowPos`, `ShowWindow`, `RegisterHotKey`, and `UnregisterHotKey` behind the safe `HwndNativeApi` trait.
- `pulse-win32-hwnd` includes `CompactWindowClassSpec` and `WindowsSysCompactWindowFactory` on MSVC targets, registering a content-free class, calling `CreateWindowExW` for a hidden popup/toolwindow/no-activate compact HWND, and destroying it with `DestroyWindow`.
- `pulse-win32-hwnd` includes `HwndMessagePumpBudget`, `HwndMessagePumpReport`, and MSVC-only `WindowsSysMessagePump`, which drains pending compact-HWND messages with bounded nonblocking `PeekMessageW(PM_REMOVE)`, `TranslateMessage`, and `DispatchMessageW`.
- `pulse-win32-hwnd` includes `HwndHitTestBridge`; the MSVC WndProc handles `WM_NCHITTEST` by converting screen coordinates with `ScreenToClient` and returning pure `Win32HitTestCode` values from cached content-free geometry.
- `pulse-win32-hwnd` handles the compact HWND activation edge: MSVC WndProc returns `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`, verified against a real hidden HWND.
- `pulse-win32-hwnd` handles content-free compact mouse input dispatch: `HwndMouseInputBridge` maps only client-body `WM_LBUTTONUP` to `CompactPrimaryClick`; transparent margin, drag grip, and outside points dispatch nothing; MSVC WndProc tests verify the queue against a real hidden HWND.
- `pulse-win32-hwnd` handles content-free paint readiness dispatch: `HwndPaintBridge` maps `WM_PAINT` to `CompactRepaintRequested`; MSVC WndProc tests verify the render queue against a real hidden HWND and validate the paint region.
- Direct2D/DirectComposition drawing has not been wired yet.
- `pulse-island-spike --adapter-action-policy` exports deterministic adapter-action diagnostics.
- `pulse-island-spike --adapter-replay-policy` drives S7 rapid state changes and S8 immersive mock scenarios through the pure native adapter plan/action/state layers, proving one window generation, one create, zero destroy, zero activation, one hotkey registration, no hotkey unregister, S7 bounded action budget (`total_actions=9`, `max_actions_per_frame=7`), and immersive hide/clear-topmost actions.
- `pulse-island-spike --w2-review-ready` exports a machine-checkable W2 review readiness summary: `w2_review_ready=true`, `gate_audit=21/21`, `adapter_readiness=plan,state,action,replay`, scope remains mocked `PresentationPlan`, and `w3_ready=true`.
- `pulse-island-spike --w2-review-manifest` exports a stable machine-checkable W2 review contract: manifest version, W2 package/scope/status, Gate A checklist/evidence counts, adapter readiness, evidence doc, zero forbidden dependency hits, and W3 authorized scope.
- `pulse-island-spike --architecture-policy` exports deterministic W2 architecture diagnostics: the UI/spike/win32/hwnd-boundary manifests have zero forbidden provider-adapter, SQLite, browser/WebView/Tauri/Electron dependency hits, the mock `PresentationPlanSource` seam remains replaceable, the HWND boundary manifest is present, and the HWND WndProc covers hit-test, no-activate mouse activation, content-free mouse dispatch, and content-free paint dispatch.
- `pulse-island-spike --gate-audit` aggregates the 21 Spike A acceptance checklist items into functional/window/performance/architecture counts, keeps scope explicit as mock `PresentationPlan` input only, and reports `w3_ready=true w2_review=accepted`.
- W2 gate evidence is tracked in `docs/pulse-island/W2-GATE-AUDIT.md`, currently accepted for W3 start.

## W3 evidence implemented

- `pulse-link-core` is in the workspace for provider-neutral Spike C Link/Shim primitives.
- `pulse-link-core` includes a content-free local frame header with fixed magic/version/message-kind/request-id/payload-length fields.
- `pulse-link-core` enforces Spike C payload caps before payload parsing: 8 KiB Hook ingress, 8 KiB Island control, and 128 KiB full snapshot.
- `pulse-link-shim` is in the workspace as the short-lived W3 Shim app.
- `pulse-link-shim` enforces Safe Mode at the earliest Pulse-owned boundary: Safe Mode performs no wake and no forwarding.
- `pulse-link-shim` preflights bounded input and exits success for oversized/forbidden/malformed input or delivery failure.
- `pulse-link-core` includes the pure Spike C lifecycle state model for wake, warm, active, Island-active, Drop Mode, grace, checkpoint, and exit transitions.
- `pulse-persistence` is in the workspace with a bounded breadcrumb abstraction: active cap 128, recent-terminal cap 20, per-task cap 1 KiB, total snapshot cap 256 KiB, lifecycle bucket validation, and complete-replacement in-memory checkpointing.
- `pulse-persistence` includes a file-backed atomic breadcrumb store: missing snapshot files load as empty state, checkpoint writes complete replacement snapshots through a same-directory temp file, the temp file is removed after successful replacement, oversized checkpoints are rejected before replacing the previous valid snapshot, and the stored file remains compact/content-minimized rather than append-only raw events.
- `pulse-link-core` includes a fake Island protocol session for Hello/GetSnapshot/Subscribe, monotonic SnapshotDelta delivery, and revision-gap full-snapshot recovery before real named pipes.
- `pulse-win32` includes content-free Link local object name derivation for mutex, ingress pipe, Island pipe, and ready event without exposing raw install id, raw user SID, or raw logon session.
- `pulse-win32` includes a pure single-instance ownership model for first owner, existing owner reuse, bounded stale mutex/pipe retries, fail-open stale exhaustion, and per-logon-session scoping.
- `pulse-link-core` includes an initial handoff launch plan proving the first event goes through inherited stdin, not command-line arguments, environment variables, or temporary file names.
- `pulse-win32-link` is in the workspace as the W3 Link transport unsafe-boundary crate. It includes safe preflight for mutex, ingress pipe, Island pipe, and inherited handoff pipe setup; a native backend executor that advances state only after preflight and native success; and an MSVC-only `windows-sys` adapter for `CreateMutexW`, `CreateNamedPipeW`, `CreatePipe`, and `CloseHandle`.
- `pulse-win32-link` includes native shutdown cleanup: handoff write/read handles, Island pipe, ingress pipe, and mutex close in reverse ownership order; successful closes are cleared from state; failed closes are retained for retry diagnostics.
- `pulse-win32-link` includes Island client pipe connection preflight and native connection seam: an Island client can connect only after the Island pipe server exists, duplicate client connection is rejected, and the MSVC adapter uses `CreateFileW` for the scoped Island pipe.
- `pulse-win32-link` includes an MSVC-only OS-backed transport smoke harness: real scoped mutex, ingress pipe, Island pipe, inherited handoff pipe, fake Island client connection, and cleanup of all six owned handles.
- `pulse-win32-link` includes an MSVC-only real ingress named-pipe frame/ack harness: client writes a frame to the ingress server, server reads exact bytes, server writes a one-byte ack, client reads it back, and all three owned handles close.
- `pulse-link` includes an MSVC-only OS-backed ingress reducer harness: the real ingress frame/ack harness accepts a content-free Hook frame header, drives a synthetic reducer event, writes a complete replacement checkpoint, and closes all native handles.
- `pulse-win32-link` and `pulse-link` include an MSVC-only OS-backed multi-frame ingress reducer loop: multiple frames are acknowledged on one named-pipe connection, malformed headers are rejected before reducer mutation, later valid frames still reduce/checkpoint, and all native handles close.
- `pulse-win32-link` and `pulse-link-spike-client` include an MSVC-only OS-backed Island protocol loop: real Island pipe request/response bytes are bound to the fake Island Hello, snapshot, subscribe, monotonic delta, and revision-gap recovery sequence, with native handle cleanup.
- `pulse-link` includes an MSVC-only OS-backed C8 residue harness: native Link transport starts, terminal synthetic reducer state enters the 90-second grace driver, final checkpoint stops Link, mutex/ingress/Island handles close, and a short-lived child process exits with zero child residue.
- `pulse-link` includes an MSVC-only OS-backed Spike C C0-C9 aggregate harness: all ten Spike C scenarios are covered, transport-specific paths are tied to real mutex/named-pipe/handoff/residue evidence, fail-open/provider-neutral behavior is preserved, and OS-backed slices retain zero native handles.
- `pulse-link-shim` wires `pulse-win32-link` behind a provider-neutral native Shim seam: existing Link ingress is reused when acknowledged, first wake creates an inherited handoff pipe, native handoff setup failure remains fail-open, and the payload is not exposed through argv/env/temp metadata.
- `pulse-link` wires `pulse-win32-link` behind a provider-neutral native startup seam: Link creates/acquires the scoped mutex, ingress pipe server, and Island pipe server before reporting ready.
- `apps/pulse-link` has a native transport runtime seam that retains startup handles and invokes `pulse-win32-link` shutdown cleanup after the final checkpoint.
- `apps/pulse-link` is in the workspace with a pure synthetic runner connecting admitted events, reducer output, lifecycle transitions, and in-memory breadcrumbs.
- `apps/pulse-link` runtime now accepts an injected `BreadcrumbStore`, and the contract tests cover file-backed checkpoint plus restart recovery as degraded state.
- `apps/pulse-link` includes a Drop Mode grace driver with Spike C fixed 90-second deadline, caller-owned clock, new-event grace cancellation, final checkpoint write at expiry, and C8 scenario coverage through the driver instead of manual lifecycle forcing.
- `apps/pulse-link-spike-client` is in the workspace with a fake Island client wrapper for attach and delta receipt before pipe transport.
- `apps/pulse-link-spike-client` now has a native Island pipe connection seam before the pure fake session attach flow.
- `apps/pulse-link-spike-client` now has a pipe-backed fake Island message loop seam that requires an Island pipe connection before startup and handles Hello, snapshot, subscription, monotonic delta, and revision-gap recovery.
- `apps/pulse-link` includes a pure synthetic C0-C9 scenario harness covering existing delivery, first wake, parallel race, unavailable Link, malformed/oversized ingress, Drop Mode breadcrumb, Island attach/detach/reattach, restart recovery as degraded, grace exit, and bounded event storm. C1/C2 now use the pure ownership/handoff models.
- `docs/pulse-island/W3-GATE-AUDIT.md` tracks current W3 evidence and remaining Spike C work.

## W4 evidence implemented

- `pulse-island-spike --gate-audit` now reports `w4_ready=true w3_review=accepted` and `active_work=W4_Provider_Probe_Harness`.
- `pulse-island-spike --provider-probe-manifest` exports the W4 read-only capability-discovery manifest for Codex CLI, Claude Code, and Antigravity, with all providers at `not_probed`.
- `pulse-island-spike --provider-probe-report=<provider>` exports a sanitized read-only provider inventory report with all capability rows at `not_probed`, no raw provider content, no raw provider configuration, and no support labels.
- `pulse-island-spike --provider-config-transaction-fixture=<provider>` exports a synthetic user-config transaction fixture proving install/update/uninstall reporting without reading or writing real provider config.
- `pulse-island-spike --provider-probe-scorecard` exports a no-selection scorecard scaffold: no provider has probe evidence, no first adapter is selected, and all providers remain `not_probed`.
- `pulse-island-spike --provider-resource-fixture=<provider>` exports synthetic resource measurement categories only, with `resource_budget_claim=not_measured`.
- `pulse-island-spike --provider-evidence-register=<provider>` exports sanitized evidence-register summaries without raw source locations, provider content, provider configuration, or capability claims.
- `pulse-island-spike --provider-official-evidence-source-locator=<provider>` exports the official source locator scaffold required by `15-provider-capability-probe.md`: source type, redacted source location ID, publication/update date, tested provider version, supported capability claim, and known constraints, without raw source locations or raw documentation.
- `pulse-island-spike --provider-probe-summary-fixture=<provider>` exports sanitized probe-result summaries without raw payload retention.
- `pulse-island-spike --provider-probe-scorecard=sanitized-fixture` scores sanitized summaries while keeping `first_adapter_candidate=none` and `w5_adapter_creation_authorized=false`.
- `pulse-island-spike --provider-hard-disqualifiers=sanitized-fixture` evaluates hard gates and keeps W5 blocked when full evidence is missing.
- `pulse-island-spike --provider-evidence-gap-summary` aggregates missing direct evidence gates across Codex CLI, Claude Code, and Antigravity. Current output reports 21 missing direct gates, `w4_complete=false`, and `w5_start_allowed=false`.
- `pulse-island-spike --provider-probe-audit` exports the aggregate W4 scaffold/audit status, including `read_only_local_probe_run=present`, `read_only_local_probe_summary=present`, `read_only_resource_measurement_plan=present`, `probe_card_execution_plans=present`, `evidence_retention_policy=present`, `live_authorization_preflight=blocking`, `missing_capability_rationale=present`, `release_decision_logs=present`, `official_evidence_source_locators=scaffold_only`, `direct_gate_packets=present`, `direct_evidence_import_checklist=present`, `authorized_evidence_runbooks=present`, `sanitized_evidence_output_template=present`, `sanitized_evidence_bundle_validator=present`, `release_elevation_preflight=blocking`, `w5_observe_adapter_contract=scaffold_only`, `w4_completion_gate=blocking`, and `next_work=direct_gate_evidence_when_authorized`.
- `pulse-island-spike --provider-surface-inventory` exports Probe Card declared provider surfaces without live claims.
- `pulse-island-spike --provider-probe-readiness` exports scaffold readiness plus remaining live probe/W5 gates, Probe Card execution plan readiness, evidence retention policy readiness, live authorization preflight readiness, missing-capability rationale readiness, release decision log readiness, official evidence source locator readiness, direct gate packet readiness, direct evidence import checklist readiness, authorized evidence runbook readiness, sanitized evidence output template readiness, sanitized evidence bundle validator readiness, release elevation preflight readiness, W5 observe adapter contract scaffold readiness, W4 completion gate readiness, and `next_allowed_work=collect_direct_gate_evidence_when_authorized`.
- Read-only local CLI preflight on 2026-07-08 found Codex CLI `0.142.4`, Claude Code `2.1.195`, and no `antigravity` command on PATH. This is environment-manifest evidence only, not capability support.
- `pulse-island-spike --provider-local-environment-manifest=read-only-fixture` exports the sanitized local environment manifest without paths, config, account data, network, or support claims.
- `pulse-island-spike --provider-live-probe-dry-run=read-only-fixture` exports the next read-only probe actions without executing provider tasks.
- `pulse-island-spike --provider-live-probe-run=read-only-local` executes approved local `--version` category checks and exports only sanitized environment categories. It retains no raw versions, command paths, config, account data, transcript/session data, support labels, winner, or W5 authorization.
- `pulse-island-spike --provider-live-probe-summary=read-only-local` converts the read-only local probe run into sanitized evidence summaries without capability elevation.
- `pulse-island-spike --provider-resource-measurement-plan=read-only-local` exports a planned-only resource measurement matrix without starting providers or measuring resource budgets.
- `pulse-island-spike --provider-probe-card-execution-plan=<provider>` exports Probe Cards as phase-based execution plans without running provider tasks.
- `pulse-island-spike --provider-evidence-retention-policy` exports the W4 repo/local/never-retain evidence policy.
- `pulse-island-spike --provider-live-authorization-preflight=<provider>` exports the default blocked live-action preflight.
- `pulse-island-spike --provider-missing-capability-rationale=<provider>` explains absent capabilities without blank cells or release elevation.
- `pulse-island-spike --provider-release-decision-log=<provider>` exports sanitized W4 release decision logs that defer provider selection until direct evidence exists.
- `pulse-island-spike --provider-capability-matrix=<provider>` exports the full `15-provider-capability-probe.md` capability matrix scaffold. All 15 template capabilities are populated with `evidence_source=missing`, `probe_result=not_probed`, `identity_strength=none`, `health_ceiling=unavailable`, `release=not_probed`, and no raw provider content/configuration.
- `pulse-island-spike --provider-release-label-evaluation=sanitized-fixture` evaluates release-label eligibility from sanitized fixture state only. All elevated labels remain ineligible until direct gates are present; W5 remains blocked.
- `pulse-island-spike --provider-probe-phase-status=<provider>` exports the P0-P8 probe phase status matrix. P0 official-surface inventory is read-only scaffolded; P1-P8 remain not executed and require authorization.
- `pulse-island-spike --provider-direct-gate-packet=<provider>` exports per-provider direct evidence packets without executing live provider actions.
- `pulse-island-spike --provider-direct-evidence-import-checklist=<provider>` exports the direct-evidence import checklist. It requires authorized local artifacts for each direct gate, rejects sanitized fixtures and read-only version evidence as release-elevating inputs, rejects raw provider content/configuration, and keeps W5 blocked.
- `pulse-island-spike --provider-authorized-evidence-runbook=<provider>` exports the authorized direct-evidence runbook scaffold. It defines manual authorized steps for synthetic workspace preparation, local-only provider config backup, Probe Card phase execution, direct gate artifact collection, required redaction, and release-elevation preflight while executing no provider action and keeping W5 blocked.
- `pulse-island-spike --provider-sanitized-evidence-output-template=<provider>` exports the sanitized evidence output template for authorized direct evidence after redaction. It declares the only repo-safe artifacts and keeps raw prompts/transcripts, raw provider configuration, customer source, and credentials out of the repository.
- `pulse-island-spike --provider-sanitized-evidence-bundle-validator=<provider>` exports the sanitized bundle validator scaffold. It requires repo-safe artifacts, rejects raw prompts/transcripts/provider config/customer source/credentials/terminal buffers/private endpoint traffic, does not import artifacts, and keeps direct evidence plus W5 authorization unclaimed.
- `pulse-island-spike --provider-release-elevation-preflight=<provider>` exports the per-provider release elevation preflight for `supported_observe`. Current output keeps the provider at `not_probed`, reports direct evidence missing and hard disqualifiers uncleared, and blocks release elevation plus W5 adapter creation.
- `pulse-island-spike --provider-w5-observe-adapter-contract=<provider>` exports a W5 observe adapter contract scaffold without creating an adapter. It requires a future `supported_observe` release label, lists the only allowed observe capabilities, excludes external session control, approval UI, transcript/history parsing, raw-prompt task titles, exact route claims without exact evidence, terminal claims without terminal evidence, and unscoped Fuel sources, and keeps `w5_adapter_creation_authorized=false`.
- `pulse-island-spike --provider-w4-completion-gate` exports the W4 completion/W5 start gate and keeps W5 blocked until direct evidence exists.
- `pulse-island-spike --provider-w5-start-preflight` exports the W5 start decision preflight. Current output keeps `w4_complete=false`, `w5_start_allowed=false`, `selected_provider=none`, and `w5_adapter_creation_authorized=false` until a provider reaches `supported_observe` through direct evidence.
- `docs/pulse-island/W4-PROBE-HARNESS-AUDIT.md` tracks current W4 evidence and remaining W4 work.

## Checks last run

- PASS: `cargo fmt --check`
- PASS: `cargo test --workspace`
- PASS: `cargo clippy --workspace --all-targets`
- PASS: `cargo tree --workspace --depth 1`
- PASS: `cargo run -p pulse-island-spike -- S7 --replay`
- PASS: `cargo run -p pulse-island-spike -- --focus-policy`
- PASS: `cargo run -p pulse-island-spike -- --truth-priority-policy`
- PASS: `cargo run -p pulse-island-spike -- --palette-policy`
- PASS: `cargo run -p pulse-island-spike -- --layout-policy`
- PASS: `cargo run -p pulse-island-spike -- --route-policy`
- PASS: `cargo run -p pulse-island-spike -- --lifecycle-policy`
- PASS: `cargo run -p pulse-island-spike -- --surface-handle-policy`
- PASS: `cargo run -p pulse-island-spike -- --hit-test-policy`
- PASS: `cargo run -p pulse-island-spike -- --hotkey-policy`
- PASS: `cargo run -p pulse-island-spike -- --dpi-policy`
- PASS: `cargo run -p pulse-island-spike -- --cache-policy`
- PASS: `cargo run -p pulse-island-spike -- --animation-policy`
- PASS: `cargo run -p pulse-island-spike -- --resource-policy`
- PASS: `cargo run -p pulse-island-spike -- --accessibility-policy`
- PASS: `cargo run -p pulse-island-spike -- --measurement-policy`
- PASS: `cargo run -p pulse-island-spike -- --overlay-policy`
- PASS: `cargo run -p pulse-island-spike -- --window-policy`
- PASS: `cargo run -p pulse-island-spike -- --adapter-plan-policy`
- PASS: `cargo run -p pulse-island-spike -- --adapter-state-policy`
- PASS: `cargo run -p pulse-island-spike -- --adapter-action-policy`
- PASS: `cargo run -p pulse-island-spike -- --adapter-replay-policy`
- PASS: `cargo run -p pulse-island-spike -- --w2-review-ready`
- PASS: `cargo run -p pulse-island-spike -- --w2-review-manifest`
- PASS: `cargo run -p pulse-island-spike -- --architecture-policy`
- PASS: `cargo run -p pulse-island-spike -- --gate-audit`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32'` ran 28 `pulse-win32` tests, including the two `windows-sys` parity tests.
- PASS: `cargo test -p pulse-win32-hwnd` ran 10 preflight/native-backend/factory-spec/pump-report/hit-test-bridge/mouse-input-bridge/paint-bridge boundary tests.
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32-hwnd'` ran 16 `pulse-win32-hwnd` tests, compiled the MSVC-only `windows-sys` adapter, created/destroyed a real hidden compact HWND, drained a posted `WM_NULL` without blocking, verified real WndProc `WM_NCHITTEST` results, verified `WM_MOUSEACTIVATE -> MA_NOACTIVATE`, verified content-free `WM_LBUTTONUP` dispatch, and verified content-free `WM_PAINT` dispatch.
- PASS: `cargo test -p pulse-win32 native_adapter_driver`
- PASS: `cargo test -p pulse-win32 native_adapter_commands_carry_backend_payloads_for_visible_plan`
- PASS: `cargo test -p pulse-link-core`
- PASS: `cargo test -p pulse-link-shim`
- PASS: `cargo test -p pulse-persistence --test breadcrumb_contract`
- PASS: `cargo test -p pulse-link --test link_runner_contract`
- PASS: `cargo test -p pulse-link --test link_scenarios_contract`
- PASS: `cargo test -p pulse-link-shim --test shim_fail_open`
- PASS: `cargo test -p pulse-link --test link_native_transport_contract`
- PASS: `cargo test -p pulse-win32-link --test link_transport_contract`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32-link windows_sys_os_transport_harness_creates_pipe_mutex_handoff_and_cleans_up'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32-link windows_sys_ingress_pipe_round_trips_frame_header_and_ack'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_ingress_frame_ack_drives_reducer_and_checkpoint'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_ingress_frame_ack_loop_rejects_bad_frame_without_stopping_reducer'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_grace_exit_closes_transport_and_leaves_no_child_residue'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_c0_c9_harness_covers_all_spike_c_scenarios'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link-spike-client windows_sys_island_pipe_protocol_loop_handles_attach_delta_and_gap'`
- PASS: `cargo test -p pulse-link-spike-client --test spike_client_contract` ran 6 tests including pipe message-loop startup, attach sequence, delta delivery, and revision-gap recovery.
- PASS: `cargo test -p pulse-island-spike cli_exports_w2_review_manifest_as_machine_checkable_contract`
- PASS: `cargo test -p pulse-island-spike w4_`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-manifest`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-report=codex_cli`
- PASS: `cargo run -p pulse-island-spike -- --provider-config-transaction-fixture=claude_code`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-scorecard`
- PASS: `cargo run -p pulse-island-spike -- --provider-resource-fixture=codex_cli`
- PASS: `cargo run -p pulse-island-spike -- --provider-evidence-register=codex_cli`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-summary-fixture=claude_code`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-scorecard=sanitized-fixture`
- PASS: `cargo run -p pulse-island-spike -- --provider-hard-disqualifiers=sanitized-fixture`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_evidence_gap_summary_across_providers_without_live_actions`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-evidence-gap-summary`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-audit`
- PASS: `cargo run -p pulse-island-spike -- --provider-surface-inventory`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-readiness`
- PASS: `cargo run -p pulse-island-spike -- --provider-local-environment-manifest=read-only-fixture`
- PASS: `cargo run -p pulse-island-spike -- --provider-live-probe-dry-run=read-only-fixture`
- PASS: `cargo run -p pulse-island-spike -- --provider-live-probe-run=read-only-local`
- PASS: `cargo run -p pulse-island-spike -- --provider-live-probe-summary=read-only-local`
- PASS: `cargo run -p pulse-island-spike -- --provider-resource-measurement-plan=read-only-local`
- PASS: `cargo run -p pulse-island-spike -- --provider-probe-card-execution-plan=claude_code`
- PASS: `cargo run -p pulse-island-spike -- --provider-evidence-retention-policy`
- PASS: `cargo run -p pulse-island-spike -- --provider-live-authorization-preflight=codex_cli`
- PASS: `cargo run -p pulse-island-spike -- --provider-missing-capability-rationale=claude_code`
- PASS: `cargo run -p pulse-island-spike -- --provider-release-decision-log=codex_cli`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_full_capability_matrix_without_blank_or_elevated_rows`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-capability-matrix=codex_cli`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_release_label_evaluation_without_elevating_sanitized_fixture`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-release-label-evaluation=sanitized-fixture`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_probe_phase_status_without_executing_live_phases`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-probe-phase-status=claude_code`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_w5_start_preflight_that_blocks_adapter_creation_without_direct_evidence`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_direct_evidence_import_checklist_without_accepting_weak_evidence`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_sanitized_evidence_output_template_without_raw_artifacts`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_release_elevation_preflight_without_promoting_provider`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_w5_observe_adapter_contract_without_creating_adapter`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_authorized_evidence_runbook_without_executing_provider_actions`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_sanitized_evidence_bundle_validator_without_importing_artifacts`
- PASS: `cargo test -p pulse-island-spike cli_exports_w4_official_evidence_source_locator_without_raw_docs_or_claims`
- PASS: `cargo test -p pulse-island-spike w4_` ran 36 W4 tests.
- PASS: `cargo run -p pulse-island-spike -- --provider-direct-gate-packet=codex_cli`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-direct-evidence-import-checklist=claude_code`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-official-evidence-source-locator=codex_cli`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-authorized-evidence-runbook=codex_cli`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-sanitized-evidence-output-template=codex_cli`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-sanitized-evidence-bundle-validator=claude_code`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-release-elevation-preflight=codex_cli`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-w5-observe-adapter-contract=claude_code`
- PASS: `cargo run -p pulse-island-spike -- --provider-w4-completion-gate`
- PASS: `cargo run -q -p pulse-island-spike -- --provider-w5-start-preflight`
- PASS: stale-boundary scan across `.ai-bridge` and `docs/pulse-island` found no legacy-boundary markers and no stale W4 25-35 test-count text.
- PASS: read-only `Get-Command codex, claude, antigravity`
- PASS: read-only `codex --version`
- PASS: read-only `claude --version`
- INFO: read-only `antigravity --version` not found on PATH
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo check -p pulse-link-shim -p pulse-link'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo check -p pulse-persistence -p pulse-link'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo check -p pulse-link'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo check -p pulse-win32-link -p pulse-link'`
- PASS: `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo check -p pulse-win32-link -p pulse-link-spike-client'`
- INFO: `windows-window v0.0.0` was probed as a possible safe wrapper but does not expose the documented `Window` API; it is not used.

## Next work

1. Collect direct W4 gate evidence only when explicitly authorized: live provider probe execution, live resource measurement, install/rollback real fixture, Late Attach real result, and terminal-truth real result.
2. Keep lifecycle, install/rollback, Late Attach, terminal truth, and resource-budget elevation gated until each has direct evidence.
3. Keep provider adapters, live Hook installation, provider config mutation, and production route activation gated until W4/W5 evidence explicitly authorizes them.
