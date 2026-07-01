# Pulse Island · Pulse Link Runtime Architecture

**Status:** Architecture overview  
**Applies to:** Pulse Link, built-in adapter runtime, Island bridge, bounded local state  
**Normative runtime detail:** `14-spike-c-link-transport-drop-mode.md`  
**Depends on:** `03-event-reduction-engine.md`, `08-integration-hook-protocol.md`, `11-rust-workspace-architecture.md`, `14-spike-c-link-transport-drop-mode.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Link is the on-demand local runtime that receives safe integration signals, reduces them into compact task state, stores bounded breadcrumbs, and publishes snapshots to Pulse Island.

It is not:

- a permanent startup daemon
- a system-wide monitor
- a terminal wrapper
- a transcript/indexing service
- a provider task controller

> Link exists because active work or an explicit Island request needs it, and exits after active work has ended and its bounded grace period expires.

---

## 2. Normative implementation source

This document describes component responsibilities. The following behavior is normative in `14-spike-c-link-transport-drop-mode.md` and must not be redefined here:

- Link process lifecycle and no-permanent-idle rule
- single-instance namespace
- ingress and Island pipe split
- first-event inherited anonymous-pipe handoff
- message framing and payload caps
- current-user ACL requirements
- Drop Mode limits
- breadcrumb schema and retention caps
- Island attach/detach behavior
- 90-second grace exit
- transport/resource/failure acceptance scenarios

When in doubt, implement `14`.

---

## 3. Component topology

```text
Provider Hook / explicit launcher / bounded process observation
→ pulse-link-shim.exe (when a Hook command target is required)
→ pulse-link.exe
   ├── Adapter Runtime
   ├── Ingress Admission
   ├── Event Reduction Engine
   ├── Compact Snapshot Registry
   ├── Breadcrumb Store
   ├── Island Publisher
   └── Health / Lifecycle Coordinator
→ pulse-island.exe
```

### 3.1 Adapter Runtime

Adapters are provider translators. They may emit only:

```text
CandidateInstance
NormalizedEvent
CapabilityDelta
ContextRouteCandidate
UsageInput
```

Adapters cannot mutate task snapshots, write persistence, publish directly to Island, or bypass Event Reduction.

### 3.2 Ingress Admission

Ingress validates bounded wire input before it reaches a reducer:

- protocol version
- message kind
- length before allocation
- provider/event allow-list
- field caps
- forbidden content fields
- source integrity metadata

Rejected input is discarded. It never becomes a raw payload record.

### 3.3 Event Reduction

Only the Event Reduction Engine transforms admitted events into task snapshots. It applies identity reconciliation, source precedence, terminal protection, freshness, sanitization, and material-delta selection.

### 3.4 Snapshot Registry and Breadcrumb Store

Link holds only bounded current/recent compact state. It is not an append-only event log.

Retention obeys the active privacy profile:

- Minimal local state may retain bounded recent terminal breadcrumbs.
- Strict local state removes terminal breadcrumb after its atomic terminal transition.
- Passive-only does not create integration breadcrumbs.

### 3.5 Island Publisher

Island receives state, not events. It may receive only:

```text
HelloAck
FullSnapshot
SnapshotDelta
LinkHealth
ProtocolError
```

It cannot request `EventBatch`, raw Hook data, normalized-event replay, or provider history.

---

## 4. Runtime states

```text
NotRunning
→ Starting
→ Warm
→ Active
↔ IslandActive
↔ DropMode
→ GracePeriod
→ CheckpointAndExit
→ NotRunning
```

There is no permanent process-level `Idle` state.

| State | Allowed work | Forbidden work |
|---|---|---|
| Starting | Mutex, pipe setup, bounded handoff, breadcrumb load | UI, scans, provider control |
| Warm | Validate first/recovered state, initialize reducer | quota polling, rendering |
| Active | Reduce events, maintain compact snapshots | raw history retention |
| IslandActive | Publish snapshots/deltas | raw event export |
| DropMode | Reduce safe events and persist compact breadcrumb | UI, graphics, network, history scan, quota loop |
| GracePeriod | Wait for new relevant event, perform final checkpoint | recurring idle work |
| CheckpointAndExit | Bounded final write and cleanup | retry daemon behavior |

---

## 5. Local transport

Link exposes separate per-user, per-logon-session local channels:

```text
Ingress pipe
  Shim / formal integration → Link

Island pipe
  Island → Link request/subscribe
  Link → Island FullSnapshot / SnapshotDelta
```

The first event during Link wake-up is handed off through an inherited anonymous pipe, never command line, environment, temporary filename, or log.

All channels use current-user ACLs, schema validation, strict payload limits, and bounded timeouts.

---

## 6. Drop Mode

Drop Mode is expected when Island is not connected.

Allowed:

- receive validated event envelopes
- reduce compact state
- maintain task/session/process identity hints
- persist bounded breadcrumbs
- wait for terminal transitions and active-task exit

Forbidden:

- D3D, DirectComposition, Direct2D, DirectWrite, or any UI allocation
- provider transcript/history scan
- token timeline construction
- quota polling loop
- periodic full filesystem or process scan
- network request
- unbounded event queue or disk log

Drop Mode is an observer with a small notebook, not a hidden monitoring service.

---

## 7. Adapter scheduling and structured local state

Adapters use official Hook/API sources first. Process observation is a bounded fallback. Local structured-state readers are framework capability only and remain disabled for a provider unless its Probe Card explicitly approves:

1. source/schema
2. allowed fields
3. dropped fields
4. identity/freshness policy
5. privacy/retention policy
6. resource and fault behavior

Raw transcripts, session JSONL, editor history, and arbitrary local files are not approved inputs by default.

---

## 8. Resource policy

The executable budgets and measurement procedures are defined by `14`. The architecture rules are:

- no unbounded queue, cache, or log
- no background work without a bounded reason
- no rendering/GPU work in Link
- no high-frequency polling while no active task exists
- process exit waits preferred over recurring scans
- no always-on service lifecycle

If a budget breach occurs, Link reduces Pulse capability or exits after checkpoint. It never kills or alters provider processes.

---

## 9. Failure isolation

| Failure | Link response | Provider impact |
|---|---|---|
| malformed ingress | reject before reducer | none |
| adapter error | mark adapter/task source degraded, bounded backoff | none |
| Island disconnect/crash | enter Drop Mode | none |
| breadcrumb write failure | retain in-memory state, retry later material checkpoint | none |
| Link crash | later Hook/Island wake may restart; restored state begins degraded | none |
| protocol mismatch | fail-open at Shim; Link marks integration degraded | none |

A Pulse failure reduces observation only. It never becomes a provider task failure.

---

## 10. Safe Mode

Safe Mode is checked at the earliest Pulse-owned executable boundary:

```text
Shim sees Safe Mode
→ no Link wake
→ no ingress forwarding
→ exit 0 within fail-open budget
```

Island also does not request Link wake in Safe Mode. Existing provider Hook configuration remains untouched until the user explicitly re-enables or removes it.

---

## 11. Design invariants

1. Link collects bounded state; it does not surveil history.
2. Island receives snapshots, never raw event streams.
3. One user/logon session has at most one Link instance.
4. No active task plus no grace period means Link exits.
5. Provider behavior is always more important than Pulse observation continuity.
6. Structured local state requires provider-specific probe approval.
