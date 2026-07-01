# Pulse Island · Integration and Hook Protocol

**Status:** Normative Hook/Shim/Link contract  
**Applies to:** Provider Hooks, explicit launchers, `pulse-link-shim.exe`, Pulse Link ingress, integration install/repair/uninstall  
**Depends on:** `01-privacy-data-boundaries.md`, `06-pulse-link-runtime-architecture.md`, `14-spike-c-link-transport-drop-mode.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse observes provider lifecycle signals without replacing provider behavior.

> A provider must work normally when Pulse is absent, busy, crashed, disabled, in Safe Mode, or intentionally not running.

Provider Hooks, if enabled, are observer paths. They are never gates that hold an Agent task hostage.

---

## 2. Roles

```text
Provider Hook / formal integration
→ pulse-link-shim.exe
→ pulse-link.exe
→ Event Reduction Engine
→ compact snapshot
→ Pulse Island
```

### Provider Hook

A documented provider callback/integration surface.

### Shim

A short-lived local executable. It validates the outer bounded input, honors Safe Mode, wakes Link when allowed, forwards one bounded envelope, and exits fail-open.

### Link

Owns admission, reduction, bounded breadcrumb persistence, and Island state publishing.

### Island

Consumes compact snapshots only. It never receives raw Hook input.

---

## 3. Integration modes

| Mode | Behavior |
|---|---|
| Passive | No Hook/config install. Process/window facts only. |
| Observe | User explicitly installs one provider-supported user-level Hook/integration that invokes Shim. |
| Explicit launcher | User runs `pulse <provider>` by opt-in. No PATH replacement or shell alias takeover. |

Pulse never silently replaces provider commands, edits shell profile/PATH, or adds a permanent service.

---

## 4. Hook eligibility

A provider Hook path is usable only when it is:

1. documented or formally supported by the provider,
2. user-scoped or explicitly user-approved,
3. fail-open,
4. removable without affecting unrelated config,
5. compatible with a bounded input allow-list,
6. usable without credentials, trust bypass, private endpoints, or UI automation.

If these conditions fail, Pulse falls back to Passive/Observed. It does not invent another integration path.

---

## 5. Ownership and configuration mutation

Pulse-owned configuration is identified through a provider-compatible exact command signature plus non-secret installation identifier:

```text
pulse-link-shim.exe
--provider <provider-id>
--integration-id <installation-id>
```

Rules:

- Do not add unsupported `owner`, `metadata`, or arbitrary integration keys.
- Preserve unrelated Hook entries and ordering.
- Update/uninstall locate only the exact Pulse-owned signature.
- Use targeted backup fragments, never stale full-config restoration.
- Never modify project-level config as a fallback.

Installation:

```text
read user config
→ parse/validate
→ locate existing Pulse signature
→ add/update only Pulse entry
→ atomic write
→ re-read/validate
→ non-destructive health check
```

---

## 6. Bounded Hook envelope

The Shim consumes a bounded provider input and emits only an approved Pulse envelope.

```text
PulseHookEnvelope
├── protocol_version
├── integration_id
├── provider
├── hook_event
├── occurred_at
├── session_ref (opaque)
├── turn_ref (optional opaque)
├── process_ref (optional bounded)
├── workspace_ref (optional)
├── lifecycle_hint (optional)
├── attention_hint (optional)
├── safe_summary (optional, static/provider-approved category)
├── safe_error (optional, static/provider-approved category)
├── capability_hints[]
└── bounded source metadata
```

Forbidden in every envelope:

```text
prompt text
assistant text
transcript content or transcript path as parse target
terminal output
command arguments
tool input/output
source code/diffs
credentials/cookies/auth headers
environment variables
raw provider payload blobs
unknown arbitrary object fields
```

Input is length-checked before allocation. Rejection happens before reducer state mutation.

---

## 7. Shim contract

### 7.1 Normal observation path

```text
read bounded input
→ validate outer envelope
→ check Safe Mode
→ attempt existing Link ingress pipe
→ if unavailable, start Link with inherited anonymous-pipe handoff
→ forward one bounded frame
→ exit
```

The initial event must never be placed in command-line arguments, environment variables, temporary file names, logs, or diagnostics.

### 7.2 Fail-open behavior

For ordinary observation events, Shim returns success within its hard time budget when:

- Link accepts the frame,
- Link is unavailable,
- Link fails to start,
- pipe connection times out,
- input is malformed/oversized,
- acknowledgement is invalid.

The provider continues native behavior in all cases.

### 7.3 Safe Mode

Safe Mode is checked before Link wake or forwarding:

```text
Shim sees current-user Safe Mode flag
→ no Link wake
→ no ingress forwarding
→ exits 0 within normal fail-open budget
```

Existing provider Hook configuration remains installed but inert from Pulse’s perspective. This does not change provider Hook semantics or the provider task.

---

## 8. Link transport contract

`14-spike-c-link-transport-drop-mode.md` is authoritative.

Required architecture:

```text
Shim / formal integration
→ ingress pipe

Island
→ distinct Island pipe

new Link initial event
→ inherited anonymous handoff pipe
```

Island receives only bounded state-oriented messages:

```text
HelloAck
FullSnapshot
SnapshotDelta
LinkHealth
ProtocolError
```

No raw event or `EventBatch` replay is exposed to Island.

---

## 9. Waiting and decision events

A waiting/permission Hook may create a `waiting_observed` event only when the provider formally exposes it.

P0 behavior:

```text
provider waiting event
→ Shim forwards bounded waiting signal
→ Pulse may show yellow state and route action
→ Shim returns no provider decision
→ provider continues native waiting/permission UI
```

Pulse does not return allow/deny/approve/reject, inject extra context, or replace the native decision surface in P0.

---

## 10. Protocol compatibility

- Envelope protocol carries major/minor version.
- Unknown major version rejects safely and fails open.
- Unknown optional minor fields are ignored only when schema policy permits; they are never persisted as raw extension data.
- Link/Shim support current and previous compatible Hook protocol during a rolling update window.
- Version mismatch degrades Pulse integration health, not provider task state.

---

## 11. Required acceptance scenarios

1. Link absent: Shim exits success and provider continues.
2. Malformed/oversized input: rejected before reducer; provider continues.
3. Parallel Hook calls: at most one Link instance starts.
4. New Link first event: inherited handoff path, never command line.
5. Island reconnect: `FullSnapshot` then deltas, never event replay.
6. Existing provider Hook while Safe Mode enabled: Shim exits success and does not start Link.
7. Install/update/uninstall modifies only the exact Pulse command signature.
8. Waiting Hook without Island: provider native permission behavior remains unchanged.

---

## 12. Design invariants

1. Hook failure cannot break Agent work.
2. Shim is a fail-open messenger, not an Agent gatekeeper.
3. Safe Mode stops Pulse at Shim ingress, not by mutating provider tools.
4. No Hook path carries task content into Pulse.
5. Link publishes state, not event history.
