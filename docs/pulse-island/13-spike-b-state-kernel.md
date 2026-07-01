# Pulse Island · Spike B: State Kernel and Truth Fixtures

**Status:** Executable spike plan  
**Goal:** Prove that bounded normalized events become deterministic, truthful task state and presentation input without UI, live providers, Windows APIs, or persistence  
**Depends on:** `02-agent-state-model.md`, `03-event-reduction-engine.md`, `04-multi-agent-arbitration.md`, `05-context-routing.md`, `11-rust-workspace-architecture.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Spike question

Can the pure state kernel reduce incomplete, delayed, duplicated, and conflicting evidence without inventing lifecycle, Fuel, route certainty, or provider capability?

A successful Spike B proves:

> Same admitted inputs plus same clock produce the same compact task snapshots and the same presentation plan. When evidence is insufficient, the result becomes lesser truth, not a convenient guess.

---

## 2. Strict scope

In scope:

```text
pulse-domain
pulse-protocol
pulse-reducer
pulse-fuel
pulse-routing
pulse-arbitration
pulse-testkit
sanitized synthetic fixtures
```

Out of scope:

```text
Win32
named pipes
SQLite
provider configuration
real Hook input
process/window APIs
UI rendering
network
transcript/session-file parsing
```

---

## 3. Required kernel APIs

```text
admit(envelope) -> AdmittedEvent | RejectionCategory
resolve_identity(index, event, clock) -> ResolvedTarget
reduce(prior, event, clock) -> ReductionResult
arbitrate(snapshots, preferences, leases, clock) -> PresentationPlan
```

Admission must validate size and forbidden fields before task state mutation. All state logic receives an injected clock.

---

## 4. Fixture rules

Fixtures are synthetic and sanitized. They contain no real prompts, transcripts, workspace paths, account IDs, credentials, commands, source code, diffs, or provider configuration.

Every fixture declares:

```text
name
clock_start
initial state
ordered events
privacy profile
expected task snapshots
expected presentation plan
expected persistence intent
```

---

## 5. Required fixture families

### F1 · Lifecycle basics

```text
start → activity → running
start → explicit completion → completed
start → explicit failure → failed
start → explicit limit block → limited
process-only discovery → observed/unknown only
```

### F2 · Waiting truthfulness

```text
verified waiting → waiting_user
repeated waiting → one material transition
lower-rank activity after waiting → waiting remains
verified waiting clear → running
stale waiting + fresh stronger activity → running
```

### F3 · Terminal protection

```text
completion → delayed activity → completed
failure → delayed activity → failed
process exit without terminal evidence → terminated/offline, not completed
silence → stalled/watch, not failed
high usage percentage without block → Fuel warning at most, not limited
```

### F4 · Identity safety

```text
two sessions in one workspace → two tasks
same PID, different process start → two tasks
same PID, conflicting session IDs → split/degrade, never merge blindly
workspace-only correlation → observed only
formal session arrives after process candidate → upgrade only with valid correlation
```

### F5 · Freshness and recovery

```text
fresh formal source → attached
stale source → degraded
fresh source after degraded → recover appropriately
breadcrumb restore after restart → degraded until fresh evidence
```

### F6 · Summary and privacy

```text
waiting summary survives generic activity
failure summary survives lower-value event
safe title absent → generic provider/workspace label
prompt-like/secret-like field → rejected or redacted before state
```

### F7 · Fuel separation

```text
quota source unavailable → no quota UI capability
account quota source → reported quota only, not task tokens
task token counter reset → no negative delta
duplicate sample → no duplicate burn
Fuel stale/revoked → lifecycle unchanged; capability degrades
separate providers → quota windows never aggregate
```

### F8 · Arbitration

Canonical required scenarios:

```text
failed + waiting + limited + running → failed
waiting + limited + running → waiting
limited + pinned running → limited
waiting + high Fuel risk → waiting
three ordinary attached-running tasks → aggregate active
process-only observed + attached running → attached running
```

### F9 · Route truthfulness

```text
Exact route evidence → Open original task allowed
Strong window evidence only → Focus agent window, never Open original task
Useful workspace evidence → Open workspace
process-only evidence → Show process details
stale route evidence → downgrade to fallback
```

### F10 · Privacy-profile retention

```text
Minimal terminal transition → bounded recent terminal breadcrumb allowed
Strict terminal transition → terminal breadcrumb removed after atomic transition
Strict restart → no terminal breadcrumb resurrection
Passive-only → no integration breadcrumb
```

### F11 · Storm and malformed input

```text
100k repeated ordinary activity events
100k token-like updates
10k invalid/unknown envelopes
hundreds of current tasks under caps
```

Expected:

- bounded memory
- no raw event retention
- terminal/waiting/failure never lost through ordinary coalescing
- diagnostics are category counters, not raw samples
- deterministic result

---

## 6. Required properties

1. Terminal state cannot be reverted by older/lower-rank nonterminal event.
2. Lower source rank cannot override fresh stronger field evidence.
3. Process-only evidence cannot become Attached, waiting, completed, failed, or Fuel-aware.
4. Token totals never go negative.
5. Every snapshot string observes its cap.
6. Exact route labels require Exact evidence.
7. Arbitration returns at most one primary and at most three Peek items.
8. Same inputs plus same clock yield identical output.
9. Strict retention overrides default terminal grace.
10. Fuel availability/revocation never changes lifecycle truth by itself.

---

## 7. Performance targets

| Workload | Required result |
|---|---|
| Typical: 3 tasks, sparse events | reducer P95 <= 1 ms per admitted event on reference machine |
| Terminal transition | reducer P95 <= 2 ms |
| Arbitration: 100 tasks | P95 <= 2 ms |
| Storm | bounded memory and deterministic output |
| Recovery: 100 breadcrumbs | no unbounded reconstruction/history work |

Exact machine-specific benchmarks may evolve, but deterministic behavior and bounded memory are release requirements.

---

## 8. Exit criteria

Spike B passes only when:

- F1–F11 and property tests pass
- forbidden/oversized data is rejected before state mutation
- primary arbitration order matches `25-consistency-closure.md`
- route labels match route strength
- strict retention behavior is tested
- process-only observation cannot impersonate agent semantics
- Fuel is source-gated and lifecycle-independent
- no Windows, provider, network, UI, or persistence dependency is needed for core tests

---

## 9. Failure interpretation

| Failure | Required correction |
|---|---|
| Ambiguous lifecycle | Add lesser/degraded state; do not force terminal state. |
| Reducer needs provider detail | Move mapping back to adapter boundary. |
| UI needs raw text | Redesign UI around snapshot fields. |
| Event storm grows memory | Tighten coalescing/caps, not retention. |
| Route label too strong | Lower route strength/label. |
| Fuel changes lifecycle without proof | Separate feature capability from lifecycle rule. |

---

## 10. Design invariants

1. Spike B proves truth handling, not provider reach.
2. Unknown, Observed, and Degraded are successful truthful outcomes.
3. The kernel may discard detail but never create certainty.
4. All fixtures are synthetic and content-minimized.
