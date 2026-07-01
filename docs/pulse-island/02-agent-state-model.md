# Pulse Island · Agent State Model

**Status:** Design baseline  
**Applies to:** Pulse Link, provider adapters, Island renderer, Context Router  
**Last updated:** 2026-06-30

---

## 1. Purpose

Codex CLI, Claude Code, and Antigravity CLI do not expose identical lifecycle events, session IDs, capabilities, or data quality. Pulse Island must not leak those differences into the main user experience.

This document defines a single normalized state model that lets Pulse answer, with explicit confidence:

- Is this agent task running, waiting, completed, failed, or unknown?
- Can Pulse identify the task and return the user to its original context?
- Is state data current enough to display as Attached?
- Is the task merely observed as a process?
- Is a provider-specific control action available?

The model is designed for observation first. Control is an optional capability, not a lifecycle requirement.

---

## 2. Core terms

### Agent process

A local OS process that appears to be a supported tool or a child process belonging to one.

A process alone is not a task.

### Agent session

A provider-defined or Pulse-correlated unit of work. A session may span more than one process and may contain several turns, subagents, or tool calls.

### Pulse task

The user-visible card that represents one active, recent, or failed unit of agentic work. It is built from a normalized session identity and current lifecycle state.

### Provider event

A raw event from a Hook, documented app/server API, SDK, local session file, or safe process observation.

Provider events never reach the Island UI directly.

### Normalized event

A bounded, sanitized event accepted by the Pulse reducer.

### Capability

A verified statement about what an adapter can do for this task now. Capabilities are explicit and revocable.

---

## 3. Task identity model

Pulse must avoid merging different sessions in the same workspace and avoid attaching stale state to a reused PID.

### 3.1 Identity layers

Each task is identified through the strongest available layer.

| Strength | Identity source | Example | UI capability ceiling |
|---|---|---|---|
| A | Provider session ID plus live provider binding | Formal session/thread ID and active integration event stream | Attached or better |
| B | Provider session ID plus verified process start time | Hook session ID correlated to PID start timestamp | Attached |
| C | PID plus process start time plus workspace identity | Stable process correlation with known working directory | Observed / limited Attached only |
| D | Workspace plus recent activity heuristic | Same project and approximate timing | Observed only |
| E | Process signature only | `claude.exe` or child process recognized | Observed only |

Pulse must not elevate a task past `Observed` using D or E identity.

### 3.2 Stable task key

The internal task key is opaque and never equals a raw provider session ID.

```text
TaskKey = HMAC(local_install_key,
  provider
  + provider_session_id_or_process_fingerprint
  + process_started_at
  + workspace_stable_hash
)
```

The local install key is generated per user and protected through the local security model. The key is only used to prevent cross-session collision and to avoid writing raw identity into broad records.

### 3.3 Reconciliation rules

When new evidence arrives:

1. Prefer formal provider session identity over all weaker signals.
2. Preserve the earliest validated process start time.
3. Merge only when provider, session identity, and temporal correlation agree.
4. Split immediately when a single process becomes associated with two conflicting provider session IDs.
5. Never merge based on workspace alone.
6. If correlation becomes ambiguous, mark the task `Degraded` and surface it as separate observed entries rather than inventing certainty.

---

## 4. Normalized task record

The task record is deliberately small. It contains current state, not a replayable history.

```text
TaskRecord
├── task_key
├── provider
├── adapter_version
├── identity_strength
├── capability_set
├── lifecycle_state
├── attention_state
├── context_state
├── health_state
├── process_binding
├── workspace_ref
├── safe_task_title
├── last_event_summary
├── safe_error_summary
├── started_at
├── last_activity_at
├── terminal_at
├── last_verified_at
├── source_freshness
├── fuel_summary_ref
└── revision
```

### 4.1 Provider

Allowed initial values:

- `codex_cli`
- `claude_code`
- `antigravity_cli`
- `unknown_supported_process`

### 4.2 Lifecycle state

Lifecycle describes what the task is doing, independent of how much Pulse knows about it.

| State | Meaning |
|---|---|
| `discovered` | Pulse found a plausible agent process or source but has not validated task semantics. |
| `starting` | A formal or reliable lifecycle source says the session has started. |
| `running` | The task is actively executing or receiving meaningful activity. |
| `waiting_user` | The provider reliably indicates that user attention is required. |
| `paused` | The provider explicitly reports a non-terminal paused state. Optional provider support. |
| `stalled` | Work appears inactive beyond a conservative threshold, but failure is not confirmed. |
| `completed` | The provider or reliable completion evidence indicates normal completion. |
| `failed` | The provider or reliable process/result evidence indicates failure. |
| `limited` | A quota/rate limit demonstrably prevents task continuation. |
| `terminated` | Process/session ended without enough evidence to call completed or failed. |
| `unknown` | No safe lifecycle conclusion can be made. |

### 4.3 Attention state

Attention expresses whether the user should be pulled back.

| State | Meaning |
|---|---|
| `none` | No user action is currently indicated. |
| `informational` | A low-priority fact, usually completion or minor degraded state. |
| `watch` | A task is likely worth monitoring, such as elevated fuel or a possible stall. |
| `needs_user` | A provider indicates the task is waiting for the user. |
| `blocked` | A hard failure, confirmed quota limit, or other stop condition exists. |

### 4.4 Context state

Context is intentionally independent of lifecycle.

| State | Meaning |
|---|---|
| `none` | No route is known. |
| `agent_ready` | Pulse can open the relevant official tool, but not a precise task. |
| `workspace_ready` | Pulse can open the correct workspace or project directory. |
| `context_ready` | Pulse can focus an original window/tab or open an exact provider thread/session. |

### 4.5 Health state

Health describes confidence and freshness.

| State | Meaning |
|---|---|
| `attached` | Current task state is coming from a validated, live supported source. |
| `observed` | Process or limited information exists, but task semantics are incomplete. |
| `degraded` | A previously usable source is stale, conflicting, broken, or incomplete. |
| `offline` | The associated process/session cannot be reached or validated. |

### 4.6 Capability set

Capabilities are not inferred from a provider name. They are attached only after verification.

```text
Capabilities
├── discover_process
├── observe_lifecycle
├── observe_waiting
├── observe_safe_title
├── observe_safe_error
├── observe_plan_summary
├── observe_session_tokens
├── observe_quota_snapshot
├── open_agent
├── open_workspace
├── open_exact_context
├── control_stop
├── control_steer
├── control_resume
└── control_decision
```

`control_*` capabilities are disabled by default and are not required for P0.

---

## 5. State transitions

### 5.1 High-level graph

```text
                     ┌───────────────┐
                     │   discovered  │
                     └───────┬───────┘
                             │ validated start or activity
                             v
                       ┌──────────┐
                 ┌────>│ starting │────┐
                 │     └──────────┘    │
                 │                     v
                 │                ┌─────────┐
                 │                │ running │
                 │                └─┬───┬───┘
                 │                  │   │
                 │          user    │   │ inactivity
                 │          needed  │   v
                 │                  v ┌─────────┐
                 │           ┌────────┤ stalled │
                 │           │        └────┬────┘
                 │           v             │ verified activity
                 │     ┌──────────────┐    │
                 │     │ waiting_user │────┘
                 │     └──────┬───────┘
                 │            │ resolved / activity
                 │            v
                 │        ┌─────────┐
                 └────────┤ running │
                          └────┬────┘
                               │
       ┌───────────────┬───────┼───────────────┬───────────────┐
       v               v       v               v               v
  completed         failed   limited       terminated       unknown
```

### 5.2 Transition rules

| From | Event condition | To | Notes |
|---|---|---|---|
| `discovered` | Formal session start or validated active event | `starting` or `running` | Health can become `attached` only with identity A/B. |
| `discovered` | Process remains visible without semantics | `unknown` | Health remains `observed`. |
| `starting` | Meaningful provider activity | `running` | A start event alone is not enough to infer rich detail. |
| `running` | Formal waiting / user-input requirement | `waiting_user` | Requires adapter-specific verified mapping. |
| `running` | Reliable terminal success | `completed` | Completion must be explicit or corroborated. |
| `running` | Reliable failure / non-zero terminal failure mapping | `failed` | Do not use transient tool error alone. |
| `running` | Verified quota limit blocks continuation | `limited` | This produces red signal. |
| `running` | No meaningful activity beyond provider threshold | `stalled` | Never immediately turns red. |
| `stalled` | New reliable activity | `running` | Clear stall promptly. |
| `waiting_user` | Provider resumes activity | `running` | Do not assume user chose an option. |
| any active | Process exit with confirmed normal completion | `completed` | Provider evidence wins. |
| any active | Process exit with confirmed error evidence | `failed` | Provider evidence wins. |
| any active | Process/session disappears without conclusion | `terminated` | Attention is informational unless user-pinned. |
| any | Conflicting sources or stale live feed | same lifecycle, health=`degraded` | Lifecycle does not need to be erased. |

### 5.3 No false terminal state rule

Pulse must never infer `completed` merely because:

- a process is quiet
- a terminal window was closed
- no Hook arrived recently
- a transcript file stopped growing
- a child process exited while a parent session remained

When evidence is incomplete, use `terminated`, `stalled`, or `unknown`.

---

## 6. Freshness and staleness

A task state is only useful if Pulse knows how fresh it is.

### 6.1 Source freshness tiers

| Freshness | Meaning |
|---|---|
| `live` | A supported event stream or active hook activity is within its expected delivery window. |
| `recent` | Last reliable evidence is recent enough for the provider's normal quiet behavior. |
| `aging` | No new evidence for a longer period; display may remain but must be visually muted. |
| `stale` | State is too old to represent as current. |
| `expired` | Session should be removed from active consideration unless pinned or needed for recent completion. |

### 6.2 Provider-specific thresholds

Thresholds must be adapter configuration, not global hardcoded product assumptions. Each adapter declares:

```text
expected_event_gap_running
expected_event_gap_waiting
stale_after
expire_after
```

The initial generic defaults for unverified providers are conservative:

- `stalled` consideration after 6 minutes without meaningful evidence
- `aging` after 10 minutes
- `stale` after 20 minutes
- `expired` after 90 minutes if process/session cannot be validated

These defaults are not evidence of failure and do not create a red state.

---

## 7. Island signal mapping

The renderer maps state, attention, and health together.

| Lifecycle | Attention | Health | Island expression |
|---|---|---|---|
| `running` | `none` | `attached` | Green |
| `running` | `watch` | `attached` | Green with secondary Fuel/resource hint |
| `waiting_user` | `needs_user` | `attached` | Yellow |
| `failed` | `blocked` | any | Red |
| `limited` | `blocked` | attached/recent | Red with limit reason |
| `stalled` | `watch` | attached/recent | Muted green/gray watch state, not red |
| `completed` | informational | any | Completion confirmation, then clear |
| `unknown` | none | observed | Gray observed state |
| any nonterminal | any | degraded | State remains, but confidence indicator becomes muted/degraded |
| `terminated` | informational | offline | Gray terminal/unknown outcome |

The Island must not use color alone. It always pairs state with a concise text label in Peek and Focus Card.

---

## 8. Context routing

Every task may carry one best context route.

```text
ContextRoute
├── route_type
├── target_ref
├── verified_at
├── launch_policy
└── fallback_route
```

Allowed route types:

- `focus_window`
- `focus_terminal_tab`
- `open_provider_thread`
- `open_workspace`
- `reveal_workspace_folder`
- `open_provider_usage`
- `open_agent`
- `show_process_details`

Route selection priority:

1. Exact original context
2. Correct workspace
3. Correct agent surface
4. Safe process details

Context routes must expire or be revalidated when the underlying window/process disappears.

---

## 9. Provider adapter contract

Each provider adapter must supply a clear mapping table before it ships.

```text
AdapterContract
├── discovery_sources
├── identity_sources
├── supported_normalized_events
├── lifecycle_mapping
├── freshness_policy
├── capability_rules
├── context_route_rules
├── token_source_rules
├── quota_source_rules
├── terminal_evidence_rules
└── known_degradations
```

The contract must explicitly state what it does not know. A provider without a formal external control interface simply omits control capabilities.

---

## 10. Acceptance tests

### Identity safety

- Two sessions in the same workspace remain distinct.
- A reused PID cannot inherit an old task record.
- A task with conflicting session IDs becomes degraded rather than merged.

### Lifecycle truthfulness

- Quiet active work becomes `stalled`, not `failed`.
- Uncertain exit becomes `terminated`, not `completed`.
- A confirmed quota stop maps to `limited` and red.
- A waiting state is yellow only when backed by a verified provider signal.

### Late attach

- Island starts after a tracked task begins and restores task identity/state from Link breadcrumb plus live evidence.
- Island restart does not restart or duplicate the provider task.
- A task that began before Link existed can remain observed-only without fabricated title, plan, or waiting state.

### Context routing

- `Open original task` never opens a new blank terminal as a substitute.
- Lost original window falls back to workspace or agent route with a visible downgrade.

---

## 11. Design invariants

1. A process is not automatically a task.
2. A session ID is not automatically live context.
3. State and confidence are separate dimensions.
4. Context routing is valuable even when control is unavailable.
5. Control never upgrades observational confidence.
6. Unknown is a valid, honest state.
7. The reducer may reduce information, but it may not invent information.
