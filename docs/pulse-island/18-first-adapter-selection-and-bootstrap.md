# Pulse Island · First Adapter Selection and Workspace Bootstrap

**Status:** Execution decision gate  
**Applies to:** Choice between Codex CLI and Claude Code as first real adapter, and the transition from design package to Rust implementation  
**Depends on:** `14-spike-c-link-transport-drop-mode.md`, `15-provider-capability-probe.md`, `16-codex-cli-probe-card.md`, `17-claude-code-probe-card.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Island now has enough architecture to begin implementation, but it must not allow a provider preference to skip proof gates.

This document makes two operational decisions:

1. **Implementation begins with shared Spike A/B/C foundations, not a provider adapter.**
2. **Codex CLI and Claude Code enter a short, evidence-only probe race after Spike C.** The first supported adapter is chosen by measured reliability, safety, and user-visible value.

The product must not decide “Codex first” or “Claude first” solely because one has richer theoretical APIs or because one is more familiar to the team.

---

## 2. No-provider-first rule

The repository begins with four provider-neutral deliverables:

```text
1. Native Signal Benchmark (Spike A)
2. State Kernel + Truth Fixtures (Spike B)
3. Link / Shim / Drop Mode (Spike C)
4. Provider Probe Harness
```

Until these are complete:

- no real provider Hook is installed
- no user configuration is modified
- no provider-specific parser is added
- no quota page/endpoint is queried
- no provider support claim is made

This keeps the first real integration narrow and prevents provider quirks from dictating the core state model.

---

## 3. Probe race design

After Spike C passes, run Codex and Claude probes in parallel only through the shared Provider Capability Probe Protocol.

### 3.1 Same gates, same evidence bar

Both probes must execute:

```text
P0 official-surface inventory
P1 passive process floor
P2 user-level install / rollback
P3 lifecycle semantics
P4 late attach
P5 context routing
P6 Fuel boundaries
P7 fail-open fault injection
P8 resource / retention measurement
```

No provider gets a softer gate because it exposes a more attractive feature.

### 3.2 Initial first-adapter scorecard

Score only measured results. Use a 0–3 score for each dimension:

| Dimension | 0 | 1 | 2 | 3 |
|---|---|---|---|---|
| User-level ingress | none | manual launcher only | formal Hook but limited | formal Hook, clean fail-open lifecycle |
| Session identity | process only | heuristic | Hook ID but ambiguous | stable Hook ID + strong correlation |
| Running truth | unknown | coarse activity | reliable freshness | reliable lifecycle/status boundary |
| Waiting truth | unavailable | heuristic | formal event but fragile | formal event + native pass-through verified |
| Terminal truth | unavailable | termination only | partial terminal mapping | explicit safe completed/failed mapping |
| Late attach | none | cold observed only | breadcrumb works with caveats | Link + Island restart proven |
| Context return | agent only | workspace only | strong terminal/window for some cases | exact context route proven |
| Fuel | unavailable | generic official usage route | one scoped source | reliable independent quota + task signal |
| Install safety | manual / invasive | reversible but brittle | targeted mutation | targeted update/rollback + fault test |
| Resource/fault behavior | fails gates | caveats | within budget | within budget + resilient stress test |

### 3.3 Weighted selection

For first-adapter selection, weight the dimensions as follows:

```text
Safety and fail-open behavior      25%
Truthful lifecycle / waiting       20%
Late attach                        15%
Install / rollback quality         15%
Context return                     10%
Resource profile                   10%
Fuel                               5%
```

Fuel has a low selection weight intentionally. It is strategically valuable but must not outrank a trustworthy attention loop.

### 3.4 Hard disqualifiers

A provider cannot be the first supported adapter if any of these are true:

- user-level integration needs project-file modification by default
- Hook/config changes cannot be cleanly removed
- Link failure changes provider behavior
- a waiting state requires Pulse to control a decision
- late attach duplicates/restarts the provider task
- claimed completion/failure is not supported by evidence
- task content must be parsed/stored to provide baseline value
- Link budget is exceeded under ordinary event traffic

---

## 4. Current decision posture

### 4.1 Codex CLI

Promising strengths to probe:

- formal Hook lifecycle path
- session/cwd identity candidate
- permission-request candidate
- separate official App Server track for Pulse-managed sessions
- possible official rate-limit source for Fuel

Primary caution:

- Hook `Stop` is not sufficient completion proof
- App Server thread history must not be mistaken for control over raw terminal in-flight work
- P0 raw-terminal session control remains out of scope

### 4.2 Claude Code

Promising strengths to probe:

- rich official Hook event surface
- user-level settings path on Windows
- explicit permission-dialog event
- robust session/cwd breadcrumbs
- precise native pass-through opportunity

Primary caution:

- Hook payloads contain unusually rich sensitive data
- Stop is a response boundary, not automatic task/session completion
- `/usage` is a user UI/local-history view, not a Pulse machine API
- OTel is outside local-only P0 boundary

### 4.3 No winner before live probe

Neither provider is designated first yet.

The initial engineering work should be provider-neutral. The first real adapter is selected only after its Probe Report and scorecard exist, and only if it reaches the `supported_observe` bar.

---

## 5. Bootstrap sequence

### Phase 0: Repository setup

Create the Cargo workspace and minimum policy files.

```text
Cargo.toml
rust-toolchain.toml
.cargo/config.toml
README.md
CONTRIBUTING.md
LICENSE decision placeholder
```

Initial workspace members:

```text
crates/pulse-domain
crates/pulse-protocol
crates/pulse-reducer
crates/pulse-arbitration
crates/pulse-routing
crates/pulse-fuel
crates/pulse-testkit
crates/pulse-win32
crates/pulse-island-ui
crates/pulse-persistence
crates/pulse-link-core
apps/pulse-island-spike
apps/pulse-link
apps/pulse-link-shim
apps/pulse-link-spike-client
```

Do not create provider adapter crates until Spike C passes.

### Phase 1: Shared quality policy

Set workspace-level enforcement for:

- Rust edition and minimum supported compiler
- deny unsafe code by default, with explicit narrowly scoped Win32 exceptions
- formatting and lint policy
- dependency review policy
- Windows-only build guard for Win32/UI binaries
- no browser/UI framework dependency
- no network client dependency in core/Link crates
- test fixture content policy

### Phase 2: Spike B first or Spike A first

Spike A and Spike B can be implemented in parallel after the domain/protocol baseline exists.

Recommended sequencing:

```text
Week 1 conceptual order:
Domain + Protocol
→ Spike B state fixtures
→ Spike A native signal shell
→ Spike C transport / Drop Mode
```

This does not imply calendar commitments. It only gives dependency order.

### Phase 3: Spike C and probe harness

After Spike B types stabilize:

- wire Link core to reducer
- implement Shim/Link named-pipe contract
- create fake Island CLI subscriber
- create fixture-driven synthetic Hook host
- add a provider-probe report skeleton generator

Only after C passes should a Codex or Claude adapter crate be created.

---

## 6. First implementation backlog

The first implementation backlog is intentionally small.

### B0.1: Domain types

Implement:

```text
ProviderId
TaskKey
LifecycleState
AttentionState
HealthState
ContextState
CapabilityId
TaskSnapshot
SafeSummary
SafeError
```

Acceptance:

- all string-bearing types have hard caps
- constructors enforce valid enum/state combinations where possible
- no Windows/provider dependency

### B0.2: Protocol framing and envelopes

Implement:

```text
PulseHookEnvelope
NormalizedEvent
FrameHeader
SnapshotDelta
FullSnapshot
ProtocolVersion
```

Acceptance:

- full byte-length validation before allocation
- no arbitrary JSON value/object escape hatch
- fuzz/property tests for malformed frame inputs

### B0.3: Reducer fixture runner

Implement:

```text
FixedClock
FixtureParser
TaskIndex
Admission
IdentityResolution
Reduce
SnapshotAssertion
```

Acceptance:

- F1–F10 fixture skeletons in place
- fixtures are synthetic and sanitized
- test command produces deterministic structured report

### A0.1: Native shell

Implement:

```text
Win32 window bootstrap
D3D11 device creation
DirectComposition root
Direct2D/DirectWrite minimal scene
MockPresentationPlanSource
```

Acceptance:

- compact mock Island shows/hides
- no focus theft
- no WebView/browser dependency

### C0.1: Link singleton and shim boundary

Implement:

```text
Current-user mutex
ingress pipe
bounded stdin JSON parser in Shim
inherited initial-event pipe
Link wake-if-needed
```

Acceptance:

- C1/C2/C3 synthetic scenarios pass before persistence or UI integration

---

## 7. Explicitly deferred bootstrap work

Do not begin these during the shared bootstrap:

- provider configuration editing code
- App Server client
- Claude `/usage` integration
- OTel configuration
- SQLite usage history database
- terminal window focus implementation
- notifications
- global Fuel dashboard
- approval controls
- dynamic external adapter loading
- auto-update service

Every one of these risks broadening the first commit beyond the proof loop.

---

## 8. Definition of implementation-ready

The design package is ready to move from architecture into code when the team accepts these statements:

1. Product truth is represented by the state kernel, not UI or provider code.
2. Provider support is capability-by-capability and probe-gated.
3. The first visible value is signal + safe return route, not control.
4. Link may lose an event, but may never harm an Agent or collect its content.
5. Provider selection happens after measurement, not before.
6. Every code task can cite a gate, a crate boundary, and an acceptance test.

---

## 9. Design invariants

1. Spike A/B/C are product infrastructure, not throwaway prototypes.
2. First-adapter selection is evidence-led and reversible.
3. No provider integration can expand Pulse’s privacy boundary.
4. The project’s first code commits should make incorrect claims harder, not merely make a window appear.
5. Adapter speed never outranks truthful observation.
