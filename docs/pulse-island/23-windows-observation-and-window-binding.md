# Pulse Island · Windows Observation and Window-Binding Contract

**Status:** Platform behavior baseline  
**Applies to:** Passive mode, process discovery, context-route evidence, fullscreen suppression, terminal/window correlation  
**Depends on:** `02-agent-state-model.md`, `05-context-routing.md`, `06-pulse-link-runtime-architecture.md`, `09-native-island-ui-system.md`, `19-antigravity-probe-card.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Passive mode and degraded integrations still need to be useful. On Windows, Pulse can observe processes, parent/child relationships, process lifetimes, candidate windows, and workspace references when independently available.

That does **not** mean Pulse may infer task semantics from arbitrary desktop activity.

This document defines the narrow Windows observation floor:

> Observe stable operating-system facts.  
> Promote to task truth only when provider evidence independently supports it.

The contract lets Pulse provide honest `Observed`, `Workspace-ready`, and carefully verified terminal/window routes without turning into a screen scraper or a general-purpose desktop monitor.

---

## 2. What Windows observation may establish

Windows-level evidence may establish only these facts:

| Fact | Permitted use |
|---|---|
| Executable identity | Candidate provider-process discovery. |
| PID + process start time | Stable process fingerprint; PID-reuse defense. |
| Parent / child relation | Conservative process-tree correlation. |
| Process exit | Process has ended, not task outcome. |
| Current user and logon session | Ensure Pulse observes only its own local user session. |
| Candidate visible window belonging to process | Potential route target, subject to revalidation. |
| Window visibility/minimized state | Route feasibility only. |
| Monitor / fullscreen context | Island suppression behavior. |
| Current directory, when safely and legally accessible | Optional workspace anchor only. |

Windows-level evidence may **not** establish by itself:

- task is actively progressing
- task is waiting for user
- task completed successfully
- task failed
- task title
- terminal command content
- session identity
- token usage or account quota
- exact terminal tab ownership
- permission request meaning

---

## 3. Passive-mode promise

Passive mode is not a weaker implementation of Attached mode. It has its own honest product promise:

```text
Pulse can notice a compatible local process and, where independently known,
help you locate the related application, workspace, or process details.
```

### 3.1 Passive-mode UI ceiling

| Evidence | Allowed UI |
|---|---|
| Process fingerprint only | `Codex · Observed process` + `Show process details` |
| Process + safe workspace anchor | `Claude Code · Observed` + `Open workspace` |
| Process + verified host window | `Antigravity · Observed` + `Focus agent window` |
| No valid process/window | No active island item; optional recent signal only |

A passive process must never display a green “working” lamp merely because it is alive.

---

## 4. Process identity model

### 4.1 Process fingerprint

Every observed process uses a composite fingerprint:

```text
ProcessFingerprint
├── pid
├── process_started_at
├── executable_identity
├── user_sid_hash
├── logon_session_hash
├── parent_pid (optional)
├── parent_started_at (optional)
└── provider_candidate_kind
```

`pid` alone is never a stable identity.

### 4.2 Executable identity

Provider candidate detection must use a narrow signed/known executable identity policy where practical:

- executable base name
- resolved executable path only inside current-user local processing
- optional publisher or file-version metadata for diagnostics/probe validation
- provider adapter registry rule

The broad UI must not display raw executable paths. Diagnostics may show a redacted category such as:

```text
recognized_provider_binary
unknown_binary_variant
unverified_binary_path
```

### 4.3 Process-tree depth

Pulse may inspect a bounded parent/child graph around a candidate process.

Initial limits:

```text
parent traversal depth: 4
child traversal depth: 6
candidate descendants retained per root: 64
```

The goal is correlation, not machine-wide process indexing. Exceeding limits produces a degraded/ambiguous result rather than deeper scanning.

### 4.4 Current-user scope

Pulse observes only processes that belong to the current user and current logon session unless a future managed deployment explicitly defines a different consent and security model.

No cross-user or elevated-process observation is required in P0.

---

## 5. Process discovery lifecycle

### 5.1 Allowed triggers

Discovery should be event-led or explicitly user-led whenever possible:

- provider Hook gives a process hint
- Island opens and requests cold discovery
- user opens Integrations health check
- Link starts from a supported Hook and validates related process binding
- bounded periodic reconciliation while an observed candidate exists

### 5.2 Disallowed discovery behavior

Pulse must not:

- scan all processes continuously at high frequency
- keep a global historical process database
- inspect process memory
- read command line arguments into product state
- read environment variables
- enumerate unrelated process trees for analytics
- create a background service merely to detect future processes

### 5.3 Cold discovery cadence

When Island starts in Passive mode:

```text
one bounded discovery pass
→ optional one short delayed reconciliation pass
→ stop unless user requests refresh or a known candidate needs validation
```

When Link is already tracking a candidate:

```text
wait on process handle for exit
→ avoid polling process table
```

A static idle system must not pay recurring process-scan cost.

---

## 6. Process exit semantics

### 6.1 Safe mapping

A verified process exit allows only:

```text
process binding ended
→ task may be terminated / offline / expired
```

It does not allow:

```text
process exit
→ completed
process exit
→ failed
```

unless a provider adapter separately provides corroborating terminal evidence.

### 6.2 Parent/child complications

Agentic tools may spawn shells, runtimes, subagents, language servers, terminal hosts, or child workers. Therefore:

- a child exit does not imply parent task end
- a terminal host exit does not prove provider task result
- a parent may outlive a turn
- a provider may hand work to a child process

The reducer must preserve provider evidence precedence over any process-tree inference.

---

## 7. Workspace association

### 7.1 Allowed workspace anchors

A workspace route can be constructed only from one of:

- provider Hook/API-provided `cwd` or workspace URI
- user-selected workspace association
- safely observed process current directory, when the platform probe validates access and correctness
- a verified existing IDE/workspace window association

### 7.2 Workspace privacy

Pulse stores:

```text
workspace_stable_hash
short_display_name (optional)
safe_open_path_ref (short-lived where needed)
```

It does not store workspace content, repository status, branch names, recently opened files, or editor history as part of Passive mode.

### 7.3 Ambiguity

When two candidate processes share a workspace:

- do not merge them
- do not assume they are the same task
- show separate Observed entries or a workspace cluster count
- retain individual process fingerprints

---

## 8. Window observation and route evidence

### 8.1 Window ownership

A top-level window may be considered a candidate route target only when all of these hold:

1. The window belongs to the current user/logon session.
2. The owning process matches a validated process fingerprint or a strongly correlated host process.
3. The window is visible or can be restored through ordinary Windows behavior.
4. The route is revalidated immediately before user invocation.

### 8.2 What window title may do

Window title text may be used only as a low-confidence display hint for diagnostics/probe development. It may not establish:

- provider identity
- workspace identity
- task identity
- terminal tab identity
- exact original context

No product state should depend on title parsing.

### 8.3 Window route strength

| Evidence | Route strength | Allowed label |
|---|---|---|
| Provider exact deep link/session target | Exact | `Open original task` |
| Validated process owns one relevant top-level window | Strong | `Focus agent window` |
| Host terminal/window correlation uncertain | Useful at most | `Open workspace` or `Open agent` |
| Process only | Weak | `Show process details` |

A window is never “original task” proof just because it belongs to a provider executable.

---

## 9. Terminal-host correlation

Terminal correlation is valuable but exceptionally easy to overclaim.

### 9.1 P0 posture

Pulse may investigate terminal correlation as an experimental route enhancement. It is not required for the first supported Observe adapter.

### 9.2 Required evidence for a strong terminal route

All links below must be validated:

```text
provider process fingerprint
→ direct or bounded child relation
→ terminal host process fingerprint
→ one visible top-level terminal window
→ host-specific tab/session association, if available
→ route target revalidated at click time
```

If the terminal host cannot provide a stable tab/session association, Pulse may focus the host window only if it can truthfully label the action `Focus related terminal`.

It must not call this `Open original task`.

### 9.3 Explicit prohibitions

Pulse must not:

- scrape terminal screen text
- use console buffer reads as a transcript API
- send key combinations to locate a tab
- simulate mouse clicks into a terminal
- use accessibility-tree text to reconstruct command/session history
- launch a new terminal as an alleged task return

### 9.4 Route fallback

```text
exact provider route unavailable
→ strong terminal/window route unavailable
→ workspace route
→ agent route
→ process details
```

The user-facing label changes at every downgrade.

---

## 10. Foreground activation

### 10.1 User intent required

Pulse may attempt to focus another application only after explicit user action:

- click on a route action
- selected Command Palette command
- keyboard confirm on focused Pulse item

Passive state changes, notifications, or background task events never activate another window.

### 10.2 Ordinary Windows behavior

Use standard Windows foreground activation paths. If the operating system does not permit immediate foreground transfer:

- do not loop/retry aggressively
- do not synthesize input to bypass focus rules
- show a quiet route failure/fallback message
- leave provider task unchanged

### 10.3 Lost target

When a window target has been closed or process identity no longer matches:

```text
Original window is no longer available.
[Open workspace]
```

Do not retain stale HWND references as proof of context.

---

## 11. Fullscreen, presentation, and remote-session detection

### 11.1 Purpose

Pulse must reduce interruption during immersive work without pretending it knows every full-screen application perfectly.

### 11.2 Policy inputs

Use a conservative combination of:

- Windows Focus/Do Not Disturb state where safely available
- fullscreen/window coverage heuristics for the foreground application
- presentation mode policy where available
- remote desktop session context
- explicit user temporary mute

### 11.3 Behavior

When an immersive condition is active:

- hide compact Island
- stop nonessential animation
- suppress Toast candidates according to attention policy
- do not auto-focus route targets
- retain only bounded unread attention summary

The product may still allow an explicit user shortcut to open Pulse if configured.

### 11.4 Uncertainty rule

If fullscreen detection is uncertain, prefer the less interruptive behavior only when the user selected a Focused/Quiet attention mode. In Balanced mode, preserve important hard-block visibility through the next safe opportunity rather than force an overlay.

---

## 12. Platform service API boundary

`pulse-win32` should expose narrow interfaces rather than direct product-aware helper sprawl.

```text
trait ProcessObserver {
    fn discover_candidates(&self, request: DiscoveryRequest) -> Vec<ProcessCandidate>;
    fn open_exit_wait(&self, fingerprint: &ProcessFingerprint) -> Result<ExitWaitHandle, PlatformError>;
    fn validate(&self, fingerprint: &ProcessFingerprint) -> ProcessValidation;
}

trait WindowRouter {
    fn find_candidates(&self, process: &ProcessFingerprint) -> Vec<WindowCandidate>;
    fn validate_route(&self, target: &WindowTarget) -> RouteValidation;
    fn focus_user_initiated(&self, target: &WindowTarget) -> FocusResult;
}

trait ImmersiveStateProbe {
    fn current_state(&self) -> ImmersiveState;
}
```

These interfaces return evidence, not product decisions. `pulse-routing` and `pulse-arbitration` decide how to interpret the evidence.

---

## 13. Probe and acceptance scenarios

### Process identity

1. PID reuse creates a new process fingerprint.
2. Same executable in two workspaces remains two Observed candidates.
3. Child-process exit does not terminally end parent task.
4. Process exit with no provider evidence becomes terminated/offline only.

### Workspace evidence

5. Safe provider workspace anchor exposes `Open workspace`.
6. Two sessions in one workspace are not merged.
7. Workspace path is hidden from compact Island when privacy setting requires it.

### Window routing

8. Process-owned window can be focused only after user action.
9. Closed/stale HWND falls back to workspace/agent route.
10. Window title changes do not change task identity.
11. Terminal host without tab proof never receives exact-task label.

### Immersive behavior

12. Fullscreen simulation hides Island and stops animated work.
13. Leaving immersive state yields at most one summary.
14. Explicit palette shortcut remains available according to user setting.

### Privacy/resource

15. No command line/environment/terminal text enters snapshot, diagnostics, or breadcrumb store.
16. Cold discovery is bounded and does not leave a long-running process scanner.
17. Process waits use exit handles where possible rather than recurring polling.

---

## 14. Design invariants

1. Windows observation establishes operating-system facts, not agent semantics.
2. A process is not a task and a window is not a session.
3. Every route claim weakens honestly as evidence weakens.
4. Passive mode remains useful without spying on user content.
5. User intent governs focus changes.
6. When platform evidence is ambiguous, Pulse lowers confidence rather than increasing automation.
