# Pulse Island · Product Foundation

**Status:** Product baseline  
**Platform:** Windows 11, native Rust  
**Product posture:** Observe first. Control only when formally supported and explicitly enabled.  
**Consistency baseline:** `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Product promise

Pulse Island is a small desktop attention layer for agentic coding work. It helps a developer answer four questions without opening a dashboard:

1. Is there trustworthy evidence that agent work is active?
2. Does an agent need me now?
3. Is a verified resource or usage condition blocking work?
4. What is the strongest verified route back to the relevant context?

The product path is:

```text
Signal
→ Peek
→ Focus Card
→ strongest verified context route
```

Pulse is not an IDE, terminal replacement, transcript viewer, provider account manager, generic process monitor, or agent orchestrator.

---

## 2. Product truth

Pulse tells less when evidence is weak.

| Truth level | What Pulse may say |
|---|---|
| Attached | Provider/integration source currently verifies task state. |
| Observed | Pulse sees process/workspace facts, not a full task lifecycle. |
| Degraded | Earlier evidence existed but is stale, ambiguous, or failing. |
| Context-ready | A task has a route candidate; final route label depends on route strength. |

A process is not a task. A window is not a session. A percentage is not a quota block. A stored session ID is not permission to control an in-flight external terminal task.

---

## 3. Signal model

### 3.1 Compact Island

The compact Island communicates one primary narrative:

```text
[state] [provider/workspace-safe subject] [one reason] [+N]
```

Examples:

```text
! Claude Code · Waiting for confirmation · +2
× Codex CLI · Build failed
● 3 agents working
○ Antigravity · Observed process
```

It never rotates through tasks on a timer. When several ordinary tasks are active, it uses an aggregate rather than fabricating a merged session.

### 3.2 State color rules

| Signal | Meaning |
|---|---|
| Red | Explicit failure, non-quota hard block, or verified usage limit currently blocking progress. |
| Yellow | Verified user confirmation, permission, or decision is needed in the original tool. |
| Green | Trustworthy attached work is active. |
| Off / parked | No active trustworthy task signal. |
| Muted / observed | Process/workspace evidence exists, but task semantics are unavailable. |

Stalled is not automatically Red. Fuel warning is not a new traffic-light color and cannot replace waiting/failure as the primary story.

---

## 4. Canonical priority

```text
failed_or_nonquota_hard_block
> waiting_user
> verified_limit_reached
> user_pinned
> high_confidence_fuel_risk
> resource_caused_stall
> running
> recent_terminal
> idle_or_observed
```

This order is lexicographic. Hidden scoring may break ties within a tier but may not reverse the order.

---

## 5. Context return promise

Pulse never pretends a generic app or new terminal is the original task.

| Evidence strength | User action wording |
|---|---|
| Exact | `Open original task`, `Open provider thread`, `Focus terminal tab` |
| Strong | `Focus agent window`, `Focus related terminal` |
| Useful | `Open workspace`, `Reveal project folder`, `Open agent`, `Open official usage` |
| Weak | `Show process details` |

`Open original task` is Exact-only. No fallback may launch a new agent command, resume a stored session as a substitute, or synthesize terminal input.

---

## 6. Provider posture

Pulse does not promise equal feature depth across providers.

| Provider | Current product posture |
|---|---|
| Codex CLI | Hook-first observation candidate. It may earn session/workspace, running, waiting, and workspace-route capabilities through its Probe Card. Raw-terminal control is out of scope. |
| Claude Code | Hook-first observation candidate. Native permission flow remains native. It may earn session/workspace, running, waiting, and workspace-route capabilities through its Probe Card. |
| Antigravity | Passive / Observed only until a formal official integration probe earns more. |

The first `supported_observe` provider is selected after the Codex/Claude probe race. A provider shown in UI may remain Passive/Observed.

No product surface should say simply “Provider X is supported.” It must disclose the specific capability level.

---

## 7. Fuel posture

Fuel is source-gated and decomposed:

```text
reported quota window
≠ task token ledger
≠ burn meter
≠ verified limit block
```

P0 default for every provider:

```text
quota = unavailable
task tokens = unavailable
burn meter = unavailable
Fuel Thread = unavailable
```

A provider-specific Probe Card may independently enable a capability. For example, a verified reported quota window may be shown without implying current-task token usage.

Fuel never displaces waiting/failure. Only a trusted source proving that a usage limit currently blocks progress may create `verified_limit_reached`.

---

## 8. Privacy promise

Pulse stores compact local state, not user work.

It does not retain prompts, transcripts, command lines, terminal output, tool input/output, code, diffs, environment variables, credentials, raw provider payloads, browser data, or copied session history.

Safe task titles are unavailable by default and require provider-specific explicit approval. Generic provider/workspace labels are the normal fallback.

Privacy profile is a retention ceiling:

- Minimal local state may retain bounded recent terminal breadcrumbs.
- Strict local state removes terminal breadcrumb after its atomic terminal transition.
- Passive-only installs no observation integration and creates no integration breadcrumbs.

---

## 9. Runtime promise

Pulse Link is an on-demand local bridge:

```text
Hook / user request
→ Shim
→ Link
→ compact state
→ Island
```

It is not a permanent background service. No active work plus no grace period means Link exits.

When Island is absent, Link may keep a bounded breadcrumb in Drop Mode. Island receives `FullSnapshot` and `SnapshotDelta`, not raw event replay.

Any Pulse failure reduces observation only. It must not change provider behavior.

---

## 10. User controls

Users can:

- choose Passive, Minimal local state, or Strict local state
- enable or remove a provider observation integration explicitly
- pin/follow/mute tasks or workspaces
- hide workspace/task labels on Island
- enter Safe Mode
- clear Pulse local data
- export content-minimized diagnostics

Safe Mode prevents Link wake at the Shim boundary while leaving provider tools and existing provider configuration unchanged.

---

## 11. MVP definition

The first credible MVP proves one narrow loop with one provider selected by probe:

```text
User starts provider normally
→ formal Pulse integration creates a bounded breadcrumb
→ Island opens later
→ Pulse shows only verified state
→ user returns through strongest verified route
→ provider remains unaffected if Pulse fails
```

MVP does not require:

- three-provider parity
- task control
- permission approval in Island
- transcript/history parsing
- universal Fuel
- exact terminal routing for arbitrary terminal tasks
- external adapter loading
- cloud sync

---

## 12. Design invariants

1. Observe-first value is sufficient for MVP.
2. Provider release status, task health, route capability, and feature capability are independent axes.
3. Uncertainty lowers the claim rather than widening collection.
4. The Island is a signal, not a dashboard.
5. No Pulse failure can masquerade as an Agent failure.
