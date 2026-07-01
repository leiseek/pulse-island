# Pulse Island · Onboarding, Settings, and Capability Disclosure

**Status:** Product execution baseline  
**Applies to:** First run, provider setup, privacy profiles, Safe Mode, capability wording, repair flow  
**Depends on:** `01-privacy-data-boundaries.md`, `07-adapter-ecosystem.md`, `15-provider-capability-probe.md`, `21-install-update-uninstall-contract.md`, `22-reliability-recovery-and-diagnostics.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse setup should feel like a careful instrument panel, not an enterprise wizard. A user should understand what Pulse can observe without learning Hook schemas, pipes, adapters, or source ranks.

The user journey is:

```text
Choose tools
→ choose privacy posture
→ verify with a real task
```

No account login, credential import, PATH change, terminal replacement, permanent Link service, or provider control is required.

---

## 2. Provider cards

Provider cards state capability, never blanket support.

### 2.1 Plain-language levels

| Product label | Meaning |
|---|---|
| `Passive` | Pulse can notice an app/process, but cannot reliably understand task state. |
| `Observation enabled` | A verified integration can deliver safe lifecycle signals. |
| `Can return you to workspace` | Pulse can open related project context, not necessarily original task. |
| `Can open original task` | Pulse has Exact task/thread/tab evidence. |
| `Safe Mode` | Pulse observation is paused; provider tools remain unchanged. |

### 2.2 Current provider posture

```text
Codex CLI
Probe candidate
Potential: observation + workspace return
Usage: unavailable unless independent Fuel capability is verified
[Enable observation when probe-supported]

Claude Code
Probe candidate
Potential: observation + native waiting signal + workspace return
Usage: unavailable in P0
[Enable observation when probe-supported]

Antigravity
Passive process/workspace observation only
No lifecycle integration verified
[Use passive mode]
```

No card says simply `Supported`.

### 2.3 One primary action

Each provider card exposes exactly one primary action:

```text
Enable observation
Use passive mode
Repair setup
Disable observation
Re-enable from Safe Mode
```

Technical event toggles and raw configuration editing are not normal settings surfaces.

---

## 3. Privacy posture

### 3.1 Minimal local state

```text
Recommended
Keep compact active and bounded recent-terminal status so Pulse can reconnect after Island opens later.
Never store prompts, transcripts, commands, output, diffs, or secrets.
```

### 3.2 Strict local state

```text
Keep compact state only while a task is active.
Remove task breadcrumb when it becomes terminal.
No recent terminal or post-session signal history.
```

### 3.3 Passive-only

```text
Do not install observation integrations.
Pulse may show only safe process/window/workspace facts.
No Late Attach promise.
```

Privacy profile is a retention ceiling. Strict overrides terminal grace and recovery retention.

---

## 4. Real-task verification

A successful setup requires a real signal from the user’s normally launched provider, not a fake demo command.

```text
<Provider> · Waiting for first task
Start the tool normally in any project.
Pulse will appear only when it has trustworthy state.
```

Outcomes:

| Outcome | User wording | Next action |
|---|---|---|
| Verified Hook event | `Observation is working` | Done |
| Process only | `Pulse can see the app, but not task state yet` | Use Passive or review setup |
| User settings block | `Observation is blocked by your settings` | Safe repair explanation |
| Managed policy block | `Your organization controls this integration` | Passive mode |
| Unknown provider version | `This version has not been verified` | Passive or explicit Experimental mode |
| Safe Mode | `Observation is paused by Pulse Safe Mode` | Review or re-enable |

Pulse never calls the provider “broken” when only Pulse integration needs attention.

---

## 5. Capability disclosure sheet

Every provider card can show a compact truth sheet.

```text
Claude Code observation

Can show
• Verified active work
• Waiting for confirmation when formal Hook evidence exists
• Related workspace

Cannot show yet
• Exact terminal session
• Account quota
• Current-session token total
• Approve or reject permissions

Pulse keeps
• Compact status/timestamps
• Workspace reference
• Safe event category

Pulse never keeps
• Prompts, transcripts, commands, output, diffs, secrets
```

The sheet must distinguish provider release status, task health, route capability, and Fuel feature capability.

---

## 6. Fuel disclosure

Fuel is not a provider-wide toggle.

```text
Reported quota window
Task token ledger
Burn Meter
Usage-limit block
```

Each appears only when its own source is verified. Normal initial wording is:

```text
Usage unavailable
Open official usage when available
```

Do not present unavailable Fuel as provider failure. Do not show token figures merely because account quota exists.

---

## 7. Settings shape

```text
General
Integrations
Privacy & data
Attention
Advanced
```

### General

Island placement, shortcut, density, reduced motion, high-contrast behavior, optional UI sign-in launch. UI sign-in launch never creates a permanent Link observer.

### Integrations

Provider cards, last health, capability summary, enable/disable/repair, Safe Mode state, and remove integration action.

### Privacy & data

Privacy profile, clear Pulse data, clear recent signals where profile permits, hide workspace/task labels, verified usage-source opt-in, diagnostics export preview.

### Attention

Quiet/Balanced/Focused, mute scope/duration, follow/pin, completion behavior.

### Advanced

Diagnostics, protocol versions, experimental flags, non-destructive health check, Safe Mode controls. No raw payload/config/token viewer.

---

## 8. Repair and Safe Mode

Repair is explicit and narrow:

```text
Observation setup needs repair
Pulse-owned integration entry is missing or changed.

[Review intended change]
[Reinstall observation]
[Use passive mode]
```

Safe Mode is separate:

```text
Pulse Safe Mode
Observation integrations are paused.
Your Agent tools continue to work normally.
Existing provider Hooks fail open and do not wake Pulse Link.

[Use passive mode] [Review setup] [Re-enable observation]
```

Safe Mode does not remove provider configuration automatically.

---

## 9. Acceptance tests

1. User can choose Passive without modifying provider config.
2. User sees data/retention consequences before enabling observation.
3. Provider card never makes blanket support claim.
4. Fuel unavailable displays as normal absence, not failure.
5. Safe Mode communicates that existing Hooks are inert for Pulse and provider behavior continues.
6. Exact task return is never shown for Strong/Useful/Weak route evidence.
7. Removing integration preserves unrelated provider configuration.
8. Clearing Pulse data does not remove provider integration without separate confirmation.

---

## 10. Design invariants

1. Onboarding teaches capability, not implementation trivia.
2. Passive mode is a first-class product path.
3. Every privilege earns a visible benefit.
4. A user can retreat to a lower integration level without breaking tools.
5. Product wording never outruns Probe evidence.
