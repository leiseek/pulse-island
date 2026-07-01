# Pulse Island · Event Reduction Engine

**Status:** Design baseline  
**Applies to:** Pulse Link, provider adapters, usage scanners, state model reducer  
**Depends on:** `00-product-foundation.md`, `01-privacy-data-boundaries.md`, `02-agent-state-model.md`  
**Last updated:** 2026-06-30

---

## 1. Purpose

Provider tools emit noisy, incomplete, differently shaped signals:

- lifecycle hooks
- documented SDK or app-server notifications
- local session-file updates
- process start and exit observations
- quota snapshot refreshes
- token counter changes
- user interaction with Pulse

Pulse Island must not act as a log viewer. It needs a small, truthful picture of each task that is safe to render at a glance.

The Event Reduction Engine converts approved incoming signals into bounded task state:

```text
Raw provider signal
→ normalized event
→ identity resolution
→ evidence validation
→ semantic reduction
→ state transition
→ attention / fuel update
→ compact task snapshot
```

The reducer is not permitted to invent a conclusion that is absent from validated evidence.

---

## 2. Design goals

The reducer must:

1. Accept heterogeneous provider signals through a single bounded envelope.
2. Preserve source provenance and freshness.
3. Reject unknown or unsafe payload fields before persistence or IPC.
4. Collapse frequent activity into low-rate state updates.
5. Detect meaningful state transitions without overreacting to transient noise.
6. Maintain a small current snapshot rather than a replayable event log.
7. Support late Island attachment from a small Link breadcrumb.
8. Make uncertainty visible through health and confidence, not fabricated certainty.
9. Keep Drop Mode CPU, memory, disk, and wakeups minimal.
10. Be deterministic and testable from fixed event fixtures.

---

## 3. Non-goals

The reducer does not:

- reconstruct full conversations
- generate a semantic summary from untrusted raw transcripts
- interpret hidden reasoning
- scrape terminal text
- perform unrestricted natural-language classification on task content
- decide provider permissions or approvals
- calculate a universal cross-provider quota balance
- archive all events for audit or analytics

---

## 4. Input model

### 4.1 Raw source classes

Every source is ranked by trust and capability. The rank affects what state it may change.

| Rank | Source class | Typical examples | May establish |
|---|---|---|---|
| 1 | Formal live provider integration | documented app-server / SDK event / official Hook event | session identity, lifecycle, waiting, completion when specified |
| 2 | Formal local session metadata | documented or verified local structured session data | safe session identity, recent activity, token counters |
| 3 | OS process observation | process start, exit, parent-child relation, executable identity | discovered/terminated process presence only |
| 4 | User-provided Pulse interaction | pin, mute, open-context confirmation | presentation and attention preferences only |
| 5 | Heuristic inference | time gap, changed file timestamp, weak workspace match | watch/stalled suggestion only, never terminal truth |

A lower rank cannot overwrite a higher-rank lifecycle conclusion unless the higher-rank source is stale or explicitly invalidated.

### 4.2 Normalized event envelope

No raw provider payload may be forwarded into the reducer. An adapter must output this bounded schema.

```text
NormalizedEvent
├── event_id
├── received_at
├── occurred_at
├── provider
├── source_class
├── source_instance_id
├── source_confidence
├── identity_hint
├── event_kind
├── lifecycle_hint
├── attention_hint
├── context_hint
├── capability_delta
├── safe_summary
├── safe_error
├── token_delta
├── quota_snapshot_delta
├── process_binding
├── freshness_hint
└── payload_revision
```

### 4.3 Hard field limits

The event envelope follows the privacy contract:

- maximum serialized event size: 8 KB
- maximum safe summary: 240 UTF-8 bytes
- maximum safe error: 320 UTF-8 bytes
- maximum identity hint: 256 UTF-8 bytes
- no arbitrary map of unknown fields
- no nested raw provider object
- no command text, prompt body, transcript excerpt, diff, or source code field

Oversized events are rejected or reduced by the adapter before the reducer sees them.

### 4.4 Event kinds

Initial event kinds are intentionally generic:

```text
process_discovered
process_exited
session_started
activity_observed
waiting_observed
waiting_cleared
completion_observed
failure_observed
limit_observed
context_route_observed
capability_observed
token_usage_observed
quota_snapshot_observed
source_health_changed
user_pinned
user_unpinned
user_opened_context
user_muted
user_unmuted
maintenance_tick
```

Provider adapters may have richer local terminology, but must map it into one or more of these kinds.

---

## 5. Reduction pipeline

The reducer has six explicit stages. Stages are small and side-effect limited so they can be tested independently.

```text
1. Admit
2. Identify
3. Validate
4. Coalesce
5. Reduce
6. Publish
```

### 5.1 Stage 1: Admit

Goal: reject input that is malformed, too large, unsafe, stale beyond recovery, or from an unauthorized local source.

Checks:

- schema version recognized
- message length within limit
- current-user IPC identity valid when applicable
- provider identifier supported
- source class declared
- required timestamp and event kind present
- safe fields already sanitized
- no forbidden fields

Outcome:

```text
AdmittedEvent | RejectedEvent(reason)
```

Rejected events increment a bounded diagnostic counter by category. They are not persisted verbatim.

### 5.2 Stage 2: Identify

Goal: match the event to an existing task or create a new task candidate.

Rules:

1. Resolve formal provider session ID first.
2. Correlate with process binding and start time if available.
3. Use workspace hash only as supporting evidence.
4. Use process signature only to create an observed candidate.
5. Do not merge based on similar titles, recent activity, or same project folder alone.

Output:

```text
ResolvedTarget
├── exact_task
├── candidate_task
├── ambiguous_tasks
└── new_task_candidate
```

Ambiguous identity does not block the event. It creates or updates an observed task with degraded health, rather than merging data unsafely.

### 5.3 Stage 3: Validate

Goal: decide whether the event has enough authority to change each individual field.

Validation is field-specific. A process exit may update process binding but cannot alone claim successful completion. A quota snapshot may update fuel but cannot claim lifecycle failure unless it contains a verified blocking result.

Each field mutation carries:

```text
FieldEvidence
├── source_rank
├── occurred_at
├── received_at
├── confidence
├── freshness
└── supersedes_rule
```

### 5.4 Stage 4: Coalesce

Goal: collapse repetitive events before mutating outward-facing state.

Examples:

```text
100 activity_observed events in 2 seconds
→ one activity freshness update

30 token_usage_observed events
→ one minute bucket token delta

same waiting_observed event repeated
→ one waiting transition, freshness refresh only

same quota snapshot from cache
→ no user-visible update if material values did not change
```

Coalescing operates per task and per event family. It never discards a terminal event merely because it arrives within a coalescing window.

### 5.5 Stage 5: Reduce

Goal: apply validated information to the `TaskRecord` state model.

The reducer:

- selects the strongest current evidence for lifecycle state
- updates freshness and health
- updates safe last-event/error summary under size limits
- updates capability set only through verified capability rules
- adds aggregate token deltas to bounded ledger buckets
- updates quota snapshot provenance and staleness
- recomputes attention state
- increments a revision only when visible state materially changes

### 5.6 Stage 6: Publish

Goal: emit only compact state changes to Island and local persistence.

A publish contains a `TaskSnapshotDelta`, not raw events.

```text
TaskSnapshotDelta
├── task_key
├── revision
├── changed_fields_mask
├── compact_task_snapshot
├── attention_delta
├── fuel_delta
└── published_at
```

If the new snapshot is materially identical to the previous one, nothing is published.

---

## 6. Evidence precedence

### 6.1 Lifecycle precedence

For a task with active and fresh evidence, use the following descending precedence:

1. Explicit provider terminal outcome: completed / failed / limited
2. Explicit provider waiting state
3. Explicit provider running / active state
4. Explicit provider paused state
5. Fresh formal Hook or local session activity
6. OS process still alive
7. Time-gap heuristic

The result must also be tempered by freshness. A stale explicit `running` event cannot suppress a newer verified `process_exited` signal forever.

### 6.2 Terminal-state precedence

Terminal states require stronger evidence than nonterminal states.

| Candidate state | Minimum evidence |
|---|---|
| completed | explicit formal completion or corroborated terminal success result |
| failed | explicit formal failure or corroborated terminal failure result |
| limited | formal limit/usage block that demonstrably stops the task |
| terminated | process/session exit without a qualifying terminal result |
| stalled | time-based lack of meaningful activity only |

### 6.3 Conflict examples

| Evidence A | Evidence B | Result |
|---|---|---|
| live provider says running | process appears quiet | running, possibly watch after threshold |
| provider said running 25 min ago | verified process exited now | terminated unless terminal result proves outcome |
| stale waiting signal | fresh provider activity | running |
| session file says completed | live provider says active | degrade and preserve active until resolved |
| quota is 95% used | task active | running with fuel watch, not limited |
| provider says rate limit blocked | process alive but no activity | limited, if event maps to the active task |

### 6.4 Confidence score

Pulse may maintain a numeric confidence internally, but should not expose artificial precision in the UI.

Suggested model:

```text
confidence = identity_strength_weight
           + source_rank_weight
           + freshness_weight
           + corroboration_weight
           - conflict_penalty
           - ambiguity_penalty
```

Presentation maps to discrete labels only:

- Attached
- Observed
- Degraded
- Offline

No UI should say `87% certain`.

---

## 7. Temporal behavior

### 7.1 Event timestamps

Events may arrive out of order. Every event carries both `occurred_at` and `received_at`.

Rules:

- `occurred_at` determines semantic order when trustworthy.
- `received_at` controls transport freshness and diagnostics.
- Events older than a bounded lateness window may refresh diagnostics but should not revert terminal state.
- Terminal events are idempotent by task and result class.

Initial lateness policy:

```text
normal event lateness window: 60 seconds
terminal event reconcile window: 5 minutes
quota snapshot freshness window: adapter-defined
```

### 7.2 Out-of-order example

```text
12:00:00 provider activity
12:00:03 provider completed
12:00:05 delayed provider activity received
```

Result remains `completed`. The delayed activity may update a bounded diagnostic counter but cannot resurrect the task to `running`.

### 7.3 Debounce windows

Suggested initial windows, all adapter-tunable:

| Event family | Coalesce / debounce |
|---|---:|
| activity | 2 seconds |
| token delta | 60 seconds bucket |
| quota snapshot | 30 seconds identical-value suppression |
| repeated waiting | suppress repeat transition until state clears |
| context route update | 2 seconds |
| health degradation | 5 seconds to avoid transient pipe blips |
| terminal events | no debounce; process immediately |

---

## 8. Meaningful-event policy

Pulse stores only one current `last_event_summary` per task.

An event is meaningful when it changes at least one of:

- lifecycle state
- attention state
- context availability
- terminal outcome
- safe error category
- fuel risk tier
- explicit provider phase summary

Examples that are not meaningful by default:

- repeated token increments
- repeated generic tool progress
- heartbeat-only updates
- unchanged quota cache values
- repeated process observation
- repeated waiting state without new reason

### Summary replacement rule

Replace the current summary only when the new event has equal or greater semantic priority.

Suggested priority:

```text
failure / limit
> waiting
> completion
> explicit phase
> activity
> process discovery
```

A low-value activity event must not overwrite `Waiting for confirmation` or `Build failed`.

---

## 9. Fuel reduction

Fuel has separate reduction rules because quota and task tokens are different facts.

### 9.1 Quota snapshots

A quota snapshot update is accepted only when it includes:

- provider / account scope identifier
- observed timestamp
- source type
- percentage or equivalent remaining/used amount
- reset time when available
- staleness rule

The reducer keeps the newest valid snapshot for each independent provider window.

It does not sum snapshots across providers or present a global percentage.

### 9.2 Token deltas

Token usage is stored as aggregate deltas by task and coarse time bucket.

Reducer behavior:

```text
new token counter
→ validate source/session relation
→ calculate nonnegative delta when counter is cumulative
→ cap anomalous delta for diagnostics review
→ add delta to current bucket
→ update burn meter inputs
```

Negative token deltas are never added. They trigger a source-reset reconciliation path.

### 9.3 Burn meter

Burn derives from aggregated session deltas, not individual token events.

Suggested windows:

```text
short: 5 minutes
primary: 20 minutes
baseline: 60 minutes
```

The engine outputs a discrete tier:

- `normal`
- `elevated`
- `high`
- `unknown`

A tier change requires both sufficient sample volume and a sustained difference. One short burst must not create a high-burn warning.

### 9.4 Fuel risk state

Fuel risk is secondary to lifecycle.

| Fuel condition | Reducer output |
|---|---|
| quota normal / unknown | no attention change |
| quota in watch range | `attention=watch` only if it is the task's highest relevant risk |
| quota low | low-fuel flag and Fuel Thread candidate |
| provider-confirmed quota block | lifecycle=`limited`, attention=`blocked` |
| high burn without quota | burn tier only; no assumed runway |

---

## 10. Drop Mode behavior

Drop Mode must remain microscopic.

When Island is not running, the reducer operates in **breadcrumb-only mode**.

It may update:

- identity proof
- process binding
- coarse lifecycle state
- waiting flag
- terminal state
- last activity timestamp
- safe title/error/event summary
- context route hints

It does not:

- retain a raw event queue
- scan historical session files continuously
- build token histories
- fetch quota snapshots on a timer
- compute charts
- create GPU resources
- publish UI deltas

### Breadcrumb write policy

- Memory state is updated immediately.
- Persistent breadcrumb writes are debounced, except for terminal/waiting/failure transitions.
- At most one durable write per task per small configured interval during ordinary activity.
- All writes are atomic and bounded.

This makes late Island attach possible without converting Link into a hidden database daemon.

---

## 11. Island-active behavior

When Island connects, Link changes publication behavior but not truth rules.

Island-active additions:

- publish compact snapshot deltas via local Named Pipe
- enable active-session token delta scans where adapter support exists
- refresh supported quota snapshots at adaptive intervals
- compute fuel tier transitions
- expose Context Router availability
- emit bounded recent signal entries only for state turns

The Island can disconnect without changing task truth. Link falls back to Drop Mode.

---

## 12. Failure handling

### Adapter crash or transport loss

- Preserve last validated lifecycle state.
- Mark health `degraded` after the short grace window.
- Stop assuming fresh task progress.
- Do not auto-promote to failed.
- Recover to attached only after a fresh validated source event.

### Link restart

- Restore bounded breadcrumbs.
- Revalidate PID/process start identity.
- Attempt source reconnection where supported.
- Mark stale breadcrumbs degraded until revalidated.

### Storage unavailable

- Continue with in-memory state when possible.
- Do not write raw fallback logs.
- Surface a local diagnostics health condition.
- Losing persistent breadcrumb state may reduce late attach, but must not interfere with provider tasks.

### Event storm

- Apply per-task and global rate caps.
- Preserve terminal, waiting, failure, and limit events.
- Coalesce activity and token events aggressively.
- Record only aggregate dropped-event counters for diagnostics.

---

## 13. Deterministic test fixtures

Reducer tests should use event sequences, expected snapshots, and no live providers.

Required fixture families:

1. Repeated activity collapses into one snapshot update.
2. Waiting cannot be overwritten by low-value activity.
3. Delayed activity cannot undo completion.
4. Quiet work becomes stalled, not failed.
5. Process exit without a result becomes terminated.
6. Formal quota block becomes limited and red.
7. Two sessions in one workspace remain separate.
8. PID reuse cannot revive an old task.
9. Conflicting provider events degrade health without inventing a terminal state.
10. Link restart restores breadcrumb as degraded until source revalidation.
11. Token counter reset does not produce negative usage.
12. Huge event payload is rejected before persistence.
13. Raw sensitive payload fields cannot reach task snapshot or logs.
14. Event storm leaves memory and queue bounded.

---

## 14. Implementation shape

Recommended Rust boundaries:

```text
pulse-link
├── adapter_ingress/
│   ├── codex.rs
│   ├── claude.rs
│   ├── antigravity.rs
│   └── process_observer.rs
├── event/
│   ├── normalized_event.rs
│   ├── admission.rs
│   ├── sanitizer.rs
│   └── coalescer.rs
├── state/
│   ├── identity.rs
│   ├── reducer.rs
│   ├── task_record.rs
│   ├── freshness.rs
│   └── evidence.rs
├── fuel/
│   ├── quota.rs
│   ├── ledger.rs
│   └── burn_meter.rs
├── persistence/
│   └── breadcrumb_store.rs
└── ipc/
    └── island_publish.rs
```

The reducer core should be pure or near-pure:

```text
reduce(previous_task, admitted_event, clock)
→ reduction_result
```

I/O, disk writes, pipe publishing, provider calls, and timers stay outside the pure reduction boundary.

---

## 15. Design invariants

1. Raw provider payloads never become reducer state.
2. Terminal claims require stronger evidence than active claims.
3. Older events cannot undo newer terminal truth.
4. Lower-rank sources do not overwrite fresh higher-rank evidence.
5. Every visible state has a provenance and freshness basis.
6. Drop Mode keeps state, not a log.
7. A reducer result may be less detailed than its input, never more certain.
8. The system can safely lose Pulse data without affecting provider tasks.
