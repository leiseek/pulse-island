# Pulse Island · Codex CLI Capability Probe Card

**Status:** Evidence-backed probe plan, not a release-support claim  
**Provider:** OpenAI Codex CLI  
**Target integration posture:** Hook-first observation, process/window fallback, optional Pulse-managed App Server track  
**Last updated:** 2026-07-01

---

## 1. Decision summary

Codex CLI is a strong candidate for the first Pulse provider probe because it has official command Hooks, a documented App Server, and an open-source implementation. The correct first integration is nevertheless narrow:

> **P0 Codex = user-level Hook breadcrumbs + safe lifecycle observation + workspace routing + passive process correlation.**

Do not begin by trying to control or take over existing terminal sessions.

The official Hook surface can provide a session identifier, working directory, event name, turn identifier for turn-scoped events, and permission-mode information. It also exposes permission-request and stop events. However, raw Hook input can contain sensitive prompt, tool-input, assistant-message, and transcript-path fields. Pulse must forward only the small allow-listed subset defined below.

The official App Server is suitable for rich clients and Pulse-managed sessions. It has thread lifecycle/status events, approvals, and account rate-limit methods. But an independently launched terminal task is not automatically a Pulse-owned in-flight App Server thread. `thread/resume` reopens a stored thread for later turns; it is not evidence that Pulse can safely take over an already-running external terminal turn.

---

## 2. Evidence register

This probe card is based on the following official OpenAI documentation, reviewed on 2026-07-01:

| Source | Relevant validated facts used by this card |
|---|---|
| Codex Hooks | User/project hook locations, command Hook input shape, lifecycle events, trusted command-Hook behavior, Windows command override, permission and stop semantics. |
| Codex App Server | JSON-RPC protocol, stdio default transport, thread lists/status, lifecycle notifications, account rate-limit interface, and explicit experimental WebSocket posture. |
| Codex CLI repository / documentation index | Codex CLI runs locally and the current documentation organizes Hooks, App Server, configuration, and CLI surfaces as separate official integration paths. |

The actual probe report must record the exact Codex version under test. Documentation evolves, generated schemas are version-specific, and the release behavior reference must win over source-code assumptions.

---

## 3. Integration modes

### 3.1 Mode A: Official user-level command Hooks, P0 candidate

Pulse installs only a user-level Codex Hook entry, preferably in the user Hook file rather than a project repository. It never adds a project-local Hook by default.

```text
Codex Hook
→ pulse-link-shim.exe
→ pulse-link.exe
→ Pulse breadcrumb + reducer
```

Properties:

- compatible with normal `codex` CLI launch
- gives a stable Hook session identifier
- gives session working directory
- allows lifecycle/activity/waiting observation
- must fail open
- can start Link before Island exists

Constraints:

- command Hooks may need Codex trust review before they run
- Hooks can be disabled by user/admin configuration
- multiple matching command Hooks can run concurrently
- Hook configuration must be merged surgically rather than replacing other Hook entries

### 3.2 Mode B: Passive process/workspace observation, P0 fallback

No Hook installation.

```text
process observation
+ conservative parent/process-start-time correlation
+ optional workspace association
→ Observed only
```

This mode may show a safe process-level presence and possibly a workspace route. It must not claim `Attached`, `waiting_user`, exact session identity, completion, failure, or quota.

### 3.3 Mode C: Pulse-managed Codex App Server, P1 candidate

Pulse explicitly starts an official `codex app-server` process and owns the threads/turns created through that process.

```text
Pulse-managed UI/session
→ codex app-server (stdio)
→ thread/start or thread/resume
→ turn events / thread status / optional formal control
```

This mode may later provide richer task status, exact App Server context, rate-limit snapshots, and carefully reviewed control actions. It is **not** the P0 path for a user who launched `codex` independently in a terminal.

### 3.4 Mode D: Account usage probe through a Pulse-owned App Server, experimental Fuel candidate

A short-lived or reused Pulse-owned official App Server may be probed for account rate-limit data only after the probe verifies all of the following:

- it does not alter or resume user threads
- it accesses only the account data surfaced by the official App Server
- it stays inside Link resource budget
- it does not collect or store provider credentials
- it provides meaningful quota-window scope and reset data

This is independent from Hook-based task observation.

---

## 4. User-level Hook installation plan

### 4.1 Target location

Use the user-level Codex Hook configuration surface. Do not modify:

- repository-local `.codex` configuration by default
- shell profile
- PATH command precedence
- global administrator-managed configuration

### 4.2 Pulse-owned Hook identity

Codex Hook schemas do not need arbitrary unknown metadata keys. Pulse identifies its own entry through a deterministic command signature:

```text
pulse-link-shim.exe
--provider codex-cli
--integration-id <non-secret Pulse installation identifier>
```

The integration identifier is not an account credential and not a task/session identifier. It is used only to locate the exact Pulse-owned command entry during update and uninstall.

### 4.3 Command configuration rules

- Use Codex’s Windows-specific command field when required by the official configuration shape.
- Set a short explicit Hook timeout compatible with Shim fail-open behavior. Never rely on the provider’s long default timeout.
- Configure only officially documented command Hook types.
- Do not set `async: true`; it is not a supported execution path for command Hooks.
- Do not use managed-hook-only configuration, trust bypasses, or permission bypass modes.

### 4.4 Installation transaction

```text
read user config
→ parse exact current Codex config shape
→ locate existing Pulse command signature
→ preserve all unrelated Hook entries and ordering
→ insert/update only Pulse entry
→ atomic write
→ re-read + validate
→ run non-destructive Hook health check
```

Uninstall removes only the exact Pulse command signature. A parse failure, managed-only policy, or disabled Hook feature results in `needs_repair` or `unavailable`, not an invasive configuration workaround.

---

## 5. Strict Hook allow-list

Codex command Hooks receive JSON on stdin. Pulse must not forward raw stdin to Link.

### 5.1 Allowed common fields

| Raw Hook field | Pulse handling |
|---|---|
| `session_id` | Accept as opaque source identity; convert to task key/hash outside broad persistence. |
| `cwd` | Normalize to workspace identity; retain display label/path only under privacy rules. |
| `hook_event_name` | Accept and map through static provider map. |
| `turn_id` | Accept only for event correlation; bounded and opaque. |
| `permission_mode` | Accept as low-sensitivity state context; never use to grant control. |
| `source` on SessionStart | Accept as a small lifecycle reason: startup/resume/clear/compact. |
| `agent_id` / `agent_type` for subagents | Accept only for bounded subagent aggregation, not transcript reading. |
| `tool_name` for relevant tool events | Accept as a coarse safe category. |
| `tool_input.description` on PermissionRequest | Optional, truncate/redact as a human-readable safe reason only after sanitizer. |

### 5.2 Explicitly forbidden or dropped fields

| Raw Hook field | Reason |
|---|---|
| `transcript_path` | Transcript format is not a stable Hook interface and Pulse does not ingest transcripts. |
| `prompt` from UserPromptSubmit | User content; do not retain or forward. |
| `tool_input.command` and arbitrary `tool_input` | Can contain commands, secrets, paths, or sensitive arguments. |
| tool output / command output | Content retention is out of scope. |
| `last_assistant_message` | Provider output content; do not ingest. |
| subagent transcript path | Same transcript prohibition. |
| model name | Drop in P0 unless a later product need justifies its privacy and UI value. |
| unknown fields | Reject at adapter boundary or ignore before Link transport; never persist. |

### 5.3 Envelope generated by the Shim

```text
PulseHookEnvelope
├── protocol_version
├── provider = codex_cli
├── integration_id
├── hook_event
├── occurred_at
├── session_ref
├── turn_ref (optional)
├── workspace_ref (optional)
├── lifecycle_hint (optional)
├── attention_hint (optional)
├── safe_summary (optional)
├── safe_error (optional)
├── capability_hints[]
└── source_metadata (bounded)
```

The envelope contains no prompt, transcript path, tool input, tool output, raw JSON blob, or provider credential material.

---

## 6. Hook-to-Pulse event map

This map is intentionally conservative. It records observation, not control.

| Codex Hook | Pulse event(s) | P0 lifecycle effect | What Pulse must not infer |
|---|---|---|---|
| `SessionStart` | `session_started` | `starting`, then `running` only after meaningful activity | completion, exact terminal window, task title from transcript |
| `UserPromptSubmit` | `activity_observed` | task becomes/re-enters `running` | retain user prompt or derive full task title from it |
| `PreToolUse` | `activity_observed` | refresh running freshness | actual command, final tool success/failure, provider approval result |
| `PostToolUse` | `activity_observed` | refresh running freshness | whole-task completion/failure from one tool result, even if Bash returned non-zero |
| `PermissionRequest` | `waiting_observed` | `waiting_user`, `needs_user` | auto-approve/deny, capture tool arguments, replace native approval UI |
| `SubagentStart` | parent activity + optional subagent count | parent stays active; P1 may build child observation | separate top-level task by default |
| `SubagentStop` | parent activity / subagent count update | no terminal result for parent | parent completion or subagent output summary |
| `Stop` | `turn_stop_observed` | mark terminal candidate / freshness boundary | `completed` solely from Hook event |

### 6.1 Why `Stop` is not automatic completion

Codex Stop Hooks can influence continuation and matching Hooks can run concurrently. Pulse always returns an empty successful result, but it cannot assume another Hook did not continue the turn or that a provider state transition fully completed. Therefore P0 maps Stop to a candidate event, not a guaranteed `completed` state.

A formal terminal outcome requires one of:

- a verified provider lifecycle/status source with explicit final status
- corroborated process/session result defined by the Adapter probe
- Pulse-managed App Server `turn/completed` event

Absent that evidence, use `terminated`, recent turn-ended information, or degraded state rather than false completion.

---

## 7. Permission-request policy

`PermissionRequest` is valuable for the Pulse yellow state, but it must remain native-provider-owned in P0.

### 7.1 P0 behavior

```text
Codex PermissionRequest
→ Pulse Shim sends bounded waiting event
→ Link updates waiting breadcrumb
→ Shim emits no allow/deny decision
→ Codex continues normal approval flow
```

Pulse can show:

```text
Codex is waiting for confirmation
Open original context
```

Pulse cannot show in P0:

```text
[Approve]
[Reject]
[Run command]
```

### 7.2 Safety constraints

- Never return `allow` or `deny` from the Pulse Hook in P0.
- Never forward tool input command/arguments to Island.
- Do not rely on description text existing for every request.
- If Link or Island is absent, normal Codex approval remains unchanged.
- If several Hooks provide decisions, Codex behavior follows its own native conflict rules; Pulse has no decision output and cannot override them.

### 7.3 Future P2 bridge

A formal in-Island decision bridge can be investigated only after an adapter-specific safety review. It would require a context-complete official decision surface, user opt-in, strict timeout, no default approval, and native fallback. This is not part of Codex P0 or P1 observation work.

---

## 8. Lifecycle confidence model

### 8.1 What Hooks can establish in P0

| Pulse capability | Initial target | Evidence |
|---|---|---|
| Discover session | Supported candidate | `session_id` in command Hook input |
| Discover workspace | Supported candidate | Hook `cwd`, normalized locally |
| Running freshness | Supported candidate | SessionStart plus prompt/tool lifecycle activity |
| Waiting for user | Supported candidate | PermissionRequest Hook, no Pulse decision output |
| Safe coarse reason | Supported candidate | Hook event name, tool category, optional sanitized description |
| Process correlation | Experimental candidate | Windows parent/start-time observation, must be empirically verified |
| Completion | Not assumed | Stop alone is insufficient for final truth |
| Failure | Not assumed | One tool failure/non-zero Bash is not whole-turn failure |
| Exact terminal context | Experimental candidate | Requires verified process-to-terminal/window correlation |

### 8.2 Health rules

- `Attached` requires a validated Hook session identity and fresh lifecycle event stream.
- `Observed` applies to process-only/cold discovery.
- `Degraded` applies when a known Hook session goes stale, configuration disables Hooks, Hook delivery fails, or identity becomes ambiguous.
- A session restored from Pulse breadcrumb after Link restart begins `Degraded` until fresh Codex Hook evidence arrives.

---

## 9. Context routing plan

### 9.1 P0: workspace-first

The stable P0 route is:

```text
Codex Hook session_id + cwd
→ workspace anchor
→ Open workspace / reveal project folder
```

The action label is `Open workspace`, not `Open original task`.

### 9.2 P0.5: related terminal/window focus, experimental

Pulse can investigate a strong route when all of these hold:

- Hook Shim can be correlated to a live Codex process via verified parent/process-start-time evidence.
- That Codex process can be correlated to exactly one terminal/application window through a supported Windows process/window relation.
- The relationship remains valid at invocation time.

If any link is missing, fall back to workspace route. Window title text alone is not enough.

### 9.3 P1: Pulse-managed App Server thread route

When Pulse owns the App Server connection and created/resumed the thread itself, it may label an exact App Server thread route truthfully. This must remain separate from a raw terminal-launched Codex session.

### 9.4 Explicit prohibitions

- Do not launch a new `codex` CLI command to simulate returning to a task.
- Do not call `thread/resume` on a stored thread merely to claim it has attached to a current external terminal turn.
- Do not parse transcripts to find task state or terminal history.
- Do not simulate keyboard/mouse input into the original terminal.

---

## 10. App Server track

### 10.1 Appropriate use

Codex App Server is an official JSON-RPC rich-client interface. It can expose thread lifecycle/status notifications, exact final turn statuses for the sessions it owns, and account rate-limit APIs.

Pulse should use it in two bounded contexts:

1. **Pulse-managed sessions, P1:** Pulse launches/owns the App Server and its threads.
2. **Fuel probe, experimental:** Pulse launches a short-lived or reused official App Server only to request official account rate-limit information, after proving this does not alter user task state.

### 10.2 Transport policy on Windows

Use the default stdio JSONL transport for a Pulse-owned App Server process.

Do not use:

- experimental/unsupported WebSocket transport as P0 control plane
- non-loopback listener
- raw bearer token arguments
- an App Server listener exposed to the network

### 10.3 Thread-history privacy policy

The App Server can list/read stored threads and can load full turn/item content. Pulse must not use these APIs to ingest history for general observation.

Allowed P0/P1 use:

- bounded thread identity/status metadata where a Pulse-managed context requires it
- source-kind/cwd filtering during a controlled probe
- no turn content fetch
- no `itemsView: full`
- no raw reasoning, command output, diff, or transcript storage

### 10.4 Pulse-managed lifecycle map

| App Server event | Pulse mapping |
|---|---|
| `thread/started` | session started / Attached |
| `thread/status/changed` with active | running; `waitingOnApproval` maps to waiting only in Pulse-managed session |
| `turn/started` | running |
| `turn/completed` status completed | completed |
| `turn/completed` status failed | failed with sanitized category only |
| `turn/completed` status interrupted | terminated/interrupted, not completed |
| `turn/interrupt` success | control result only for Pulse-managed session, future P1/P2 |

No App Server event is used as proof of a raw independently launched terminal task without an explicit verified linkage.

---

## 11. Fuel probe plan

### 11.1 Reported quota windows

The official App Server documents `account/rateLimits/read` and a corresponding update notification. Returned data can include per-bucket used percentage, window duration, reset timestamp, and a provider-classified reached-limit type.

Pulse Fuel can treat this as `Reported` only when the probe verifies:

- the returned bucket is associated with the currently authenticated official Codex client context
- refresh does not alter any active task
- account/window scope is represented separately for each limit ID
- server-supplied reset timestamp is preserved
- the result is fresh enough for display

### 11.2 Task tokens

The official App Server also exposes account-level usage summary/daily buckets. This is **not** a task-scoped token ledger and must not be displayed as tokens consumed by the currently selected Codex task.

Task-specific token observation remains unavailable until a formal, bounded task source is verified. Do not derive it from raw transcript parsing.

### 11.3 Fuel UI ceiling after probe

| Probe result | Allowed Pulse UI |
|---|---|
| Rate-limit read verified | independent `Reported` quota window(s), reset time, source label |
| Rate-limit source unavailable/stale | `Unavailable` + Open official usage route |
| Account usage summary only | optional account activity screen later; not task Fuel |
| No task token source | omit current-task token count |
| Official reached-limit associated to task block | lifecycle `limited`, red state |

---

## 12. Codex probe scenarios

### CXP-1: User Hook install and rollback

```text
prepare user config with unrelated Hook entries
→ install Pulse user Hook command
→ verify Codex trust/review behavior is respected
→ run Codex normally
→ update Pulse entry
→ uninstall Pulse entry
→ verify unrelated hooks remain unchanged
```

### CXP-2: Hook breadcrumb with Island closed

```text
Pulse Hook installed
→ run normal Codex CLI task
→ confirm SessionStart reaches Link
→ create activity through prompt/tool lifecycle
→ start Island later
→ verify session/workspace/running state
```

### CXP-3: Native approval remains native

```text
trigger a Codex approval-requiring action
→ PermissionRequest reaches Link
→ Island absent, then present
→ verify Pulse reports waiting
→ verify Codex native approval continues without Pulse decision
```

### CXP-4: Terminal semantics

```text
normal turn stop
tool command exits non-zero but task continues
provider task reaches clear failure
terminal process exits without explicit result
```

Required result:

- no false task `completed` from Stop alone
- no false task `failed` from one PostToolUse/non-zero command alone
- unknown exit maps to terminated/degraded until corroborated

### CXP-5: Late attach and restart

```text
start normal Codex task with Link installed but Island closed
→ open Island after activity
→ restart Island
→ verify current breadcrumb restored
→ restart Link
→ verify restored state starts degraded until fresh Hook evidence
```

### CXP-6: Context route truthfulness

```text
attempt exact terminal correlation
→ if verified: focus existing original context
→ if not: Open workspace fallback
```

The report must include a screenshot-free route evidence table and the exact user-visible labels used.

### CXP-7: Fuel report

```text
start Pulse-owned official App Server probe
→ call account/rateLimits/read
→ observe update notification if available
→ validate limit IDs/window/reset semantics
→ confirm no user thread/terminal changes
```

### CXP-8: Fault injection

```text
Link absent
Shim timeout
malformed Hook stdin
Hook disabled in config
Hook config conflict
App Server unavailable
rate-limit read error
```

Required result: Codex task behavior remains provider-native; only Pulse health/capability status degrades.

---

## 13. Initial capability decision matrix

This is a target matrix to validate, not a released claim.

| Capability | Initial release ceiling | Gate to raise it |
|---|---|---|
| Process discovery | Passive Observed | process fixture and Windows binding probe |
| Session identity | Hook Attached candidate | CXP-2 and session collision tests |
| Running freshness | Hook Attached candidate | CXP-2 plus quiet/staleness tests |
| Waiting user | Hook Attached candidate | CXP-3 native approval pass-through |
| Completion | Degraded/terminated only initially | explicit corroborated terminal evidence or managed App Server session |
| Failure | Degraded/terminated only initially | explicit corroborated terminal evidence or managed App Server session |
| Workspace route | Workspace-ready candidate | CXP-6 route verification |
| Exact original context | Experimental | verified terminal/window or Pulse-managed thread route |
| Reported quota window | Experimental Fuel | CXP-7 official App Server probe |
| Task token ledger | Unavailable | formal task-scoped source only |
| Stop/steer/resume | Unavailable for raw CLI | Pulse-managed App Server only after separate review |
| Approve/deny | Unavailable in P0 | separate formal control bridge review |

---

## 14. Release decision rules

### Eligible for `supported_observe`

Codex Hook mode reaches `supported_observe` only after all of these pass:

- exact user-level install/update/uninstall behavior
- Hook trust requirements respected
- SessionStart/activity/PermissionRequest mapping tested on current supported Codex version
- Link fail-open behavior demonstrated
- Island late attach works after Hook-started activity
- degraded behavior correct after Link restart/source loss
- workspace route is accurate and honestly labelled
- no prompt/transcript/tool input/raw output crosses Pulse boundary
- resource budget passes under Hook activity storm

### Eligible for `supported_fuel`

Only after official App Server rate-limit probe passes and is independently scoped from task token claims.

### Not eligible for P0

- arbitrary existing terminal-turn control
- automatic permission decisions
- raw transcript/session log parsing
- external thread takeover based only on stored thread IDs
- WebSocket control channel

---

## 15. Design invariants

1. Codex Hooks are a lifecycle breadcrumb source, not a transcript API.
2. Codex PermissionRequest is an attention signal in P0, never a Pulse-owned approval surface.
3. `Stop` is not enough proof of task completion.
4. App Server richness belongs to Pulse-managed sessions unless live external linkage is separately proven.
5. Account quota and task token use are separate data products.
6. User-level Hook installation must be precise, reversible, trusted, and fail-open.
7. Any missing Codex capability results in a lower honest label, never a private-protocol workaround.
