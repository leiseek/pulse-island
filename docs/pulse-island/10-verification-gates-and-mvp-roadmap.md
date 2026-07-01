# Pulse Island · Verification Gates and MVP Roadmap

**Status:** Execution baseline  
**Applies to:** Product sequencing, release criteria, adapter support labels, performance validation  
**Depends on:** All design documents `00` through `09`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Island has many attractive possible capabilities. The product succeeds only if it validates the smallest truthful loop before expanding:

```text
Agent starts normally
→ Pulse observes safe state
→ Island opens later
→ user sees correct attention signal
→ user can return to the best available original context
```

This roadmap is organized by proof gates, not by a feature wish list. A later phase cannot compensate for an earlier truth, privacy, performance, or late-attach failure.

---

## 2. Release vocabulary

| Label | Meaning |
|---|---|
| Spike | Technical proof; not user-ready. |
| Experimental | Explicit opt-in; known limitations displayed. |
| Supported | Passed all relevant capability, truthfulness, performance, and rollback gates. |
| P0 | Required MVP behavior. |
| P1 | Valuable enhancement after P0 proof. |
| Deferred | Not a release dependency. |

A provider is not “Supported” globally. Its individual capabilities are supported or experimental.

---

## 3. Gate 0: Product shell and baseline instrumentation

### Goal

Create a minimal native Rust workspace with measurable resource baselines before building rich behavior.

### Deliverables

- `pulse-island.exe` skeleton with Win32 window lifecycle
- `pulse-link.exe` skeleton with per-user single-instance guard
- local diagnostics mode
- process-tree memory and CPU measurement harness
- event fixture runner for state reducer
- no real provider integration yet

### Exit criteria

- Island can be shown/hidden without focus theft.
- Link starts and exits without agent interaction.
- Baseline measurement reports working set, private bytes, CPU, handles, and D3D resource counts where applicable.
- No WebView/browser runtime dependency exists.
- Empty process-tree resource use is inside an initial conservative envelope.

---

## 4. Gate A: Native Signal Benchmark

### Goal

Prove the signal UI, interaction model, and native rendering stack before provider complexity.

### Deliverables

- compact Signal surface
- mocked green/yellow/red/off/degraded states
- Peek with up to three mocked tasks
- Focus Card with mocked context route labels
- Command Palette opening through a global shortcut
- DirectComposition state transitions
- per-monitor DPI and multi-monitor behavior
- fullscreen/presentation hiding behavior

### Must prove

- transparent noninteractive regions click through
- interactive regions remain clickable
- compact island does not steal focus on state updates
- idle state has no continuous app-side redraw loop
- UI looks stable under rapid mocked state changes

### Exit criteria

| Measurement | Gate |
|---|---:|
| Shortcut to first Command Palette frame, P95 | <= 80 ms |
| State update to visible island change, P95 | <= 120 ms |
| Idle Island average CPU | <= 0.10% |
| Active light state average CPU | <= 0.35% |
| Idle Island P95 memory | <= 45 MB |
| Focus Card P95 memory | <= 85 MB |
| Process tree hard ceiling | < 100 MB |

No provider work begins until the native shell meets these bounds or has an explicit documented exception.

---

## 5. Gate B: State Kernel and Event Reducer

### Goal

Prove the normalized task model and Event Reduction Engine with deterministic fixtures.

### Deliverables

- `TaskRecord`, identity model, lifecycle/attention/context/health state
- normalized event schema
- six-stage reducer pipeline
- bounded breadcrumb store
- arbitration engine
- route-level labels
- fixture test suite

### Must prove

- same event sequence produces same snapshot result
- out-of-order activity cannot undo terminal event
- process quietness becomes stalled/watch, never false failed
- PID reuse cannot inherit stale task
- unknown cannot be rendered as attached
- event storms remain bounded

### Exit criteria

- all required reducer fixture families pass
- malformed/oversized event rejection is tested
- raw sensitive data cannot reach task snapshot/persistence/diagnostics
- active task breadcrumb can be restored after Link restart as degraded until revalidated

---

## 6. Gate C: Pulse Link Lifecycle and Hook Transport

### Goal

Prove on-demand Link wake-up, fail-open Hook handling, local IPC, and Drop Mode resource behavior.

### Deliverables

- `pulse-link-shim.exe`
- single-instance mutex and current-user named-pipe transport
- schema-versioned Hook envelope
- bounded startup and delivery deadlines
- integration configuration transaction helper
- Drop Mode breadcrumb persistence

### Must prove

- a Hook can wake Link without duplicate Link processes
- Link failure does not delay or break provider execution
- Island can start after Link recorded a breadcrumb
- Island disconnect returns Link to Drop Mode
- Link exits after last session and grace period

### Exit criteria

| Measurement | Gate |
|---|---:|
| Link Drop Mode P95 private working set | <= 10 MB |
| Link Drop Mode P99 private working set | <= 12 MB |
| Link Drop Mode hard ceiling | <= 16 MB |
| Link idle average CPU | <= 0.03% |
| Link GPU use in Drop Mode | 0 |
| provider observation Hook failure impact | none; fail-open |

---

## 7. Gate D: Codex CLI adapter

### Goal

Deliver the first full Observe-first adapter with the strongest available official sources.

### P0 target

- official integration install/rollback
- normal task discovery
- lifecycle state mapping
- late Island attachment when Link was already installed
- safe task/workspace identity where supported
- completed/failed/waiting mapping only when verified
- Context Routing at least workspace-ready
- session usage ledger where validated
- quota snapshot only through documented/supported source

### Mandatory probes

1. Start a normal Codex CLI task without Pulse UI open.
2. Confirm Link wakes through the supported integration path.
3. Start Island after task begins.
4. Confirm state without restarting or duplicating Codex.
5. Restart Island and confirm task rediscovery.
6. Trigger completion, failure, and quiet waiting-like periods.
7. Verify no false completed/failed state.
8. Verify original-context route label matches actual capability.

### Explicitly out of scope for this gate

- taking over arbitrary independently launched in-flight terminal turns
- default approval handling
- unverified private endpoint use

---

## 8. Gate E: Claude Code adapter

### Goal

Deliver high-value Hook-based observation without pretending to own the interactive client.

### P0 target

- user-level Hook integration install/rollback
- session/workspace breadcrumbs
- activity and terminal lifecycle mapping where formal hooks provide evidence
- verified waiting/permission awareness as a return-to-context signal
- workspace route and verified terminal/window focus where available
- local session token observation and burn meter if validated

### Mandatory probes

1. Start Claude Code normally with Island absent.
2. Confirm Hook wakes Link and session breadcrumb exists.
3. Start Island later and confirm correct running/waiting/completed state.
4. Confirm an unavailable Island does not interfere with native Claude permission behavior.
5. Confirm uninstall preserves other user Hook entries.
6. Confirm local token session data is never represented as global quota.

### Explicitly out of scope for this gate

- external arbitrary-session steering
- external arbitrary-session resume
- default in-Island permission approvals

---

## 9. Gate F: Antigravity capability probe

### Goal

Determine the formal integration surface before promising support depth.

### First deliverable

A probe report, not necessarily a full adapter.

The probe must verify:

- supported user-level integration mechanism, if any
- session identity availability
- lifecycle events
- workspace association
- context route possibility
- local session data format/contract, if documented
- token/quota source availability
- control surface availability, if any

### Release policy

- If official lifecycle source is validated: enable experimental Attached observation.
- If only process/workspace evidence exists: enable Observed-only mode.
- If no safe formal source exists: provide no synthetic lifecycle/fuel UI.

Antigravity remains Experimental until it passes the same truthfulness and late-attach gates as Codex and Claude.

---

## 10. Gate G: Pulse Fuel

### Goal

Make usage visible without false precision.

### P0 deliverables

- `QuotaSnapshot` with Reported/Observed/Estimated/Unavailable provenance
- session token ledger with coarse buckets
- Burn Meter using 5/20/60-minute windows
- Fuel Thread only for trustworthy low-fuel candidate
- Focus Card source labels
- one-click official usage route when data is unavailable

### Must prove

- no cross-provider quota aggregation
- local Claude token observation is not labelled account quota
- token counter reset does not produce negative consumption
- missing quota source shows Unavailable, not estimated percentage
- Fuel does not displace waiting/failure signal

### Deferred

- precise universal runway estimate
- automatic model/provider switching
- cost accounting/billing dashboard
- team quota sharing

---

## 11. Gate H: Attention and notification validation

### Goal

Verify the Island is calm and useful rather than noisy.

### Scenarios

- 100 ordinary activity events produce zero Toast notifications.
- Multiple simultaneous attention conditions become one grouped notification candidate.
- User opening original context creates Attention Lease and suppresses repeat reminder noise.
- Fullscreen/presentation/Focus mode hides Island and suppresses notifications.
- Returning from immersive mode produces at most one concise summary.
- 92% quota shows Fuel Thread but no Toast.
- confirmed quota block produces a red state and one eligible notification.

---

## 12. MVP release slice

The first user-credible MVP is not “all providers with every feature.”

### MVP required

- Native Signal / Peek / Focus Card shell
- state kernel + arbitration
- Pulse Link Drop Mode
- one supported real provider integration, preferably Codex or Claude based on measured probe success
- late Island attach for that provider after Link has been installed
- accurate Context-ready/Workspace-ready/Observed labeling
- basic Fuel support only where source is verified
- privacy/data boundary enforcement
- installation rollback and diagnostics

### MVP success statement

> Start a supported agent normally. Open Pulse Island later. See whether it is running, waiting, completed, or failed. Open the best available original context. Know usage pressure only when the source is trustworthy.

### Not required for MVP

- full three-provider parity
- in-Island decision/permission handling
- arbitrary external session control
- external third-party adapters
- team features
- cloud sync

---

## 13. Supported-claim checklist

Before a capability is allowed in user-facing marketing or settings:

1. A documented source or verified supported local integration exists.
2. Adapter mapping tests pass.
3. Late attach behavior is tested where claimed.
4. Truthfulness tests pass, including quiet/failure/unknown cases.
5. Privacy allow-list and diagnostics test pass.
6. Install/update/uninstall rollback passes.
7. Resource limits pass under event storm and multi-session conditions.
8. UI label and Context Route wording match actual capability.

Failure of any item means the capability remains Experimental or unavailable.

---

## 14. Implementation sequence

```text
0. Native shell + measurements
A. Signal benchmark
B. State kernel + reducer fixtures
C. Link + Hook transport + Drop Mode
D. First provider adapter
E. Context routing validation
F. Fuel for verified source
G. Attention policy validation
H. Second provider adapter
I. Antigravity probe and staged support
```

The team should resist parallelizing unverified provider adapters before the state kernel and Link lifecycle are proven. A clean signal core will make every adapter smaller; a weak core will make every integration fragile.

---

## 15. Design invariants

1. Measure before claiming support.
2. A passing demo is not a supported capability.
3. Late attach is a release gate, not a marketing adjective.
4. Truthfulness and privacy outrank feature count.
5. Control is optional; observation quality is the product.
6. The smallest valuable loop ships before provider parity.
