# Pulse Island · Claude Code Capability Probe Card

**Status:** Evidence-backed probe plan, not a release-support claim  
**Provider:** Anthropic Claude Code  
**Target integration posture:** User-level command Hooks for Observe-first lifecycle breadcrumbs; workspace-first routing; no external interactive-session control  
**Last updated:** 2026-07-01

---

## 1. Decision summary

Claude Code is a strong first-provider candidate for Observe-first integration because its official Hook system exposes session lifecycle, prompt-turn, tool-loop, permission, subagent, task, stop, and session-end events. The safe Pulse path is still deliberately narrow:

> **P0 Claude Code = user-level command Hooks + session/workspace breadcrumb + verified waiting signal + workspace-first return route.**

Pulse does not take control of an arbitrary Claude Code terminal session. It does not parse transcripts, read assistant text, capture command arguments, handle permission decisions, resume sessions, or use Hook outputs to modify Claude behavior.

The Hook system can provide `session_id`, `cwd`, hook-event type, permission mode, and selected lifecycle metadata. It can also expose rich sensitive content such as prompts, transcript paths, tool inputs, tool results, assistant messages, background task command lines, plan files, and compaction summaries. Pulse must allow-list only a tiny metadata subset before Link transport.

Claude Code can return Hook decisions for events including `PermissionRequest`, `PreToolUse`, `Stop`, and some task events. P0 Pulse always returns no decision. Exit code 0 with no decision allows Claude Code’s normal permission/control behavior to continue.

---

## 2. Evidence register

This probe card is based on official Claude Code documentation reviewed on 2026-07-01:

| Source | Relevant facts used by this card |
|---|---|
| Claude Code Hooks reference | Hook lifecycle/events, user and project Hook scopes, command Hook stdin input, event-specific decision controls, timeout behavior, and sensitive event fields. |
| Claude Code Settings | User-level settings path, Windows path resolution, settings hierarchy, managed settings constraints. |
| Claude Code Costs | `/usage` session and plan information, local-history scope limitation, and billing caveat. |
| Claude Code Monitoring | OpenTelemetry telemetry behavior and its potential to export metrics/logs/traces externally. |

The actual probe report must record the exact Claude Code version. Hook event schemas and behavior are versioned product surfaces; current documentation and live probe results override remembered assumptions.

---

## 3. Integration modes

### 3.1 Mode A: Official user-level command Hooks, P0 candidate

Pulse adds only its own command Hook entries to user settings:

```text
%USERPROFILE%\.claude\settings.json
→ Claude Code command Hook
→ pulse-link-shim.exe
→ pulse-link.exe
→ compact breadcrumb + reducer
```

Properties:

- works with normal `claude` CLI launch
- runs on all user projects when user settings allow it
- supplies a provider session ID and current working directory
- supplies an official permission-dialog event
- lets Pulse wake Link while Island is closed
- uses a short, synchronous, fail-open shim invocation

Constraints:

- organization-managed settings may disallow user Hooks
- existing user/project hooks can coexist and run in parallel
- Hook payload may contain sensitive content, requiring strict allow-listing
- Hook events prove an event occurred, not necessarily whole-session completion or control ownership

### 3.2 Mode B: Passive process/workspace observation, P0 fallback

No Hook is installed.

```text
process observation
+ cautious process-start-time / parent relation correlation
+ optional workspace association
→ Observed only
```

This may expose a visible Claude process and a useful workspace route. It cannot claim Hook session identity, waiting-user state, exact session context, completed/failure semantics, plan status, or account quota.

### 3.3 Mode C: User-authorized local usage adjunct, future probe only

Claude Code’s `/usage` command presents local session token statistics and subscription-plan breakdowns computed from local session history. Pulse must not screen-scrape `/usage`, drive terminal UI, or ingest full session JSONL to reproduce it.

A future usage adjunct can only be considered when a formal safe, machine-readable source is verified or the user explicitly supplies a bounded export designed for Pulse. Until then:

```text
Claude Fuel account quota = Unavailable
Claude session tokens = Unavailable except for narrow verified subagent telemetry
```

### 3.4 Mode D: Enterprise-managed telemetry, explicitly out of scope for local Pulse P0

Claude Code supports OpenTelemetry for organizational monitoring. Pulse must not enable or redirect OTel exporters, modify exporter endpoints/headers, or consume logs/traces because they can be configured to export detailed events and network traffic.

An organization may independently run its own telemetry program. That does not create a local Pulse data source unless a separately designed, admin-approved integration exists.

---

## 4. User-level Hook installation plan

### 4.1 Target location

Use only the documented user-level settings surface:

```text
~/.claude/settings.json
```

On Windows, this resolves to:

```text
%USERPROFILE%\.claude\settings.json
```

Do not modify by default:

- `.claude/settings.json` in the project
- `.claude/settings.local.json` in the project
- `CLAUDE.md`, `.claude/rules/`, agent files, skill files, or plugin files
- shell profile or PATH precedence
- managed-policy registry/file locations
- `~/.claude.json`, which can contain OAuth and other sensitive configuration

### 4.2 Managed settings policy

If managed settings prohibit user Hooks or use managed-only Hook policy:

```text
Adapter health = needs_repair or blocked
Integration mode = Passive / process-observed only
```

Pulse must not create a project Hook, plugin, or alternate shell wrapper as a workaround.

### 4.3 Pulse-owned Hook identity

Claude Code Hook objects do not require arbitrary metadata keys. Pulse identifies its own handler with an exact executable-plus-argument signature:

```text
command: <absolute path to pulse-link-shim.exe>
args:
  - --provider
  - claude-code
  - --integration-id
  - <non-secret Pulse installation identifier>
```

The integration identifier is an installation-local non-secret marker. It is not a provider credential, session ID, account ID, or workspace identifier.

### 4.4 Command configuration rules

- Use command Hook exec form with `args`, avoiding shell parsing and quoting surprises.
- Do not use a PowerShell shell form merely to launch Pulse.
- Use a short explicit per-Hook timeout, initially 1 second, while Shim itself has a 400 ms fail-open hard timeout.
- Omit `async`; Pulse needs a bounded attempt to deliver the lifecycle event before the Hook returns.
- Configure only documented command Hook handlers.
- Do not return `additionalContext`, decisions, updated input, updated tool output, or session title changes.
- Do not require bypass-permissions mode or any Hook-trust workaround.

### 4.5 Installation transaction

```text
read user settings JSON
→ parse and validate
→ locate prior Pulse executable-plus-argument signature
→ preserve unrelated hook groups and handlers
→ add or replace only Pulse-owned handlers
→ atomic write
→ re-read / validate
→ run a non-destructive Hook health check
```

Uninstall removes only matching Pulse handlers. It must not delete other handlers that happen to listen to the same Hook event.

A malformed existing settings file, inaccessible managed policy, or Hook restriction results in an honest adapter-health failure. Pulse does not rewrite the whole settings file from a template.

---

## 5. P0 Hook set

Pulse should begin with the smallest Hook set that proves useful attention state without turning every tool call into a content source.

```text
SessionStart
UserPromptSubmit
PreToolUse
PermissionRequest
Stop
StopFailure
SessionEnd
CwdChanged
```

Optional P1 probe events:

```text
PostToolUseFailure
SubagentStart
SubagentStop
TaskCreated
TaskCompleted
Notification
```

The initial P0 list deliberately excludes `PostToolUse` because the official input carries full `tool_input` and full `tool_response`; Pulse does not need those to refresh activity when `PreToolUse` already provides a safe heartbeat.

---

## 6. Strict Hook allow-list

Claude Code command Hooks receive JSON on stdin. The Shim must parse only the approved fields and immediately discard the original payload buffer after constructing a bounded Pulse envelope.

### 6.1 Allowed common fields

| Raw Hook field | Pulse handling |
|---|---|
| `session_id` | Accept as opaque session identity. Hash/opaque-key it before broad persistence. |
| `cwd` | Normalize to workspace stable identity; retain a user-local route reference only under privacy policy. |
| `hook_event_name` | Accept and map through static adapter code. |
| `permission_mode` | Accept only as coarse context; never use it to authorize action. |

### 6.2 Allowed event-specific metadata

| Event | Allowed metadata | Pulse purpose |
|---|---|---|
| `SessionStart` | `source` | startup/resume/clear/compact lifecycle reason. |
| `PreToolUse` | `tool_name` only | activity heartbeat and generic tool category. |
| `PermissionRequest` | `tool_name` only | generic waiting reason, without tool arguments. |
| `Stop` | `stop_hook_active`; bounded boolean/count state only after probe | terminal candidate / background-work caution. |
| `StopFailure` | fixed error category only, after static mapping | possible terminal failure candidate. |
| `SessionEnd` | `reason` | session termination reason category. |
| `CwdChanged` | `new_cwd` | update workspace anchor after normalization. |
| `SubagentStart` / `SubagentStop` | `agent_id`, `agent_type` only | bounded child-count observation. |
| `TaskCreated` / `TaskCompleted` | `task_id` only in initial probe | opaque child-task correlation, not user-visible title. |

### 6.3 Explicitly forbidden or dropped fields

| Raw field / source | Reason |
|---|---|
| `transcript_path`, `agent_transcript_path` | Pulse does not parse or retain transcripts. |
| `prompt`, slash-command prompt, custom instructions | User content. |
| `tool_input` in any form | May contain commands, source paths, plans, credentials, or user content. |
| `tool_response`, tool output, stdout/stderr | May contain code, secret material, or large content. |
| `last_assistant_message`, `MessageDisplay.delta` | Provider-generated text content. |
| `background_tasks[].command`, `.description`, `session_crons[].prompt` | Commands and prompt content. |
| `task_subject`, `task_description` | May encode user/project content; keep out of P0 display. |
| `Notification.message`, titles | Arbitrary content. |
| `compact_summary`, plan text, plan file path | Conversation/task content. |
| file paths from FileChanged/Worktree events | Not needed for P0 and can expose sensitive layout. |
| unknown fields | Do not forward, persist, or log. |

### 6.4 Shim output envelope

```text
PulseHookEnvelope
├── protocol_version
├── provider = claude_code
├── integration_id
├── hook_event
├── occurred_at
├── session_ref
├── workspace_ref (optional)
├── lifecycle_hint (optional)
├── attention_hint (optional)
├── safe_summary (optional, static-category-derived)
├── safe_error (optional, static-category-derived)
├── capability_hints[]
└── source_metadata (bounded)
```

No raw Claude Hook object is sent over Link IPC, written to disk, added to diagnostics, or exposed to Island.

---

## 7. Hook-to-Pulse event map

This is a deliberately conservative observation map.

| Claude Code Hook | Pulse event(s) | Initial lifecycle effect | Must not infer |
|---|---|---|---|
| `SessionStart` | `session_started` | `starting`; becomes `running` after activity | task completed, exact terminal route, task title from prompt/transcript |
| `UserPromptSubmit` | `activity_observed` | current turn becomes/re-enters `running` | retain prompt text or infer task title from it |
| `PreToolUse` | `activity_observed` | refreshes running freshness | actual command/input, tool success/failure, whole-task state |
| `PermissionRequest` | `waiting_observed` | `waiting_user`, `needs_user` | allow/deny, tool argument display, replacing native prompt |
| `Stop` | `turn_stop_observed` | terminal candidate; may become `turn ended` after probe-defined settle | session completion, no remaining background work, globally successful task |
| `StopFailure` | `turn_error_observed` | possible failure candidate using static error class | raw error text, root cause, broader session failure |
| `SessionEnd` | `session_end_observed` | `terminated` unless stronger terminal evidence exists | successful completion |
| `CwdChanged` | `workspace_changed` | workspace anchor update | opening a new project/session |
| `PostToolUseFailure` | `tool_failure_observed` | P1 watch/error detail candidate | whole-task failure from one failed tool call |
| `SubagentStart` | parent activity + optional child count | parent stays running | independent top-level user task |
| `SubagentStop` | child count update | no parent terminal conclusion | parent completed / content summary |
| `TaskCreated` / `TaskCompleted` | P1 child task lifecycle | bounded team-task state only | user-visible top-level task success |

### 7.1 Why `Stop` is a candidate, not automatic completed

Official documentation says `Stop` runs when the main agent has finished responding, but the event supports continuation/block control and can be influenced by other Hooks. The input can also report background tasks or scheduled session wakeups. Pulse itself returns no decision, but it cannot assume the entire user task or session is conclusively finished just because one `Stop` arrived.

Initial P0 behavior:

```text
Stop
→ record turn-stop candidate
→ inspect only allowed no-content flags/counts after probe
→ wait short settle window for fresh activity / native continuation
→ show recent “Turn ended” or completed only if probe validates semantics
```

Until that probe passes, `Stop` must not outrank a fresh waiting state or be rendered as a confident completed task.

### 7.2 Why `PostToolUseFailure` is not whole-task failure

A tool can fail while Claude recovers, retries, chooses another tool, or asks the user for help. P0 either ignores it as a lifecycle terminal signal or maps it to a low-severity `watch` with a generic safe category after the probe proves this is useful. It never uses tool error text as Focus Card content.

---

## 8. Native permission and decision policy

`PermissionRequest` is the key yellow-state source, but its decision surface remains Claude Code’s.

### 8.1 P0 behavior

```text
Claude PermissionRequest
→ Pulse Shim sends `waiting_observed`
→ Link records waiting breadcrumb
→ Shim prints nothing and exits 0
→ Claude Code displays/continues its native permission flow
```

Pulse can show:

```text
Claude Code is waiting for confirmation
Open workspace
```

P0 cannot show:

```text
[Approve]
[Reject]
[Always allow]
[Run command]
```

### 8.2 Safety rules

- Never output `allow`, `deny`, `ask`, `defer`, `updatedInput`, or a permission decision object.
- Never return additional context to Claude.
- Never retain permission suggestions or tool input.
- If Link/Island is missing, native Claude behavior is unchanged.
- If another Hook controls a decision, Pulse has no vote and no override.

### 8.3 Future control bridge

A future in-Island approval bridge is out of scope until all of these are proven:

- a formal Claude Code decision surface can be bridged without losing context
- user opt-in is explicit
- a timeout reliably returns to native Claude permission behavior
- no default approval exists
- safe summary is sufficient, or original context is immediately available

---

## 9. Lifecycle confidence model

### 9.1 P0 capability targets

| Capability | Initial target | Evidence |
|---|---|---|
| Discover session | Attached candidate | `session_id` in Hook input |
| Discover workspace | Attached candidate | Hook `cwd`, normalized locally |
| Running freshness | Attached candidate | UserPromptSubmit + PreToolUse heartbeat |
| Waiting user | Attached candidate | PermissionRequest Hook, no decision output |
| Session termination | Attached candidate | SessionEnd reason, mapped conservatively |
| Turn terminal candidate | Experimental | Stop plus settle/background-work semantics probe |
| Clear terminal API error | Experimental | StopFailure static error mapping probe |
| Tool failure watch | Experimental | PostToolUseFailure without raw payload retention |
| Exact terminal context | Experimental | verified Windows process/window binding only |
| Completion | Not assumed initially | requires probe-backed Stop interpretation or stronger session evidence |
| General failure | Not assumed initially | requires probe-backed StopFailure/terminal evidence |

### 9.2 Health rules

- `Attached` requires validated Hook session identity plus fresh Hook delivery.
- `Observed` is process-only/cold discovery.
- `Degraded` applies to stale Hook state, delivery/config failure, ambiguous identity, or restored breadcrumb prior to fresh Hook evidence.
- `Offline` applies when a previously correlated provider process/session is no longer reachable and no terminal evidence exists.

---

## 10. Context routing plan

### 10.1 P0: workspace-first route

The strong P0 anchor is session identity plus normalized `cwd`:

```text
Claude Hook session_id + cwd
→ workspace anchor
→ Open workspace / reveal project folder
```

The label is `Open workspace`, never `Open original task`.

### 10.2 P0.5: related terminal/window focus, experimental

Probe only when all links are independently verified:

```text
Hook shim process
→ parent/process-start-time relationship
→ live Claude Code process
→ exactly one terminal/app window
→ target revalidated at click time
```

Window title text by itself is not sufficient. If correlation fails, the route remains workspace-ready.

### 10.3 Explicit prohibitions

- Do not execute `claude --resume` or a new `claude` command as a substitute for returning to the original task.
- Do not parse transcript JSONL to discover a task or recover session context.
- Do not simulate terminal keyboard/mouse input.
- Do not open a generic Claude surface and label it as the original session.

---

## 11. Fuel posture

### 11.1 Subscription and local `/usage` data

Official Claude Code documentation says `/usage` presents detailed session token statistics and subscription-plan information, but its local 24-hour/7-day attribution is approximate and based on local session history on that machine. It excludes usage from other devices or Claude web surfaces.

Pulse must not represent this as authoritative account quota unless Anthropic exposes a supported scoped source suitable for direct integration.

### 11.2 P0 Fuel decision

```text
Account quota snapshot: Unavailable
Current-session token total: Unavailable
Official usage route: Available
```

Focus Card example:

```text
Claude Fuel
Account quota unavailable
[Open official usage]
```

### 11.3 Narrow subagent-token probe, P1 candidate

Official `PostToolUse` metadata for completed synchronous Agent calls can include numerical subagent token/cost usage. Pulse may probe a strictly numeric extraction path:

```text
PostToolUse where tool_name = Agent
→ extract only usage numeric fields / totalTokens
→ discard full tool_response content
→ label as “subagent tokens observed”, not session total or account quota
```

This path is P1 because the Hook input still contains potentially sensitive tool data and must prove field-level extraction occurs before any broad payload retention.

### 11.4 OpenTelemetry exclusion

Claude Code monitoring can export metrics, logs, and optional traces through OpenTelemetry. Pulse does not enable, redirect, scrape, or depend on that system. It would exceed the local-only, content-minimized P0 boundary and could introduce network/log retention concerns.

---

## 12. Claude Code probe scenarios

### CCP-1: User Hook install and rollback

```text
prepare %USERPROFILE%\.claude\settings.json with unrelated Hook entries
→ install Pulse handlers
→ run Claude Code normally in synthetic workspace
→ update Pulse handlers
→ uninstall Pulse handlers
→ verify unrelated handler objects remain unchanged
```

### CCP-2: Hook breadcrumb with Island closed

```text
Pulse Hooks installed
→ start normal Claude Code session
→ confirm SessionStart reaches Link
→ submit a synthetic task
→ observe PreToolUse activity heartbeat
→ start Island later
→ verify session/workspace/running state
```

### CCP-3: Native permission remains native

```text
trigger a permission dialog
→ PermissionRequest reaches Link
→ test with Island absent, then present
→ verify Pulse yellow waiting state
→ verify native Claude permission UI / behavior continues unchanged
```

### CCP-4: Turn and session semantics

```text
normal Stop
Stop with background task or scheduled wakeup where available
StopFailure
one PostToolUseFailure followed by recovery
SessionEnd for normal exit / clear / resume switch
```

Required result:

- no confident task completion from Stop before semantic probe settles
- no task failure from one failed tool call
- SessionEnd maps to termination, not success
- StopFailure only becomes failure after static error-category mapping is validated

### CCP-5: Late attach and restart

```text
start normal Claude Code session while Island is closed
→ create Hook activity
→ attach Island later
→ restart Island
→ verify compact session breadcrumb
→ restart Link
→ verify restored state begins degraded
→ receive fresh Hook event
→ verify appropriate health recovery
```

### CCP-6: Context route truthfulness

```text
attempt verified original terminal/window focus
→ if not proven, Open workspace only
→ close original terminal and verify fallback behavior
```

The report must record evidence strength, action label, target observed, and fallback label for every route.

### CCP-7: Fuel boundaries

```text
inspect /usage only through official user-visible surface
→ confirm no screen scraping or transcript parsing is introduced
→ probe Agent PostToolUse numeric usage extraction with synthetic/session-safe tests
→ verify labels preserve “subagent observed” vs “session” vs “account quota”
```

### CCP-8: Managed policy and failure injection

```text
managed setting blocks user Hooks
malformed user settings file
Link absent
Shim timeout
malformed Hook stdin
Link crash
Island crash
Pulse handler removed by user
```

Required result:

- Claude Code continues normal behavior
- Pulse degrades only its own adapter health/capability state
- no project settings or plugin workaround is applied

---

## 13. Initial capability decision matrix

This is a target matrix to validate, not a release claim.

| Capability | Initial release ceiling | Gate to raise it |
|---|---|---|
| Process discovery | Passive Observed | Windows process binding probe |
| Session identity | Hook Attached candidate | CCP-2 identity/collision tests |
| Workspace identity | Workspace-ready candidate | CCP-2 and CwdChanged tests |
| Running freshness | Hook Attached candidate | CCP-2 quiet/staleness tests |
| Waiting user | Hook Attached candidate | CCP-3 native permission pass-through |
| Turn ended | Experimental | CCP-4 Stop/background-task semantics |
| Completed | Unavailable initially | probe-backed turn/session terminal rule |
| Failure | Unavailable initially | CCP-4 StopFailure evidence mapping |
| Workspace route | Workspace-ready candidate | CCP-6 route verification |
| Exact original context | Experimental | verified terminal/window route |
| Account quota | Unavailable | official supported machine-readable source only |
| Current-session tokens | Unavailable | formal safe source only |
| Subagent numeric tokens | Experimental P1 | CCP-7 numeric-only extraction audit |
| Approve / deny | Unavailable P0 | separate control-bridge review |
| Stop / steer / resume | Unavailable | formal supported control surface only |

---

## 14. Release decision rules

### Eligible for `supported_observe`

Claude Hook mode reaches `supported_observe` only after all of these pass:

- user-level install/update/uninstall preserves unrelated Hooks
- managed-policy restrictions are respected
- SessionStart, activity, PermissionRequest, and SessionEnd mappings are verified on supported Claude Code versions
- Pulse Hook returns no decision and native permission behavior remains unchanged
- late Island attach works after Hook-created breadcrumb
- Link restart correctly restores degraded state until fresh evidence arrives
- workspace route is accurate and honestly labelled
- no transcript/prompt/tool-input/tool-output/assistant-message content reaches Link, persistence, diagnostics, or UI
- resource budget holds under Hook activity traffic

### Eligible for `supported_fuel`

Only after an official scoped source or a separately reviewed numeric-only telemetry source passes privacy, freshness, and scope tests. `/usage` screen text and session JSONL parsing are not a supported Fuel integration path.

### Not eligible for P0

- arbitrary existing terminal-session control
- permission approval/rejection in Island
- transcript or `/usage` screen parsing
- generic `claude --resume` as context recovery
- enabling or consuming OpenTelemetry exports
- task titles derived from raw prompt/tool/assistant content

---

## 15. Design invariants

1. Claude Hooks provide lifecycle evidence, not a license to ingest session content.
2. A native Claude permission prompt remains native unless a future formal bridge proves otherwise.
3. `Stop` means a main-agent response boundary, not automatically a completed user task or session.
4. Session end is termination unless stronger evidence proves success/failure.
5. `/usage` is not a general-purpose machine API for Pulse and must not be screen-scraped.
6. Workspace routing is valuable even when exact session recovery is unavailable.
7. Missing Claude capability lowers the label. It never justifies a transcript parser, UI automation, or network telemetry detour.
