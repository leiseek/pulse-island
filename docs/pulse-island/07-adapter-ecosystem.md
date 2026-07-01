# Pulse Island · Adapter Ecosystem and Capability Model

**Status:** Integration architecture baseline  
**Applies to:** Built-in provider adapters, capability gating, future external adapter boundary  
**Depends on:** `01-privacy-data-boundaries.md`, `02-agent-state-model.md`, `03-event-reduction-engine.md`, `05-context-routing.md`, `06-pulse-link-runtime-architecture.md`, `15-provider-capability-probe.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

An adapter translates provider facts into safe, bounded Pulse inputs. It earns each capability through a source, a probe, a privacy review, a failure review, and fixtures.

> An adapter translates truth. It never manufactures it.

Adapters emit only `CandidateInstance`, `NormalizedEvent`, `CapabilityDelta`, `ContextRouteCandidate`, or optional `UsageInput`. They never mutate task state, write breadcrumbs, publish directly to Island, or retain raw provider payloads.

---

## 2. Four capability axes

| Axis | Examples | Meaning |
|---|---|---|
| Provider release status | `not_probed`, `process_observed`, `experimental_attached`, `supported_observe`, `supported_fuel` | What a provider integration has earned through release gates. |
| Per-task health | `attached`, `observed`, `degraded`, `offline` | Reliability of current task evidence. |
| Per-task route capability | `none`, `agent_ready`, `workspace_ready`, `context_ready` | Return-to-context level. |
| Per-task feature capability | `observe_waiting`, `open_workspace`, `observe_quota_snapshot` | Specific verified functions. |

These axes are independent. `supported_observe` does not imply Fuel, exact route, or control. `workspace_ready` is not a provider release label.

---

## 3. Capability ladder

```text
process_observed
→ experimental_attached
→ supported_observe
→ supported_fuel
→ supported_control
```

Each step may be shipped independently. Passive process observation remains valid value when richer sources are unavailable.

---

## 4. Approved integration modes

| Mode | Default ceiling |
|---|---|
| Official Hook | Attached observation for explicitly verified events. |
| Official local API / SDK | Capability ceiling determined by probe. |
| Structured local state | Disabled until the Probe Card approves schema, fields, freshness, privacy, and fault behavior. |
| Window binding | Strong route at most; not exact task by default. |
| Process observation | Observed/process details and perhaps workspace route only. |
| Explicit launcher | Strong initial identity only; no PATH replacement. |

Raw transcripts, session JSONL, editor history, arbitrary extension files, copied local logs, OCR, screen scraping, private endpoints, and UI automation are not approved fallback sources.

---

## 5. Fuel is source-gated

```text
reported quota window
≠ task token ledger
≠ burn meter
≠ verified task limit block
```

Default P0 state for every provider until an individual probe proves otherwise:

```text
quota snapshot = unavailable
task token ledger = unavailable
burn meter = unavailable
Fuel Thread = unavailable
```

| Fuel capability | Required proof |
|---|---|
| Reported quota snapshot | Official source, scope, freshness, reset semantics, independent windows. |
| Task tokens | Formal task-scoped numeric source, no transcript parsing. |
| Burn Meter | Valid task-token samples and bounded rollups. |
| Limit reached | Trusted evidence that limit currently blocks task progress. |

Provider ceilings:

- **Codex CLI:** An experimental independent reported quota probe through a Pulse-owned official App Server may be investigated. Current-task token ledger remains unavailable until a formal task source exists.
- **Claude Code:** Account quota and current-session token totals are unavailable in P0. A narrow P1 subagent numeric probe is independent and optional.
- **Antigravity:** All Fuel capability remains unavailable until an official scoped source is proven.

---

## 6. Provider posture

### Codex CLI

Hook-first observation candidate: session/workspace identity, running freshness, waiting signal, and workspace route may be probed. Raw-terminal external control is not a claim.

### Claude Code

Hook-first observation candidate: session/workspace identity, running freshness, native permission waiting, and workspace route may be probed. Native permissions remain native; external session control is not a claim.

### Antigravity

Passive / Observed only until its own formal official integration probe earns more. Process evidence cannot create running, waiting, terminal, Fuel, exact-route, or control claims.

---

## 7. Capability state

| State | Meaning |
|---|---|
| `unavailable` | No approved source. |
| `probing` | Source is being validated. |
| `available` | Verified and usable now. |
| `degraded` | Previously valid source is stale or failing. |
| `blocked` | User or policy forbids use. |

Unsupported capabilities are omitted from normal UI rather than shown as decorative disabled controls.

---

## 8. Structured local-state gate

A provider may enable structured local state only when its Probe Card and report identify:

1. approved source/schema,
2. exact accepted fields,
3. fields that are dropped,
4. identity/freshness rules,
5. retention/privacy limits,
6. resource and failure behavior.

Framework support is not permission to create a generic file scanner.

---

## 9. Adapter health

```text
ready | active | degraded | needs_repair | blocked | disabled | experimental
```

Adapter health is separate from task lifecycle. A broken Hook makes Pulse observation incomplete; it never makes the provider task failed.

---

## 10. Release gate

No adapter becomes `supported_observe` merely because it compiles or detects a process. It must pass provider-specific proof for:

- safe installation/update/uninstall
- fail-open behavior
- lifecycle truthfulness, including quiet and terminal ambiguity
- Late Attach and restart recovery
- route wording truthfulness
- privacy allow-list
- resource and breadcrumb bounds
- fault injection

The Provider Capability Probe Protocol and the Codex, Claude, and Antigravity Probe Cards are authoritative.

---

## 11. External adapters later

P0 uses built-in adapters only. A future external adapter must be out-of-process, versioned, bounded, explicitly enabled, publisher-identified, and unable to access raw transcripts or the Pulse database.

---

## 12. Design invariants

1. Provider support is capability-by-capability.
2. Observation does not imply Fuel, exact routing, or control.
3. Process presence is not task understanding.
4. Structured local state is source-gated.
5. Missing data stays unavailable.
6. Adapter faults degrade Pulse only.
