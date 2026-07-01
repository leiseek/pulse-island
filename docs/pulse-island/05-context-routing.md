# Pulse Island · Context Routing

**Status:** Normative route-selection contract  
**Applies to:** Focus Card route actions, Peek affordances, Pulse Link route discovery, Windows activation  
**Depends on:** `02-agent-state-model.md`, `04-multi-agent-arbitration.md`, `23-windows-observation-and-window-binding.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse compresses an Agent state into a small signal. Context Routing gives the user the safest way back to the relevant real context without pretending a new shell, generic app, or workspace folder is the original in-flight task.

It answers:

> Given this task, what is the strongest verified return action Pulse can perform now?

Routing is graded. It never fabricates a missing session, restarts work, or simulates control of a terminal.

---

## 2. Capability and route-strength model

### 2.1 Per-task context capability

| Context capability | Meaning |
|---|---|
| `none` | No safe route exists. |
| `agent_ready` | Pulse can open a provider/agent surface, but not the original task. |
| `workspace_ready` | Pulse can open the related workspace/folder, but not the original task. |
| `context_ready` | Pulse has a route candidate to the original context. Final action wording still depends on route strength. |

### 2.2 Route strength and labels

| Strength | Evidence | Allowed labels |
|---|---|---|
| Exact | Documented provider task/thread route, or a verified exact terminal tab/session route | `Open original task`, `Open provider thread`, `Focus terminal tab` |
| Strong | Validated relevant provider/agent window, but exact task/tab is not proven | `Focus agent window`, `Focus related terminal` |
| Useful | Verified workspace, folder, agent surface, or official usage destination | `Open workspace`, `Reveal project folder`, `Open agent`, `Open official usage` |
| Weak | Process identity only | `Show process details` |
| None | No verified target | No primary action |

`Open original task` is Exact-only. A window that belongs to a provider process is not enough proof by itself.

---

## 3. Route record

```text
ContextRoute
├── route_id
├── task_key
├── route_kind
├── route_strength
├── target_ref
├── evidence[]
├── verified_at
├── expires_at
├── launch_policy
├── fallback_route_ids[]
├── user_visible_label
└── last_launch_result
```

### 3.1 Route kinds

```text
open_provider_thread
focus_terminal_tab
focus_agent_window
focus_related_terminal
open_workspace
reveal_workspace_folder
open_agent
open_provider_usage
show_process_details
```

There is intentionally no `open_new_terminal`, `restart_task`, or generic `resume_task` route kind.

---

## 4. Route anchors and evidence

### 4.1 Session / provider anchor

```text
provider
provider_session_or_thread_ref
source_instance_id
validated_at
```

This can support an Exact route only where the provider exposes a documented task/thread activation mechanism and the association is verified for the current task.

### 4.2 Window anchor

```text
hwnd
window_process_fingerprint
window_class
validated_at
```

A window handle is not durable identity. It must be revalidated immediately before activation.

### 4.3 Terminal anchor

```text
terminal_host_process_fingerprint
provider_process_fingerprint
host_specific_tab_or_session_ref (optional)
validated_at
```

Without a stable host-specific tab/session association, a terminal may be Strong at most. It cannot be labelled Exact.

### 4.4 Workspace anchor

```text
workspace_stable_hash
short_display_name (optional)
safe_open_path_ref
validated_at
```

Workspace is a valuable Useful route. It is never an alias for the original session.

### 4.5 Process anchor

```text
pid
process_started_at
executable_identity
user_session_scope
```

Process anchor permits Weak process details only unless combined with stronger provider/route evidence.

---

## 5. Candidate construction and selection

Adapters may emit route candidates; Windows services may validate OS facts. Neither layer may promote a candidate by UI success alone.

### 5.1 Selection order

```text
1. Exact provider task/thread route
2. Exact verified terminal tab/session route
3. Strong verified agent/window route
4. Useful workspace/folder route
5. Useful agent/provider/usage route
6. Weak process details
```

### 5.2 Replacement rule

A new route replaces the best route only when it is:

- stronger, or
- equally strong and more recently verified, or
- explicitly user-confirmed while still satisfying the same evidence category

A successful workspace open does not upgrade the task to Exact or `context_ready`.

### 5.3 No speculative correlation

Window title, console text, screen content, accessibility text, shell history, command lines, environment variables, OCR, or synthetic input are not valid route evidence.

---

## 6. Launch policies

### 6.1 Exact provider route

```text
revalidate provider/session reference
→ invoke documented provider activation/deep-link mechanism
→ report exact route result
→ fall back only with a visibly weaker label
```

### 6.2 Exact terminal-tab route

```text
revalidate terminal host, provider process, and host-specific tab/session binding
→ user-initiated focus
→ report exact route result
```

### 6.3 Strong agent/window route

```text
revalidate process fingerprint and HWND
→ restore/minimize handling only through ordinary Windows behavior
→ user-initiated focus
→ report strong route result
```

### 6.4 Useful workspace / agent / usage route

```text
validate workspace/provider target
→ open existing IDE window when verified, otherwise configured workspace/folder route
→ report useful route result
```

### 6.5 Weak process details

Show a safe local details surface with provider category, process age, health, and route unavailability explanation. Do not display command-line arguments, environment variables, terminal content, or raw paths by default.

---

## 7. Failure and fallback

```text
user invokes best route
→ revalidate
→ attempt action
→ if unavailable, invalidate stale evidence
→ offer next lower verified route
→ update context capability / label
```

Examples:

| Failed route | Honest fallback |
|---|---|
| Exact provider thread unavailable | Open workspace or agent surface |
| Exact terminal tab unavailable | Focus related terminal only if Strong evidence remains; otherwise workspace |
| Agent window closed | Open workspace |
| Workspace disconnected/unavailable | Show process details or no route |

Pulse never silently starts a new CLI, invokes provider resume, opens a new terminal, or simulates keystrokes as a fallback.

---

## 8. User intent and attention leases

Routing and attention are related but not identical.

| Successful route | Attention behavior |
|---|---|
| Exact original context | Start 5-minute attention lease for unchanged issue. |
| Strong/Useful fallback | Start 60-second route-attempt quiet window only. Task remains unresolved. |
| Weak process details | No attention lease. |
| Failed action | No lease; show truthful fallback. |

A new failure, a new waiting event, or a higher arbitration tier can preempt immediately.

---

## 9. Windows activation rules

- Route activation requires explicit user action.
- Passive state changes never foreground another app.
- Do not loop focus APIs or synthesize input to bypass Windows focus policy.
- Do not drag target windows to the Island monitor.
- Cross-user, cross-logon-session, elevated, stale, or closed targets are unavailable rather than force-focused.
- Fullscreen/presentation policy suppresses automatic activation; user actions still follow ordinary OS rules.

---

## 10. Adapter responsibilities

Each adapter must declare only route capabilities it can prove.

```text
RouteCapabilities
├── open_exact_context
├── focus_terminal_tab
├── focus_agent_window
├── open_workspace
├── reveal_workspace_folder
├── open_agent
├── open_provider_usage
└── show_process_details
```

Capabilities are task-specific and may degrade/revoke when source freshness or target validation fails.

---

## 11. Acceptance scenarios

1. Exact provider route opens only a verified provider task/thread.
2. A process-owned window with no exact tab/session proof renders `Focus agent window`, not `Open original task`.
3. A stale HWND or PID reuse invalidates the route and falls back honestly.
4. Workspace route always says `Open workspace` or `Reveal project folder`.
5. Process-only task shows details only and cannot claim `context_ready`.
6. No route launches a new CLI/session or simulates terminal input.
7. Exact-route success creates a 5-minute lease; Useful/Strong success creates only 60-second quiet window.
8. Route activity remains local and stores no command line, transcript, tool payload, or arbitrary screen content.

---

## 12. Design invariants

1. A route returns the user to existing context; it never manufactures context.
2. User-visible labels match evidence strength exactly.
3. Strong is useful but not Exact.
4. Workspace is valuable but never a substitute name for a task/session.
5. Routing changes presentation/navigation only. It does not alter provider task state.
