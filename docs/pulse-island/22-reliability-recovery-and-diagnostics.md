# Pulse Island · Reliability, Recovery, and Diagnostics

**Status:** Operational trust contract  
**Applies to:** Island, Link, Shim, integration health, Safe Mode, diagnostics, recovery  
**Depends on:** `01-privacy-data-boundaries.md`, `08-integration-hook-protocol.md`, `14-spike-c-link-transport-drop-mode.md`, `21-install-update-uninstall-contract.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Principle

> A Pulse failure is a Pulse failure.  
> A provider task failure is a provider task failure.  
> Missing evidence makes Pulse say less, not more.

Pulse preserves priorities in this order:

```text
provider behavior unchanged
→ provider configuration valid
→ sensitive content uncollected
→ task truth honest
→ Pulse resource bounded
→ convenience/history where safe
```

---

## 2. Failure domains

| Domain | Example | User treatment |
|---|---|---|
| Provider task | Verified waiting, failure, completion, limit block | Normal task state/attention signal |
| Provider integration | Hook removed, schema drift, managed policy block | Integration health, not task failure |
| Pulse runtime | Island crash, Link restart, pipe disconnect, breadcrumb write failure | Pulse degraded/recovering status |
| Local environment | Disk unavailable, session switch, update boundary | Local warning only when it affects Pulse evidence |

A lost Hook never turns a provider task red.

---

## 3. Runtime failure behavior

### Island absent or crash

```text
Island absent/disconnects
→ Link enters Drop Mode
→ compact breadcrumb continues only within policy
→ provider remains unchanged
→ later Island receives FullSnapshot + SnapshotDelta only
```

### Link unavailable or crash

```text
provider Hook invokes Shim
→ Link unavailable/crashed
→ Shim fails open
→ provider continues
→ later wake may restore bounded state as Degraded
→ fresh evidence may recover task health
```

Pulse never infers completion/failure during Link absence.

### Breadcrumb failure

```text
checkpoint fails
→ in-memory state may remain valid
→ persistence health becomes degraded
→ later material checkpoint retries with bounded backoff
→ no raw log/spool fallback
```

### Provider configuration drift

```text
Pulse-owned entry removed/invalidated
→ integration health = needs_repair or blocked
→ existing task evidence expires normally
→ provider untouched
```

Repair is explicit user action. Pulse does not auto-edit provider config in the background.

---

## 4. Automatic recovery limits

Pulse may:

- reconnect Island to Link
- restart Link only when explicitly woken by Hook or Island request and Safe Mode is off
- reload compact breadcrumb state
- retry bounded local checkpoint later
- reopen its own pipe endpoint after transient failure

Pulse may not:

- restart/stop provider processes
- rerun a task
- resume a stored provider session
- approve a permission
- reinstall provider integration automatically
- scan provider history to reconstruct missed activity
- turn an Observed task into Attached after restart without fresh source evidence

---

## 5. Safe Mode

Safe Mode is an operational circuit breaker, not an Agent error.

```text
Safe Mode enabled
→ Shim reads current-user flag
→ no Link wake
→ no ingress forwarding
→ Shim exits success within fail-open budget
→ Island does not request Link wake
→ Island presents Passive mode only
```

Existing provider Hook entries remain unchanged. The provider can still invoke the Shim, but the Shim becomes a fast no-op for Pulse and does not alter provider-native behavior.

Safe Mode may be entered by user action, repeated Pulse crash threshold, repeated malformed ingress, incompatible protocol, or failed Pulse migration.

Safe Mode may be exited only by explicit user action after review/re-enable. It never silently rewrites provider configuration.

---

## 6. Privacy-profile recovery rules

| Profile | Recovery ceiling |
|---|---|
| Minimal local state | Bounded active/recent breadcrumb may restore as Degraded. |
| Strict local state | Active nonterminal breadcrumb may restore as Degraded; terminal breadcrumb must never resurrect. |
| Passive-only | No integration breadcrumb recovery. |

A recovery path never bypasses retention policy to improve convenience.

---

## 7. Resource watchdog

Pulse watches only bounded local metrics:

```text
private working set
CPU time window
handle/thread trend
graphics allocation count for Island
Link grace duration
breadcrumb size
```

| Condition | Response |
|---|---|
| Transient soft breach | Reduce nonessential Pulse motion/refresh work. |
| Sustained soft breach | Disable optional Pulse features and mark diagnostics degraded. |
| Hard process-tree breach | Checkpoint then stop optional Pulse component; provider untouched. |
| Repeated breach | Recommend Safe Mode. |

The watchdog never kills provider processes or hides a task state to make Pulse look healthy.

---

## 8. Diagnostics

Diagnostics are user-initiated, previewable, local-first, and content-minimized.

Allowed:

```text
Pulse version/build
Windows category
Island/Link state
integration health
safe category codes
protocol compatibility
aggregate memory/CPU/handles
breadcrumb count/size
feature capability flags
```

Excluded by default:

```text
task title
workspace path/display name
session ID
raw event summary
provider config text
prompt/transcript/tool content
credentials/cookies/tokens
raw pipe name/SID
unredacted stack trace
```

No automatic diagnostic or crash upload in P0. Local crash markers contain only process role, version, timestamp, category, and high-level active state.

---

## 9. Required reliability scenarios

1. Kill Island during active synthetic task: Link enters Drop Mode; provider host continues.
2. Kill Link: next Hook wakes it; restored task starts Degraded.
3. Make breadcrumb storage unwritable: provider continues; Pulse reports persistence issue.
4. Disconnect/reconnect Island: FullSnapshot restores current state without event replay.
5. Remove Pulse-owned Hook: provider works; integration becomes needs repair.
6. Malformed provider envelope: rejected; provider continues.
7. Strict terminal state followed by restart: terminal breadcrumb absent.
8. Existing Hook with Safe Mode: Shim exits success; no Link process starts.
9. Default diagnostic export excludes content/path/session/config fields.
10. Resource breach reduces Pulse only; provider remains unaffected.

---

## 10. Design invariants

1. Pulse errors never masquerade as Agent errors.
2. Recovery restores observation only, never provider work.
3. Safe Mode makes Pulse smaller, not more invasive.
4. Diagnostics explain categories, not user content.
5. Retention profile limits recovery as well as normal storage.
