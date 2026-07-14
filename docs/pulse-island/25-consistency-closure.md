# Pulse Island · Consistency Closure v0.1.1

**Status:** Normative correction layer  
**Applies to:** All Pulse Island design documents `00` through `24`, README, and implementation handoffs  
**Precedence:** This document supersedes any earlier conflicting statement in the listed documents.  
**Last updated:** 2026-07-07

---

## 1. Purpose

The Pulse Island design package was reviewed end-to-end after the initial architecture pass. The review found no need to redesign the product, but it did identify several places where older overview wording diverged from later, more evidence-based contracts.

This document closes those gaps before and during provider-neutral implementation work.

> Provider capability, Fuel capability, route certainty, task state, and integration health are independent axes.

No one axis may silently upgrade another.

---

## 2. Normative authority map

When documents disagree, use this order.

| Topic | Normative source |
|---|---|
| Product posture and non-goals | `00-product-foundation.md` as amended here |
| Privacy and retention ceilings | `01-privacy-data-boundaries.md` plus Section 8 of this document |
| Task state and evidence | `02-agent-state-model.md` and `03-event-reduction-engine.md` |
| Primary narrative selection | Section 5 of this document, then `04-multi-agent-arbitration.md` |
| Route strength and Windows evidence | Section 6 of this document, `05-context-routing.md`, and `23-windows-observation-and-window-binding.md` |
| Link lifecycle and IPC | `14-spike-c-link-transport-drop-mode.md` |
| Provider capability release claims | `15-provider-capability-probe.md` plus provider Probe Cards `16`, `17`, and `19` |
| Canonical implementation order | `24-implementation-work-packages.md` |
| Installation/update/uninstall | `21-install-update-uninstall-contract.md` |
| Recovery and Safe Mode | Section 9 of this document and `22-reliability-recovery-and-diagnostics.md` |

`06-pulse-link-runtime-architecture.md` is an architecture overview only. It must not introduce a different pipe, startup, state, or retention contract from `14`.

---

## 3. Capability taxonomy

Pulse uses four separate axes. They must not be conflated in code, UI, documentation, or release wording.

| Axis | Examples | Meaning |
|---|---|---|
| Provider release status | `not_probed`, `process_observed`, `experimental_attached`, `supported_observe`, `supported_fuel`, `supported_control` | What the provider integration has earned through probe and release gates. |
| Per-task health | `attached`, `observed`, `degraded`, `offline` | How reliable the current task evidence is now. |
| Per-task route capability | `none`, `agent_ready`, `workspace_ready`, `context_ready` | What return-to-context level is currently available. |
| Per-task feature capability | `observe_waiting`, `open_workspace`, `open_exact_context`, `observe_quota_snapshot`, `observe_session_tokens` | Specific independently verified functions. |

Rules:

1. `workspace_ready` is a route capability, never a provider release label.
2. `attached` is task health, never a blanket provider-support claim.
3. `supported_observe` does not imply Fuel, exact routing, or control.
4. `supported_fuel` does not imply task-scoped tokens, Fuel Thread eligibility, or account-level quota for every task.
5. `context_ready` does not imply an exact action label unless route strength is Exact.

---

## 4. Provider support envelope

### 4.1 MVP provider posture

The MVP does not promise equal support for three providers.

```text
First supported_observe provider
= one provider selected after the W4 Codex/Claude probe race.

Codex CLI
= probe candidate, Hook-first observation candidate.

Claude Code
= probe candidate, Hook-first observation candidate.

Antigravity
= Passive / Observed only until its own formal official integration probe
  independently earns a higher capability.
```

A provider may appear in Pulse UI while only supporting Passive / Observed mode.

### 4.2 Passive/Observed ceiling

Process-only evidence may show:

```text
<Provider> · Observed process
[Show process details]
```

With independently verified workspace evidence, it may also show:

```text
<Provider> · Observed
[Open workspace]
```

Process-only evidence must not show:

- green running state
- yellow waiting state
- red failed/limited state
- completed state
- task title inferred from window/process content
- Fuel percentage, token count, burn estimate, or Fuel Thread
- `Open original task`

### 4.3 Release-claim rule

No product copy may say simply `<Provider> supported`.

Use capability-specific wording such as:

```text
Observation enabled
Workspace return available
Exact task return unavailable
Usage unavailable
Experimental reported quota window
```

---

## 5. Canonical primary-task order

The Arbitration layer must use this lexicographic tier order. Lower tie-breakers may only break ties within a tier; they may never reorder tiers.

```text
1. failed_or_nonquota_hard_block
2. waiting_user
3. verified_limit_reached
4. user_pinned
5. high_confidence_fuel_risk
6. resource_caused_stall
7. running
8. recent_terminal
9. idle_or_observed
```

### 5.1 Definitions

- `failed_or_nonquota_hard_block`: explicit provider failure, verified hard external block other than usage-limit exhaustion, or a task-specific error that prevents progress.
- `waiting_user`: provider-verified user confirmation, permission, or decision request.
- `verified_limit_reached`: a trusted rate/usage limit is actually blocking task progress. A high percentage alone is not enough.
- `high_confidence_fuel_risk`: warning only. It must not become red or outrank waiting/limit/failure.
- `resource_caused_stall`: a confirmed local resource condition causally blocks the task. Generic silence remains stalled/watch, not this class.

### 5.2 Required arbitration fixtures

```text
failed + waiting_user + limited + running
→ primary = failed

waiting_user + limited + running
→ primary = waiting_user

limited + pinned running
→ primary = limited

high_fuel_risk + waiting_user
→ primary = waiting_user

process_observed + attached_running
→ primary = attached_running
```

---

## 6. Canonical route strength and action labels

Route labels are determined by route strength, not by optimistic interpretation of task context state.

| Route strength | Evidence | Allowed user-facing labels |
|---|---|---|
| Exact | Documented provider task/thread route, or verified exact terminal-tab/session target | `Open original task`, `Open provider thread`, `Focus terminal tab` |
| Strong | Validated relevant provider/agent window, but exact task/tab cannot be proven | `Focus agent window`, `Focus related terminal` |
| Useful | Verified workspace, folder, provider surface, or official usage destination | `Open workspace`, `Reveal project folder`, `Open agent`, `Open official usage` |
| Weak | Process identity only | `Show process details` |
| None | No safe target | No primary route action |

Rules:

1. `Open original task` is Exact-only.
2. A process-owned window is not exact task proof merely because it belongs to the provider executable.
3. A terminal host without a stable provider/host tab association cannot receive `Focus terminal tab` or `Open original task`.
4. A route downgrade changes the label immediately and visibly.
5. No fallback launches a new CLI, resumes a stored session, sends synthetic keys, or claims recovery.

---

## 7. Canonical Link / Shim / IPC contract

`14-spike-c-link-transport-drop-mode.md` is the normative implementation contract.

### 7.1 Lifecycle

```text
NotRunning
→ Starting
→ Warm / Active / IslandActive / DropMode
→ GracePeriod
→ CheckpointAndExit
→ NotRunning
```

There is no permanent running `Idle` Link state. No active work plus no grace period means Link exits.

### 7.2 Transport

```text
Hook / Shim ingress
→ per-user ingress pipe

Island client
→ distinct per-user Island pipe
```

The first event for a newly started Link is passed through an inherited anonymous handoff pipe. It must not appear in command-line arguments, environment variables, temporary file names, or logs.

### 7.3 Island-facing protocol

Island may receive only bounded state-oriented messages:

```text
HelloAck
FullSnapshot
SnapshotDelta
LinkHealth
ProtocolError
```

Island does not receive:

- raw Hook payloads
- normalized event replay
- `EventBatch`
- provider transcript/history replay

On reconnect or revision gap, Island requests `FullSnapshot`, then resumes `SnapshotDelta` consumption.

---

## 8. Privacy profile retention precedence

Privacy profile is a retention ceiling. A more restrictive profile always overrides a less restrictive default retention rule.

### 8.1 Minimal local state

Allowed:

- active-task breadcrumb while task is nonterminal
- immediate terminal checkpoint
- bounded recent-terminal retention
- bounded recent-signal retention
- restart recovery from current/recent compact breadcrumb, downgraded to `degraded` until fresh evidence arrives

### 8.2 Strict local state

Allowed:

- active-task breadcrumb only while the task is nonterminal
- terminal checkpoint only long enough to complete the terminal transition atomically

Required after terminal checkpoint:

```text
remove terminal task breadcrumb
omit recent-terminal retention
omit recent-signal retention
never resurrect terminal breadcrumb after Link restart
```

### 8.3 Passive-only

```text
Do not install observation integrations.
Do not create integration breadcrumbs.
Allow only bounded current-process/window discovery during user-triggered or active validation operations.
```

### 8.4 Safe title rule

A safe task title is unavailable by default.

It may be displayed only when the relevant provider Probe Card and release capability matrix identify a specific provider field as safe, bounded, and allowed for display. Prompt text, command text, transcript text, tool input/output, and assistant text are never title sources.

When no safe title exists, use a generic provider/workspace label.

---

## 9. Safe Mode operational contract

Safe Mode is enforced at the earliest Pulse-owned executable boundary.

```text
Provider Hook invokes existing Pulse Shim
→ Shim reads current-user Safe Mode flag
→ Shim performs no Link wake
→ Shim performs no ingress forwarding
→ Shim exits 0 within normal fail-open budget

Island in Safe Mode
→ does not request Link wake
→ renders Passive mode only

Existing provider Hook configuration
→ remains unchanged until user explicitly repairs, re-enables, or removes it
```

Safe Mode does not:

- change provider behavior
- remove provider Hook entries automatically
- rewrite provider configuration
- restart provider processes
- reconstruct task history

### 9.1 Safe Mode acceptance scenario

```text
existing provider Hook entry remains installed
→ provider invokes Pulse Shim
→ Safe Mode is enabled
→ Shim exits 0
→ no pulse-link.exe process is spawned
→ provider follows native behavior unchanged
```

---

## 10. Integration ownership contract

Pulse identifies its own integration entry using a provider-compatible command signature plus a non-secret installation identifier.

Example semantic shape:

```text
pulse-link-shim.exe
--provider <provider-id>
--integration-id <installation-id>
```

Rules:

- Do not add unknown `owner`, `metadata`, or `integration_id` configuration keys when a provider schema does not support them.
- Provider-specific configuration determines how the exact executable-plus-argument signature is represented.
- Update, repair, and uninstall locate only the exact Pulse-owned signature.
- Pulse does not overwrite unrelated Hook handlers or restore stale whole-file backups.

---

## 11. Structured local-state source gate

The architecture may support provider-local structured-state readers, but this is a framework capability, not an entitlement.

A provider adapter may enable structured local state only when all of the following are named in that provider’s Probe Card and report:

1. Official or formally validated schema/source.
2. Exact approved fields.
3. Explicit dropped fields.
4. Freshness and identity rules.
5. Retention/privacy review.
6. Resource budget and failure behavior.

Raw transcripts, session JSONL, editor history, and arbitrary state files are not approved sources unless a future provider-specific Probe Card explicitly changes that decision.

---

## 12. Canonical execution order

`24-implementation-work-packages.md` is the canonical work-order source.

```text
W0 Workspace Foundation
→ W1 State Truth Kernel
→ W2 Native Signal Shell
→ W3 Link / Shim / Drop Mode
→ W4 Provider Probe Harness
→ W5 First narrow supported Observe adapter
→ W6 independent Context / Fuel enhancements
```

Controlled parallelism:

- W2 may begin after W0 contracts and the mock `PresentationPlan` seam are stable.
- W3 may not begin until W1 passes its truth fixtures.
- Live provider Hook installation may not begin until W3 and W4 gates are ready.
- Provider Fuel, exact routing, and terminal completion/failure mapping are W6 source-gated enhancements unless independently earned during a W5 probe.

---

## 13. Mandatory cross-layer regression scenarios

Add these to the appropriate Spike / work-package test suites before W5 begins:

1. Failed, waiting, limited, and running arbitration tier ordering.
2. Strong window route never uses Exact route wording.
3. Strict mode removes a terminal breadcrumb at terminal transition and after restart.
4. Safe Mode prevents Link spawn while existing provider Hooks remain fail-open.
5. Island reconnect receives `FullSnapshot` + deltas only, never event replay.
6. Process-only Antigravity cannot render running/waiting/completed/failed/Fuel state.
7. Fuel source availability can be absent, provider-specific, stale, or revoked without changing lifecycle truth.
8. An unknown/unsupported local structured-state source is rejected before it affects task state.

---

## 14. Implementation impact

### Work permitted now

- W4 Provider Probe Harness in read-only capability-discovery mode.
- W0/W1/W2/W3 are accepted as sequencing prerequisites for W4; their audit docs remain regression evidence, not active work queues.
- W4 report scaffolding must follow `15-provider-capability-probe.md` and the provider-specific Probe Cards while avoiding live provider mutation or support claims.

### Later-gated work

- Live provider Hook installation.
- Provider configuration mutation.
- Provider adapter implementation.
- Provider Fuel collection.
- Exact/strong route activation implementation.
- Production Link IPC that deviates from `14`.
- Any external session control or approval UI.

---

## 15. Design invariants

1. A provider process is not a task.
2. A provider support badge is not a capability bundle.
3. A usage source is not automatically a task-token source.
4. A relevant window is not automatically the original task.
5. A Pulse failure reduces observation only.
6. Privacy profile limits retention everywhere, including recovery.
7. When a contract is not proven, Pulse must downgrade its claim rather than improvise.
