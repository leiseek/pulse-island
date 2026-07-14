# Pulse Island · Implementation Work Packages

**Status:** Canonical execution order  
**Applies to:** Repository bootstrap, implementation sequencing, review boundaries, acceptance gates  
**Depends on:** `11-rust-workspace-architecture.md`, `12-spike-a-native-signal-benchmark.md`, `13-spike-b-state-kernel.md`, `14-spike-c-link-transport-drop-mode.md`, `15-provider-capability-probe.md`, `23-windows-observation-and-window-binding.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

This is the canonical implementation order for Pulse Island. Earlier Gate/Spike documents describe proof goals; this document decides what may begin, what may parallelize, and what is gated for later work.

A work package is complete only when its truth, privacy, resource, failure, and test obligations are proven.

---

## 2. Canonical staircase

```text
W0 Workspace Foundation
→ W1 State Truth Kernel
→ W2 Native Signal Shell
→ W3 Link / Shim / Drop Mode
→ W4 Provider Probe Harness
→ W5 First narrow supported Observe adapter
→ W6 independent Context / Fuel enhancements
```

### 2.1 Controlled parallelism

- W2 may start after W0 has stable domain/protocol contracts and a mock `PresentationPlan` seam.
- W3 may not start until W1 passes required truth fixtures.
- W4 may be scaffolded during W3, but no live provider config mutation or Hook install occurs before W3 transport/fail-open gates pass.
- W5 begins only after W4 chooses a provider through evidence, not preference.
- W6 capabilities are independent source-gated enhancements. They are not bundled into W5 by default.

---

## 3. W0 · Workspace Foundation

### Goal

Create the provider-neutral Rust workspace and policy guardrails.

### Owns

```text
Cargo workspace root
rust-toolchain and lint policy
pulse-domain
pulse-protocol
pulse-testkit
workspace README / contribution policy
```

### Required outcomes

- core types use bounded strings and explicit enums
- framing validates length before allocation
- no network client, provider adapter, UI framework, SQLite, or Win32 dependency in core crates
- no generic `utils` crate
- `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` pass without providers or network

### Explicit non-goals

Reducer behavior, Link transport, UI, provider configuration, process observation, and persistence.

---

## 4. W1 · State Truth Kernel

### Goal

Implement deterministic task truth, identity, capability declarations, Fuel source state, route declarations, and arbitration as pure logic.

### Owns

```text
pulse-reducer
pulse-fuel
pulse-routing
pulse-arbitration
fixtures/reducer
fixtures/routing
fixtures/performance
```

### Required outcomes

- lifecycle, attention, health, route capability, provider release status, and feature capability remain separate axes
- process-only evidence cannot become running, waiting, completed, failed, Attached, or Fuel-aware
- terminal state cannot be undone by delayed lower-rank activity
- privacy profile is a retention ceiling
- same fixture plus fixed clock yields same snapshot and presentation plan
- arbitration uses canonical lexicographic tier order

### Required fixtures

```text
basic lifecycle
waiting truthfulness
terminal protection
identity/PID safety
freshness and degradation
summary priority
Fuel separation
arbitration
storm bounds
privacy/protocol hardening
```

Mandatory cross-layer additions:

```text
failed + waiting + limited + running → failed
waiting + limited + running → waiting
strong window route → never exact route label
process-only Antigravity → no running/waiting/terminal/Fuel state
Fuel revoked/stale → lifecycle unchanged
Strict terminal transition → no retained terminal breadcrumb after restart
```

### Exit gate

Spike B passes. No Windows API, provider config, IPC, raw provider file parsing, or UI is needed to run the core suite.

---

## 5. W2 · Native Signal Shell

### Goal

Prove native Signal → Peek → Focus Card → Palette with mock presentation input.

### Owns

```text
pulse-win32 minimal window/DPI/hotkey primitives
pulse-island-ui
apps/pulse-island-spike
performance fixture harness
mock compositor animation policy
```

### Required outcomes

- non-activating compact Island
- real interactive regions plus transparent click-through margins
- mock green/yellow/red/observed/aggregate states
- mock Exact/Strong/Useful/Weak route labels rendered faithfully
- no timer rotation
- reduced motion, high contrast, DPI, multi-monitor, immersive suppression
- no browser/WebView runtime and no static 60 Hz app-side redraw

### Exit gate

Spike A passes its behavior and resource targets.

---

## 6. W3 · Link, Shim, and Drop Mode

### Goal

Implement the provider-neutral event bridge under the exact Spike C contract.

### Normative source

`14-spike-c-link-transport-drop-mode.md` is authoritative for lifecycle, pipes, initial handoff, message caps, breadcrumb caps, Drop Mode, late Island attach, and grace exit.

### Owns

```text
pulse-link-core
pulse-persistence bounded breadcrumb store
pulse-win32 pipe/mutex primitives
apps/pulse-link
apps/pulse-link-shim
apps/pulse-link-spike-client
fixtures/link
```

### Required outcomes

- one Link per user/logon session
- separate ingress and Island pipes
- first event through inherited anonymous pipe, never command line
- `FullSnapshot` and `SnapshotDelta` only for Island state recovery
- bounded breadcrumb and privacy-profile retention behavior
- 90-second grace exit
- Safe Mode checked in Shim before Link wake/forward
- fail-open behavior under malformed input, Link absence, and protocol mismatch

### Exit gate

Spike C C0–C9 plus Safe Mode and Strict-retention scenarios pass. Link has no UI/GPU/network work in Drop Mode.

---

## 7. W4 · Provider Probe Harness

### Goal

Create repeatable evidence collection and capability reports before shipping an adapter.

### Required outcomes

- provider report contains version, environment category, integration mode, capability matrix, known limitations, resource figures, and release recommendation
- reports and fixtures contain no raw provider content/configuration
- install/update/uninstall transaction tests can run against synthetic config fixtures
- result labels distinguish provider release status from task health and route capability
- Codex and Claude Probe Cards can be executed as test plans
- Antigravity remains `not_probed` / `process_observed` unless evidence raises it

### Explicit non-goals

Shipping provider support, dynamic plugins, production analytics, control features.

---

## 8. W5 · First narrow supported Observe adapter

### Selection rule

Choose Codex or Claude only after the W4 scorecard and hard-disqualifier review. The first adapter is selected by measured safety, lifecycle truth, Late Attach, rollback, context return, and resource behavior.

### Allowed capability envelope

```text
formal user-level ingress
stable session identity where proved
workspace association
running freshness where proved
waiting signal where proved
late Island attach after Link breadcrumb
workspace-ready route
accurate Observed/Degraded fallback
```

### Explicit exclusions

```text
arbitrary external session control
approval/deny UI
transcript/history parsing
task title from raw prompt content
exact route without Exact evidence
completion/failure without explicit terminal evidence
Fuel without a separate scoped source
```

### Exit gate

The selected provider reaches `supported_observe` through its Probe Card. The non-selected provider remains a probe candidate, not a feature debt.

---

## 9. W6 · Independent enhancements

Each enhancement is a separate evidence-bearing package.

| Enhancement | Required gate |
|---|---|
| Strong/Exact terminal or window route | Route proof under `05` / `23` |
| Exact provider thread route | Official route plus verified task linkage |
| Reported quota window | Official source with scope/reset/freshness proof |
| Task token ledger | Formal task-scoped numeric source |
| Burn Meter | Valid task-token samples and bounded rollups |
| Completion/failure mapping | Explicit provider terminal evidence |
| Native Toast behavior | Attention/immersive suppression test |
| Provider control | Separate formal control and safety review |

No W6 enhancement may use transcripts, private endpoints, UI automation, or a generic “best-effort” parser to fill a source gap.

---

## 10. Cross-cutting review checklist

Every change set answers:

1. Which state or capability claim does this create?
2. What source and Probe evidence support it?
3. Which content is intentionally not read or stored?
4. How does it fail without affecting a provider?
5. What deterministic or Windows test prevents false confidence?
6. Does it obey the current privacy profile and Safe Mode rules?
7. Does it change a document governed by `25-consistency-closure.md`?

---

## 11. Definition of done

A package is complete only when:

1. Code is formatted, linted, and tested.
2. Relevant deterministic and Windows integration tests pass.
3. Failure behavior is implemented and tested.
4. Privacy/storage implications are reviewed.
5. Resource impact is measured or explicitly feature-gated.
6. User-visible labels match evidence strength.
7. No provider task behavior changed in the test scenario.
8. Relevant design documents, the Probe Card, and this work-package map remain accurate.

---

## 12. Active work boundary

Implementation currently authorizes:

```text
W4 Provider Probe Harness
```

W0/W1/W2 have deterministic gate evidence in `W1-GATE-AUDIT.md` and `W2-GATE-AUDIT.md`. W3 has accepted evidence in `W3-GATE-AUDIT.md` for W4 provider-probe start. The current active boundary is W4 under `15-provider-capability-probe.md` and the provider-specific Probe Cards.

Do not use older review prose or archived handoff notes to reopen W0/W1/W2/W3 before W4. W2 and W3 have no current implementation tasks unless a fresh regression test fails. If sequencing text conflicts, the current authority order is `25-consistency-closure.md`, this work-package map, `W2-GATE-AUDIT.md`, and `W3-GATE-AUDIT.md`.

W4 work means provider-neutral, read-only capability discovery and report scaffolding:

```text
provider report skeletons
environment/category manifests
capability matrices
known limitation summaries
resource figure placeholders
release recommendation labels
synthetic config transaction fixtures
```

It still does not authorize live provider Hook installation, provider configuration changes, provider adapters, real Fuel collection, provider process control, production route activation, network/App Server queries, transcript/session parsing, or arbitrary local-state parsing.

---

## 13. Design invariants

1. The order makes false claims hardest.
2. A visual demo never excuses a truth/privacy/fail-open failure.
3. Provider adapters are milestones, not starting points.
4. “Implemented” without a passing gate is not a release state.
