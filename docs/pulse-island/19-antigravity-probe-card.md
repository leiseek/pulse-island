# Pulse Island · Antigravity Capability Probe Card

**Status:** Probe-only. No integration capability is presumed.  
**Provider:** Antigravity  
**Target posture:** Passive observation first; formal integration only after a current official surface is verified  
**Last updated:** 2026-07-01

---

## 1. Decision summary

Antigravity must enter Pulse Island through a stricter path than Codex CLI or Claude Code.

At the time this card was written, Pulse has **not** established a public, stable, officially documented integration surface for:

- user-level lifecycle Hooks
- externally addressable session identity
- task state callbacks
- permission/approval events
- CLI-to-IDE context deep links
- account quota or task-token telemetry
- safe external task control

Therefore the initial product posture is:

> **Antigravity starts at Passive / Observed.**
>
> It can rise only one capability at a time after a formal official surface, live probe, privacy review, and late-attach test all pass.

Pulse must not use UI automation, OCR, editor scraping, private network endpoints, memory inspection, browser-cookie extraction, or simulated keyboard/mouse input to fill missing integration gaps.

---

## 2. Release labels

| Label | Allowed user-visible meaning |
|---|---|
| `not_probed` | Pulse has no supported Antigravity evidence. |
| `process_observed` | Pulse found a likely Antigravity process only. |
| `workspace_ready` | Pulse can safely open a related workspace/folder, not the exact task. |
| `experimental_attached` | A formal integration provides a verified current state under stated limitations. |
| `supported_observe` | Required lifecycle, Late Attach, rollback, privacy, and resource gates pass. |
| `supported_fuel` | A separately scoped official usage source is verified. |
| `supported_control` | A formal narrow control surface passes independent review. Not P0. |

A provider process showing up in Task Manager is not `Attached`.

---

## 3. Capability assumptions: all false until proven

The adapter must begin from this matrix, not from an optimistic abstraction.

| Capability | Initial status | Upgrade evidence required |
|---|---|---|
| Discover process | Probe candidate | Windows process identity test |
| Discover workspace | Probe candidate | Safe workspace association, no title scraping |
| Discover session | Unavailable | Formal stable session identifier |
| Observe running | Unavailable | Formal active/lifecycle source |
| Observe waiting | Unavailable | Formal user-input/permission event |
| Observe completion | Unavailable | Explicit verified terminal outcome |
| Observe failure | Unavailable | Explicit verified terminal failure outcome |
| Open workspace | Probe candidate | Verifiable workspace anchor |
| Open exact task | Unavailable | Official deep link or strongly verified exact window binding |
| Session tokens | Unavailable | Formal bounded session telemetry |
| Quota snapshot | Unavailable | Official scoped rate-limit/usage source |
| Approval control | Unavailable | Formal decision bridge with native fallback |
| Stop/steer/resume | Unavailable | Formal lifecycle-control API |

No missing cell is a reason to infer state from visual editor behavior.

---

## 4. Probe order

Antigravity should be investigated in the lowest-risk sequence below. A failed earlier stage blocks later claims but does not prevent shipping a lower truthful mode.

```text
A0. Product and install surface inventory
A1. Passive process discovery
A2. Workspace association
A3. Official integration registration / rollback
A4. Session identity and lifecycle event probe
A5. Late Attach probe
A6. Context route probe
A7. Fuel source probe
A8. Fault, privacy, and resource probe
```

---

## 5. A0: Official surface inventory

### Goal

Identify whether an official, current, user-installable integration surface exists.

### Required questions

| Area | Probe question |
|---|---|
| CLI | Is there a supported CLI command and documented event/integration surface? |
| IDE | Is there a documented extension API, command API, URI scheme, task API, or agent API? |
| Agent manager | Is there a formal way to enumerate active agents/tasks? |
| Hooks | Can a user register lifecycle Hooks without project-file modification? |
| Session identity | Is an opaque stable session/task ID available externally? |
| Permissions | Is there a formal user-attention or permission-request callback? |
| Context route | Can an exact agent task or workspace be opened through a supported route? |
| Usage | Is task-token or quota information exposed with defined scope and reset semantics? |
| Control | Is any task stop/resume/steer method explicitly documented? |
| Install | Is the integration user-level, reversible, and non-admin? |

### Evidence standard

A candidate surface is acceptable only when it is:

- official and current
- documented or formally declared in the product’s extension/integration model
- usable under normal user permissions
- compatible with a local-only, fail-open observer
- available without credential extraction or a private endpoint

If only marketing material or visual UI behavior is found, the answer is `not_probed`, not `supported`.

---

## 6. A1: Passive process discovery

### Goal

Establish the honest baseline without modifying Antigravity.

### Test

```text
Start Antigravity normally in a synthetic workspace
→ enumerate candidate process tree
→ capture executable identity, PID, process start time, parent relation
→ close normally
→ force a separate process exit path
```

### Allowed result

```text
Antigravity · Observed process
Started 3m ago
[Show process details]
```

### Not allowed

- `Running` merely because process exists
- a task title inferred from window title
- waiting/completed/failed status
- exact original-session action
- token or quota display

### Acceptance

- PID reuse is protected by process start time.
- Candidate ambiguity produces separate Observed entries rather than merge.
- No command-line arguments or environment data are stored/displayed.

---

## 7. A2: Workspace association

### Goal

Determine whether a workspace anchor can be obtained without screen scraping or undocumented file inspection.

### Acceptable evidence paths

- documented CLI/IDE workspace metadata
- formal extension API returning workspace URI/path
- user-approved path supplied through an explicit UI action
- process current directory only when Windows-level evidence is available and safe to use

### Disallowed evidence paths

- window-title parsing
- editor accessibility-tree scraping to read filenames
- scanning recent workspaces/history databases
- reading arbitrary extension state files
- OCR/screenshot analysis

### Allowed result

```text
Antigravity · Observed
Workspace route available
[Open workspace]
```

The route remains `workspace_ready`, never `context_ready`.

---

## 8. A3: Official integration registration and rollback

### Goal

Prove that Pulse can install exactly one Antigravity integration entry without owning its configuration.

### Prerequisite

A current official registration mechanism must be identified. Examples could include a documented user-level Hook, extension registration, local integration manifest, or SDK configuration.

### Required transaction

```text
read user-level integration configuration
→ parse and validate
→ create targeted backup fragment
→ add only Pulse-owned entry
→ write atomically
→ re-read and validate
→ run non-destructive health test
```

### Hard blocks

Stop the probe if integration requires:

- a project-file modification by default
- administrator installation
- managed-policy bypass
- shell/PATH interception
- provider credential extraction
- hidden experimental/private config key
- broad rewrite of user configuration

### Acceptance

- Existing third-party/user entries keep their content and ordering.
- Pulse can identify and remove only its own entry.
- Interrupted installation preserves a valid original or valid updated file.
- Antigravity continues operating normally if Pulse Link is absent.

---

## 9. A4: Session identity and lifecycle event probe

### Goal

Map only formal events into Pulse normalized events.

### Required event table

For every candidate provider event, capture:

```text
provider event name
source/version
allowed source fields
stable session/task identity availability
workspace identity availability
lifecycle effect
attention effect
terminal confidence
freshness expectation
safe metadata subset
explicitly dropped fields
```

### Minimum lifecycle exercise

```text
session/task start
ordinary active work
ordinary quiet interval
user-needed state if formally exposed
normal end
clear failure
process/session disappearance without outcome
integration disconnect and recovery
```

### Mapping rules

| Provider evidence | Allowed Pulse mapping |
|---|---|
| Formal task start + stable session ID | `session_started`, candidate Attached |
| Formal activity heartbeat | `activity_observed`, running freshness |
| Formal permission/input request | `waiting_observed`, yellow state |
| Formal explicit successful completion | `completion_observed` |
| Formal explicit failure result | `failure_observed` |
| Process exit alone | `terminated`, never completed/failure |
| Silence/time gap | `stalled` watch only |

### Explicit anti-patterns

Pulse must not map:

```text
window focus / editor activity → running task
agent panel text → lifecycle event
spinner visibility → running
missing spinner → completed
notification toast → verified failure
```

---

## 10. A5: Late Attach

### Goal

Prove the user promise only when a formal integration has already created Link breadcrumbs.

### Positive path

```text
Pulse integration installed
→ start Antigravity task normally while Island is closed
→ formal event wakes Link
→ task becomes active
→ start Island later
→ request snapshot
→ verify session identity, lifecycle, health, and route label
→ restart Island
→ verify reconnect
```

### Negative path

```text
Start Antigravity task before Pulse integration/Link exists
→ open Island later
→ run cold discovery
→ show only process/workspace evidence that can be independently proven
```

### Acceptance

- No task restarts.
- No duplicate task is created.
- No new Antigravity session is spawned.
- No later state is fabricated from absent event history.
- Link restart restores breadcrumb state as `degraded` until fresh provider evidence arrives.

---

## 11. A6: Context routing

### Priority chain

```text
1. Official exact task/session deep link
2. Official focus/open command for exact task
3. Strongly verified original window binding
4. Open related workspace
5. Open Antigravity application
6. Show process details
```

### Route-label contract

| Evidence level | Allowed label |
|---|---|
| Exact official session/task route | `Open original task` |
| Strong verified original window | `Focus agent window` |
| Workspace only | `Open workspace` |
| Application only | `Open Antigravity` |
| Process only | `Show process details` |

### Explicit prohibitions

- Do not launch a new CLI task as fallback.
- Do not open the app generically and call it task recovery.
- Do not synthesize a URI/deep-link format from reverse engineering.
- Do not infer exact tab ownership from title strings alone.

---

## 12. A7: Fuel source probe

Fuel must be treated as absent until scope and provenance are clear.

### Separate questions

```text
Can Antigravity report task-scoped token counts?
Can it report account/provider quota windows?
Can it report reset time?
Can it report an actual rate/usage block for a task?
Are values official, local observed, or estimated?
```

### P0 posture

```text
Quota snapshot: Unavailable
Task token ledger: Unavailable
Official usage route: Available only if a documented route exists
```

### Acceptance for `supported_fuel`

- source is official or formally documented
- account/session scope is unambiguous
- independent quota windows remain independent
- source freshness is known
- reset behavior is known
- no credentials/cookies/private endpoint is used
- usage collection stays inside Link resource budget

No generic percentage, no synthetic runway, and no Fuel Thread before these pass.

---

## 13. A8: Fault, privacy, and resource probe

### Fault injection

```text
Pulse Link absent
Shim/bridge timeout
integration event malformed
integration source disconnects
Island crashes
breadcrumb write fails
user removes Pulse integration entry
Antigravity update changes/invalidates integration surface
```

### Required behavior

- Antigravity keeps native task behavior.
- Pulse reduces itself to Degraded/Observed.
- No user task is held waiting on Pulse.
- No automatic retries become a background daemon.
- No source content is dumped into logs/diagnostics.

### Resource validation

When a formal source exists, test:

- Drop Mode memory/CPU budget
- event-storm coalescing
- no background file scanning
- no rendering/UI allocation in Link
- clean 90-second idle exit
- bounded breadcrumb file

---

## 14. Initial user experience matrix

| Probe outcome | Island wording | Available action |
|---|---|---|
| No candidate process | Hidden / no state | None |
| Candidate process | `Antigravity · Observed process` | Show process details |
| Workspace independently known | `Antigravity · Observed` | Open workspace |
| Formal activity source, no terminal source | `Antigravity · Working` with Attached badge | Open workspace / application |
| Formal waiting source | `Antigravity · Waiting` | Open exact route if verified, otherwise workspace/app |
| Formal terminal source | Completed / Failed per verified mapping | Relevant route |
| Official usage source | Fuel source label, independent window | Open official usage |

No wording should imply an Agent session is controllable unless a later formal control probe proves it.

---

## 15. Release decision rules

### `process_observed`

Requires only process discovery and privacy-safe process presentation.

### `experimental_attached`

Requires a documented official event source plus passing fault and privacy tests, but may still lack complete Late Attach or terminal mapping.

### `supported_observe`

Requires all of:

- safe user-level installation/rollback
- stable session identity
- verified active and waiting lifecycle semantics where advertised
- Late Attach with Island restart/reconnect
- no false completed/failed mapping
- honest workspace/exact-route labels
- fail-open provider behavior
- bounded Drop Mode resource use
- no content ingestion beyond approved metadata

### `supported_fuel`

Requires a separate official scoped usage source. It is not implied by `supported_observe`.

---

## 16. Design invariants

1. Antigravity is not a blank canvas for clever integrations.
2. Formal provider capability is always stronger than visual inference.
3. Process presence is a valuable floor, not an embarrassing failure.
4. Every unavailable feature stays visibly unavailable rather than being approximated by automation.
5. Passive Observed mode can ship before Attached mode if it is honest and useful.
6. Antigravity may never lower Pulse’s privacy, fail-open, or resource requirements.
