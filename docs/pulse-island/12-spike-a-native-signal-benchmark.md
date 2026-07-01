# Pulse Island · Spike A: Native Signal Benchmark

**Status:** Executable spike plan  
**Goal:** Validate the native Windows signal experience and resource envelope before provider integration  
**Depends on:** `09-native-island-ui-system.md`, `11-rust-workspace-architecture.md`, `10-verification-gates-and-mvp-roadmap.md`  
**Last updated:** 2026-07-01

---

## 1. Spike question

Can a pure native Rust Windows application render the Pulse Island Signal → Peek → Focus Card interaction with polished compositor-driven motion, correct click/focus behavior, and strict resource use, before any real Codex/Claude/Antigravity adapter is introduced?

This spike must answer with measurements, not an attractive screenshot.

---

## 2. Success outcome

At the end of Spike A, the team can confidently say:

> The native UI shell is capable of carrying Pulse Island's core attention model without a browser runtime, without focus theft, and without breaking the memory/CPU budget.

Spike A does **not** prove provider integration, Late Attach, Hook transport, quota reading, or actual task control.

---

## 3. Strict scope

### In scope

- Rust workspace baseline for UI-facing crates
- compact Win32 island window
- mocked `PresentationPlan` input
- state glyphs: running, waiting, failed, completed, degraded, aggregate active
- mock Fuel Thread
- Peek with up to three mock task rows
- Focus Card with mock route labels
- Command Palette shell and global shortcut
- per-monitor DPI handling
- topmost/non-activating window behavior
- transparent click-through regions and real interactive hit zones
- DirectComposition-driven entrance/state/expansion motion
- reduced-motion and high-contrast behavior
- fullscreen/presentation hiding proof
- automated or semi-automated measurements for latency, CPU, memory, handles, and GPU usage

### Explicitly out of scope

- Pulse Link
- named pipes
- SQLite
- real provider adapters
- Codex/Claude/Antigravity files or Hooks
- token/usage scanning
- Context Router launching actual terminal/IDE windows
- approval/permission controls
- notifications
- cloud services

No provider code should appear in Spike A.

---

## 4. Deliverable

The spike delivers one runnable binary plus a repeatable scenario runner.

```text
pulse-island-spike.exe
├── Mock scenario source
├── Native island window
├── Peek / Focus Card / Palette
├── Performance overlay available only in diagnostics mode
└── Measurement export
```

The executable is intentionally disposable at the product-logic level, but its Win32/D3D/DirectComposition foundations should be reusable by `pulse-island-ui`.

---

## 5. Minimum crate slice

Create only the smallest crates necessary for this proof:

```text
pulse-domain
pulse-arbitration (mock-friendly PresentationPlan types only if needed)
pulse-win32
pulse-island-ui
pulse-testkit
apps/pulse-island-spike
```

### 5.1 Temporary mock seam

The UI consumes a `PresentationPlanSource` trait:

```text
trait PresentationPlanSource {
    fn current_plan(&self) -> PresentationPlan;
    fn subscribe(&self, callback: PlanChangedCallback);
}
```

Spike A supplies `MockPresentationPlanSource`.

The eventual Island app will supply a pipe-backed implementation. This prevents mock behavior from leaking into UI layout or animation logic.

---

## 6. Scenario catalog

Each scenario is deterministic and can run by command-line argument or diagnostics palette command.

### S0: Idle / parked

```text
No active task
→ Island hidden or parked
→ no animation
→ no continuous draw loop
```

### S1: One running task

```text
● Codex · Running tests
```

Expected:

- restrained breathing only through compositor
- no repeated CPU-heavy redraw
- click opens Peek

### S2: Waiting task with background work

```text
! Claude · Needs confirmation · +2 active
```

Expected:

- yellow attention treatment
- one clear non-color textual reason in Peek/Focus
- no Toast work in this spike

### S3: Failed task

```text
× Codex · Build failed
```

Expected:

- brief two-step attention transition then stable state
- error reason visible in Focus Card
- no focus theft

### S4: Aggregate active work

```text
● 3 agents working · Fuel low
```

Expected:

- aggregate subject, not a rotating individual task
- Fuel Thread is visible but visually secondary

### S5: Degraded / observed state

```text
○ Claude · Status unavailable
```

Expected:

- muted treatment
- honest route-label mock, such as `Open workspace`
- no error-like red state

### S6: Completion settle-out

```text
✓ Codex · Completed
```

Expected:

- brief confirmation
- clears after configured grace period
- instant displacement by waiting/failed mock state

### S7: Rapid state changes

Sequence:

```text
running → waiting → running → failed → completed → idle
```

Expected:

- no window recreation
- no focus change
- transitions interrupt cleanly
- no unbounded GPU/CPU/memory growth

### S8: Immersive mode simulation

Simulate fullscreen/presentation policy.

Expected:

- Island hides
- animation stops
- state model remains intact
- restored state returns without replay animation storm

---

## 7. Window and input proof

### 7.1 Hit testing

Verify `WM_NCHITTEST` behavior:

| Region | Expected result |
|---|---|
| transparent margin | `HTTRANSPARENT` / pass through |
| body | interactive client area |
| configured drag grip | `HTCAPTION` or explicit drag behavior |
| button/row | interactive client area |

The test must demonstrate that an underlying app can receive clicks through transparent padding while the Island remains interactive where it visibly offers interaction.

### 7.2 Focus behavior

Test:

- state transition does not activate Island
- passive hover does not activate Island
- click on compact island may open Peek without changing focus unless input is required
- Command Palette takes focus only from user shortcut/action
- Escape closes focused surface and returns normal foreground behavior

### 7.3 Topmost behavior

Test normal windows, maximized windows, multiple monitors, and a simulated fullscreen policy. The spike must not attempt to sit above exclusive/fullscreen surfaces when policy says hide.

---

## 8. Rendering and animation proof

### 8.1 Render model

- one D3D11 device
- DirectComposition owns state-transition property animation
- Direct2D draws compact scene primitives
- DirectWrite lays out short text only
- no app-side unconditional 16 ms timer

### 8.2 Required animation checks

| Behavior | Required check |
|---|---|
| compact arrival | no visible window flash / black frame |
| running breath | compositor-driven and low amplitude |
| waiting pulse | bounded, no alert-storm flash |
| failure transition | settles to stable state, does not pulse forever |
| Peek expansion | interruptible when state changes or Escape pressed |
| Focus Card | no full scene reallocation on row selection |
| reduced motion | removes pulsing and scale movement |

### 8.3 Visual restraint

The benchmark is not a demo reel. No particle systems, glow storms, noisy graphs, animated gradients, or game-HUD effects are permitted.

---

## 9. Accessibility proof

### 9.1 Required

- state is legible without color
- keyboard navigation works in Palette and Peek
- high-contrast mode has usable text/background separation
- Windows text scaling and per-monitor DPI do not clip the state glyph or route action
- Reduced Motion changes animation policy

### 9.2 Manual review matrix

Review at minimum:

```text
100% DPI / 150% DPI / 200% DPI
light-compatible system environment / dark default / high contrast
reduced motion on / off
single monitor / two monitors with different scaling
```

The initial visual style can be dark-first, but high contrast must remain functional rather than merely not crashing.

---

## 10. Performance benchmark plan

### 10.1 Measurement method

The benchmark harness records process-tree metrics at fixed scenario milestones. It should collect at least:

- working set
- private bytes
- committed bytes where available
- CPU time deltas
- process and thread count
- GDI/User handle count where relevant
- D3D allocation count/estimated usage when available
- state-update-to-present latency
- shortcut-to-first-palette-frame latency

Measurements are diagnostic metadata only. They contain no task content.

### 10.2 Test run structure

For each scenario:

```text
warm up
→ run fixed interaction sequence
→ hold stable state for 30 seconds
→ capture metrics
→ repeat enough times for P95
→ export aggregate report
```

### 10.3 Performance gates

| Metric | Required target |
|---|---:|
| compact idle P95 memory | <= 45 MB |
| Focus Card P95 memory | <= 85 MB |
| hard process-tree ceiling | < 100 MB |
| idle average CPU | <= 0.10% |
| running-state average CPU | <= 0.35% |
| state update to visible response P95 | <= 120 ms |
| palette shortcut to first frame P95 | <= 80 ms |
| 30-minute steady-state memory growth | <= 2 MB for spike shell |
| static state app-side frame loop | none |

If an acceptable visual effect violates these targets, it is redesigned rather than excused.

---

## 11. Test and acceptance checklist

### Functional

- [ ] All S0–S8 scenarios render deterministically.
- [ ] Signal → Peek → Focus Card navigation works.
- [ ] Compact Island never timer-rotates tasks.
- [ ] Fuel Thread is secondary to task signal.
- [ ] Mock route labels preserve `Open workspace` versus `Open original task` distinction.
- [ ] Escape and global shortcut behavior is predictable.

### Window behavior

- [ ] Transparent margins click through.
- [ ] Visible interaction zones receive input.
- [ ] Passive transitions do not steal focus.
- [ ] Alt+Tab does not show compact island.
- [ ] DPI/monitor changes preserve visible placement.
- [ ] Fullscreen policy hides Island.

### Performance

- [ ] All Gate A metrics pass.
- [ ] No permanent 60 Hz app-side timer exists.
- [ ] No resource growth across rapid state transitions.
- [ ] Handle counts remain stable after 1,000 open/close cycles of Peek and Focus Card.
- [ ] No D3D resource leak after 1,000 state transitions.

### Architecture

- [ ] UI has no provider dependencies.
- [ ] UI has no SQLite dependency.
- [ ] Mock plan source can be replaced by pipe-backed source without UI API change.
- [ ] No browser/WebView/Tauri/Electron dependency exists.

---

## 12. Exit decision

### Pass

Proceed to Gate B and keep the native UI foundation.

### Conditional pass

Proceed only if a documented, limited exception exists with a plan to reach the performance target before real adapters enter the UI.

### Fail

Do not add provider integrations. Diagnose one of:

- wrong composition/rendering strategy
- window/input model unsuitable for click-through plus interaction
- layout/animation design too expensive
- instrumentation unreliable

Redesign the shell before expanding scope.

---

## 13. Follow-on handoff

After Spike A passes, implementation work moves to:

```text
Gate B: State Kernel + Event Reduction fixtures
Gate C: Pulse Link transport + Drop Mode
```

The first real Adapter must not be started until both the visual shell and truth kernel have passing gates.

---

## 14. Design invariants

1. Spike A proves the shell, not the providers.
2. Mock data must exercise real presentation semantics, not arbitrary decorative animations.
3. Performance is part of acceptance, not a later optimization pass.
4. The initial UI architecture must be replaceable only at the edges, not through a future rewrite of the entire application.
