# Pulse Island · Privacy and Data Boundaries

**Status:** Normative privacy contract  
**Applies to:** All Pulse binaries, adapters, IPC, persistence, diagnostics, installation, and recovery  
**Depends on:** `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Principle

Pulse is local-first and content-minimizing.

> Pulse records enough state to tell the user that work exists, needs attention, or can be returned to. It does not collect the work itself.

Provider capability, UI usefulness, diagnostics, and convenience never justify retaining task content that Pulse does not need.

---

## 2. Data classes

### 2.1 Allowed bounded task metadata

Only when needed for a verified capability:

```text
provider id
opaque task/session reference or hash
safe lifecycle/attention/health/context state
bounded timestamps and durations
process fingerprint (PID + start time, never command line)
workspace stable hash
optional short workspace display label
provider-approved safe category/reason
route strength and bounded route reference
feature capability states
Fuel provenance and bounded numeric source values when approved
```

### 2.2 Content Pulse never retains by default

```text
prompts
assistant text
transcripts and transcript paths as content sources
terminal buffers
commands and command arguments
tool input or tool output
code, diffs, source snippets
plan text
environment variables
credentials, API keys, cookies, auth headers
browser/session storage
raw provider payload blobs
```

This prohibition applies equally to in-memory queues, IPC, breadcrumbs, normal logs, diagnostics, crash markers, and support exports.

---

## 3. Safe display text

A safe task title is unavailable by default.

Pulse may display a task title only when a provider-specific Probe Card explicitly approves a bounded source field as safe for display. Prompt text, command text, transcript text, tool input/output, assistant text, window title parsing, and terminal text are never title sources.

When no safe title source exists, use generic display identity:

```text
Codex CLI · Waiting
Claude Code · Observed
Atlas workspace · Running
```

Safe summaries and errors must use provider-approved category mappings or static product text, not raw error/output strings.

---

## 4. Ingress and IPC boundary

Raw provider input must be reduced at the earliest Pulse-owned boundary.

```text
Provider Hook / API signal
→ Shim / Adapter allow-list
→ bounded Pulse envelope
→ Link admission
→ NormalizedEvent
→ reducer
→ compact snapshot
```

Rules:

- validate payload length before allocation
- reject forbidden fields before state mutation
- no arbitrary JSON/object escape hatch
- no raw payload persistence for retry/replay
- Island receives only `FullSnapshot`, `SnapshotDelta`, `LinkHealth`, and `ProtocolError`
- Island cannot request raw or normalized event replay

---

## 5. Storage boundary

Pulse persistence is a bounded compact-state cache, not an event history store.

Allowed persisted categories:

```text
preferences
integration registry
Pulse-owned configuration backup fragments
bounded breadcrumbs
aggregate diagnostic counters
redacted performance metrics
safe error categories
```

Disallowed persisted categories:

```text
append-only event log
transcript log
prompt history
command history
raw Hook input
provider configuration full copies
session history scrape
code/diff archive
```

All files are current-user scoped and size-capped. A failed write must not fall back to a larger or raw spool.

---

## 6. Privacy profile retention ceiling

Privacy profile is a ceiling. More restrictive profile rules override all default retention/grace behavior.

### 6.1 Minimal local state

May retain:

- nonterminal active-task breadcrumb
- immediate terminal checkpoint
- bounded recent-terminal breadcrumb
- bounded recent-signal summary

Restored breadcrumb state begins degraded until fresh evidence arrives.

### 6.2 Strict local state

May retain nonterminal active-task breadcrumb only.

At terminal transition:

```text
perform bounded atomic terminal transition
→ remove terminal task breadcrumb
→ do not retain recent-terminal entry
→ do not retain recent-signal entry
→ do not resurrect terminal state after restart
```

### 6.3 Passive-only

```text
No observation integration install.
No integration breadcrumb creation.
Only bounded user-triggered/current validation for process/window facts.
```

---

## 7. Provider integration configuration

Pulse modifies only the narrow user-level configuration required for a user-enabled integration.

Ownership is identified by a provider-compatible Pulse command signature plus non-secret installation identifier, for example:

```text
pulse-link-shim.exe
--provider <provider-id>
--integration-id <installation-id>
```

Rules:

- never add unknown owner/metadata keys when provider schema does not support them
- update/uninstall locate only the exact Pulse signature
- preserve unrelated Hook entries and provider configuration
- never restore a stale entire backup over current provider settings
- never modify project-level configuration as a fallback
- no PATH replacement, shell alias, administrator service, trust bypass, or credential extraction

---

## 8. Diagnostics and crash boundaries

Normal diagnostics may include:

```text
Pulse version/build
provider integration health
safe category codes
protocol versions
aggregate resource metrics
breadcrumb counts/sizes
capability flags
```

They exclude task titles, full workspace paths, session IDs, provider configuration contents, prompts, transcripts, tool data, credentials, and raw stack/provider strings by default.

No automatic diagnostic or crash upload exists in P0. User exports are previewed and local-first.

---

## 9. Safe Mode

Safe Mode reduces Pulse behavior without expanding collection:

```text
Shim sees Safe Mode
→ no Link wake
→ no ingress forwarding
→ exit 0

Island
→ Passive mode only
```

Existing provider Hook configuration remains untouched until user action. Safe Mode never reads provider history to reconstruct missed work.

---

## 10. Privacy acceptance tests

1. Prompt-like, secret-like, and oversized input is rejected before task state mutation.
2. No raw Hook JSON appears in breadcrumb, logs, diagnostics, crash marker, or command line.
3. Strict mode removes terminal breadcrumb and prevents restart resurrection.
4. Passive-only creates no integration breadcrumb.
5. A provider configuration update removes only Pulse-owned signature entries.
6. Default support export excludes task title, workspace path, session ID, and provider configuration text.
7. Safe Mode Hook invocation exits successfully without starting Link.

---

## 11. Design invariants

1. Pulse observes task state, never task content.
2. Safe titles require explicit provider approval.
3. Privacy profile limits every retention path, including recovery.
4. Missing data is unavailable rather than inferred from content.
5. A failure cannot create a raw-data fallback.
