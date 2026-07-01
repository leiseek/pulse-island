# Pulse Island · Installation, Update, Repair, and Uninstall Contract

**Status:** Product and runtime contract  
**Applies to:** Windows packaging, first install, upgrade, repair, rollback, integration lifecycle, removal  
**Depends on:** `01-privacy-data-boundaries.md`, `06-pulse-link-runtime-architecture.md`, `08-integration-hook-protocol.md`, `20-onboarding-settings-and-capability-disclosure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Island touches sensitive developer workflows: terminal tools, user-level Hook configuration, local process observation, and desktop attention. Installation and updates must therefore feel less like a hungry framework installer and more like a careful utility.

This contract answers four questions:

1. What does Pulse place on the machine?
2. What does installation deliberately avoid changing?
3. How can an update occur without disturbing active Agent work?
4. How can a user fully remove Pulse without damaging their tools or other integrations?

---

## 2. Installation principles

### 2.1 Per-user by default

Pulse installs per user by default. It should not require administrator rights for normal use.

Suggested ownership boundaries:

```text
Program files
%LOCALAPPDATA%\Programs\Pulse Island\

Pulse local state
%LOCALAPPDATA%\PulseIsland\

User-visible diagnostics export
user-chosen location only
```

A machine-wide installer, enterprise deployment channel, or managed policy model can exist later, but it is not P0 and must not weaken the per-user privacy model.

### 2.2 No surprise integration

Installing the application does not automatically:

- install Codex or Claude Hooks
- modify Antigravity configuration
- alter PATH ordering
- add shell aliases
- replace `codex`, `claude`, or other executables
- create a Windows service
- configure an auto-starting Link process
- import provider credentials
- scan existing session history

Provider observation is enabled only through an explicit Onboarding or Integrations action.

### 2.3 Signed, identifiable artifacts

Release artifacts must have a stable publisher identity and a verifiable version manifest. The application must expose:

- installed version
- build channel
- installation scope
- update state
- release notes or a user-visible change summary before an upgrade is applied

Pulse must never use a provider account token, browser cookie, or Agent credential as an update credential.

---

## 3. Installed components

A normal user installation may contain:

```text
pulse-island.exe
pulse-link.exe
pulse-link-shim.exe
pulse-uninstall.exe or package-owned uninstall entry
version manifest
shared native runtime files only when needed
```

### 3.1 Component roles

| Component | Runs when | Must not do |
|---|---|---|
| `pulse-island.exe` | User opens Island or user enables an optional UI sign-in launch | Start a permanent agent observer on its own. |
| `pulse-link.exe` | A supported Hook/explicit wake requests it, or Island requests a state connection | Remain alive indefinitely without active work/grace. |
| `pulse-link-shim.exe` | A provider invokes a Pulse-owned Hook command | Parse transcripts, write raw Hook input, or block the provider. |
| uninstaller | User explicitly uninstalls | Delete unrelated provider settings or credentials. |

No component runs as a privileged Windows service in the P0 architecture.

---

## 4. First install sequence

```text
Install app files
→ verify package/version integrity
→ create Pulse-owned local state directory with current-user access
→ register uninstaller
→ launch Onboarding or leave app closed
→ do not launch Link
→ do not touch provider configuration
```

### 4.1 Initial local state

Initial state contains only:

- installation identifier
- application version/channel
- user preference defaults
- empty integration registry
- no task/session breadcrumbs
- no provider config backups until a user enables an integration

The installation identifier is local, non-secret, and never reused as a provider/session/account ID.

---

## 5. Provider integration transaction

Enabling an integration is a separate transaction from application install.

```text
User clicks Enable observation
→ show capability and data disclosure
→ validate provider/config eligibility
→ read only relevant user-level provider configuration
→ create a targeted Pulse-owned backup fragment
→ add/update one identifiable Pulse integration entry
→ atomically write
→ re-read and validate
→ run non-destructive health check
→ mark integration enabled only after verified result
```

### 5.1 Atomicity requirements

- The original provider configuration stays valid until the replacement is fully written.
- A partial write cannot leave a provider configuration truncated.
- A failed validation restores the previous valid configuration when possible.
- The installer records only a narrow Pulse-owned fragment, not a copy of unrelated user configuration.
- If restoration fails, Pulse marks itself `needs_repair` and presents explicit user choices. It does not keep retrying silently.

### 5.2 Configuration backup policy

Provider integration backups:

- contain only the Pulse-owned entry plus minimal placement metadata
- are stored under the current user’s Pulse data root
- are protected by current-user file permissions
- are deleted after successful uninstall or retention expiry
- never include provider credentials, unrelated Hook entries, prompts, transcripts, or workspace content

---

## 6. Update policy

### 6.1 P0 update posture

P0 uses user-initiated update checks and user-confirmed installation. It does not perform an always-on background update check.

A future automatic update channel may be considered only when its network metadata, privacy posture, package verification, rollback behavior, and enterprise controls are separately designed.

### 6.2 Update check data boundary

A manual update check may send only the minimum version/channel/platform information required to determine whether an update exists. It must not include:

- task state
- workspace name/path
- provider/session IDs
- Hook payloads
- account identity
- usage/quota data
- diagnostics content

The UI states this plainly before enabling any future automatic update check.

### 6.3 Staged update model

```text
User approves update
→ download/validate staged package
→ wait for safe component boundary
→ replace app files atomically
→ preserve prior package for rollback until health check passes
→ start updated Island only on user action or existing UI restart policy
```

An update must not overwrite a running executable in place if this risks corrupting a live Link/Island process.

### 6.4 Active Agent work during update

When `pulse-link.exe` has active tasks or an active grace period:

```text
Update ready
Agent work is active.
Install when Pulse is idle, or install now without interrupting the Agent.

[Install when idle] [Install now] [Later]
```

Rules:

- `Install when idle` is the recommended default.
- `Install now` may restart Pulse components only after their current bounded checkpoint succeeds.
- Pulse never kills, restarts, or interrupts a provider process.
- If Link cannot checkpoint safely, update defers rather than forcing termination.
- Provider Hook calls during update must fail open.

### 6.5 Version compatibility

During a rolling update, Link and Shim must tolerate at least the current and previous compatible Hook protocol major/minor policy defined in `08-integration-hook-protocol.md`.

If compatibility is uncertain:

- Shim fails open for ordinary provider Hook events.
- Link marks integration degraded.
- Island displays a repair/update explanation, not a false task failure.

---

## 7. Schema and state migration

### 7.1 Migration rule

State migration must be additive, bounded, and reversible where feasible.

Before migration:

```text
validate current state schema
→ create bounded local recovery copy
→ migrate
→ validate migrated schema
→ retain recovery copy until next successful launch window
```

### 7.2 What migrations may touch

- Pulse preferences
- bounded breadcrumbs
- integration registry
- Pulse-owned backup fragments
- local diagnostics metadata

Migrations must not touch provider configuration except through a separate explicit integration-upgrade transaction.

### 7.3 Migration failure

On migration failure:

- preserve the last valid state snapshot
- disable only the affected Pulse feature/integration
- offer `Reset Pulse local data` and `Use passive mode`
- do not delete provider integration entries as an automatic reaction
- do not block provider execution

---

## 8. Repair

Repair is a specific action, not an opaque reinstall button.

### 8.1 App repair

App repair may:

- revalidate installed Pulse binaries
- restore a missing Pulse component from a validated package
- repair Pulse-owned local permissions
- re-register Pulse uninstaller metadata
- validate Link/Island protocol compatibility

App repair may not:

- overwrite provider config
- enable integrations
- launch Agent tools
- import credentials
- delete user data without explicit confirmation

### 8.2 Integration repair

Integration repair may:

- compare current configuration with the expected Pulse-owned entry
- show the narrow intended change
- reinstall only Pulse-owned Hook/integration entry after user confirmation
- rerun non-destructive health check

It may not:

- overwrite unrelated settings
- reorder unrelated Hooks
- modify project-level configuration as a fallback
- use a shell wrapper when Hook setup fails

---

## 9. Uninstall contract

Uninstall presents two separate choices:

```text
Remove Pulse Island application
[ ] Remove Pulse observation integrations
[ ] Remove Pulse local data
```

### 9.1 Recommended default

The default selected action is:

```text
Remove application + remove Pulse-owned provider integrations
Keep local Pulse data unless user chooses to delete it
```

Reason: leaving provider Hooks pointing to a missing Shim is fail-open by design, but creates confusing configuration debris. Removing only exact Pulse-owned entries is cleaner and safer.

### 9.2 Preserve user tools

Uninstall must not:

- uninstall Codex, Claude Code, Antigravity, IDEs, terminals, runtimes, or package managers
- remove provider credentials or account state
- remove non-Pulse Hook entries
- restore stale complete config backups over current provider configurations
- delete project files
- delete unrelated `%LOCALAPPDATA%` folders

### 9.3 Failed integration cleanup

If a provider config cannot be safely edited during uninstall:

```text
Pulse could not remove its observation entry automatically.
Your provider will continue to work normally, but this entry may remain.

[Copy safe cleanup steps] [Keep local data] [Finish uninstall]
```

The cleanup instructions must identify only Pulse’s exact command signature/entry, never show unrelated provider settings.

### 9.4 Data deletion

`Remove Pulse local data` removes:

- breadcrumbs
- Pulse preferences
- local diagnostic counters/reports
- integration backup fragments
- local installation ID

It does not remove provider configurations unless the user also selected `Remove Pulse observation integrations`.

---

## 10. Crash-safe lifecycle during install/update/remove

At any installation boundary:

- provider Hook calls fail open
- Shim observes its hard timeout
- Link may lose an observation event but must not block an Agent
- Island state can be stale/degraded after restart but cannot be fabricated as attached
- no task is restarted as part of recovery

The recovery priority is:

```text
protect Agent behavior
→ preserve valid provider config
→ preserve bounded Pulse state when possible
→ explain Pulse degradation
```

---

## 11. Packaging acceptance tests

### Install

1. Clean per-user install does not require admin rights.
2. No provider configuration changes before explicit integration enablement.
3. No Link process remains after install with no active task.
4. Installed files and local state respect current-user ownership.

### Integration enablement

5. Existing unrelated provider Hook entries remain byte/structure equivalent where formatting allows.
6. Interrupted integration write leaves valid provider configuration.
7. Failed health check leaves provider task behavior unchanged.

### Update

8. Update during active task does not terminate/restart provider process.
9. Link/Island version mismatch fails open and reports degraded state.
10. State migration failure does not delete provider configuration.

### Repair and uninstall

11. Repair changes only Pulse-owned files/entries.
12. Uninstall removes only Pulse-owned provider integration entries.
13. Uninstall with data preservation leaves no active Link process.
14. Uninstall with data deletion leaves no Pulse breadcrumbs/backups.
15. Failed integration cleanup offers narrow manual steps and does not erase unrelated config.

---

## 12. Design invariants

1. Application install and provider integration enablement are separate permissions.
2. Every configuration mutation is narrow, named, validated, and reversible.
3. Updates yield to active Agent work rather than treating it as a disposable background process.
4. A broken update may reduce Pulse visibility, never change provider behavior.
5. Uninstall is a precise cleanup operation, not a configuration reset bomb.
6. No package lifecycle step expands Pulse’s data collection boundary.
