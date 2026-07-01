# Pulse Island · Multi-Agent Arbitration

**Status:** Normative presentation-selection contract  
**Applies to:** Compact Island Signal, Peek, Focus Card ordering, attention candidates, Fuel Thread election  
**Depends on:** `02-agent-state-model.md`, `03-event-reduction-engine.md`, `25-consistency-closure.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Island receives many task snapshots but presents one compact attention story. Arbitration transforms trusted task state into a stable `PresentationPlan`.

It answers:

> Which task, aggregate, or state deserves the primary Island surface right now?

Arbitration consumes reduced snapshots only. It never reads raw provider events, transcript content, command output, or provider-specific payloads.

---

## 2. Core rules

1. The compact island tells one primary narrative, not a timer carousel.
2. Priority is a lexicographic tier order, not a hidden weighted score that may reverse product policy.
3. Fuel is secondary to task attention and cannot steal the primary state merely because usage is high.
4. Process-only or degraded evidence may inform Passive/Observed presentation but cannot impersonate Attached lifecycle certainty.
5. A task opened through a verified context route receives an attention lease; repeating the same nudge should become quieter.
6. Urgent new truth may preempt immediately. Ordinary work must respect a minimum display hold to avoid flicker.

---

## 3. Inputs and outputs

### 3.1 Inputs

```text
TaskSnapshot[]
UserAttentionPreferences
UserPins / Follow / WorkspaceMute
AttentionLeases
ImmersiveState
Clock
```

### 3.2 Output

```text
PresentationPlan
├── primary: PrimaryPresentation | None
├── peek_items: [PeekItem; 0..3]
├── aggregate: AggregatePresentation | None
├── fuel_thread: FuelPresentation | None
├── notification_candidate: NotificationCandidate | None
├── visibility_policy
└── revision_basis
```

`PresentationPlan` is a pure projection. It must be reproducible from the same snapshots, preferences, leases, and clock.

---

## 4. Canonical primary order

The following is the only cross-task priority order.

```text
1. failed_or_nonquota_hard_block
2. waiting_user
3. verified_limit_reached
4. user_pinned
5. high_confidence_fuel_risk
6. resource_caused_stall
7. running
8. recent_terminal
9. idle_or_observed
```

### 4.1 Tier meanings

| Tier | Definition | Compact intent |
|---|---|---|
| failed_or_nonquota_hard_block | Explicit provider failure or verified non-quota hard block preventing progress | Red, actionable problem |
| waiting_user | Verified request for user confirmation, permission, or decision | Yellow, return to original context |
| verified_limit_reached | Trusted usage/rate limit currently blocks task progress | Red, but below a verified waiting request |
| user_pinned | User-selected task among otherwise ordinary work | Follow user intent |
| high_confidence_fuel_risk | Trusted warning, not a block | Secondary warning, may elect Fuel Thread |
| resource_caused_stall | Confirmed local resource cause blocks task progress | Watch / degraded attention |
| running | Attached current activity or reliable running freshness | Green / aggregate active work |
| recent_terminal | Completed, terminated, or limited/failure result after attention grace | Informational settle-out |
| idle_or_observed | No active trustworthy task narrative or process-only presence | Hidden, parked, or muted Observed entry |

### 4.2 Non-negotiable distinctions

- `verified_limit_reached` is not merged into generic `hard_block` for arbitration.
- A high usage percentage is not `verified_limit_reached`.
- A stalled process is not failed.
- A process-only candidate cannot reach `waiting_user`, `running`, `completed`, or `failed` without provider evidence.
- A provider integration failure is not a task failure.

---

## 5. Candidate construction

Each task becomes at most one `AttentionCandidate`.

```text
AttentionCandidate
├── task_key
├── tier
├── evidence_health
├── attention_state
├── lifecycle_state
├── route_strength
├── user_pin/follow state
├── lease_state
├── freshness
├── last_material_change_at
└── safe display fields
```

### 5.1 Eligibility

A task is eligible for primary selection when it has a material state or attention reason. Purely observed process candidates are eligible only for muted Passive presentation and may not displace an Attached running/waiting/failed task.

### 5.2 Materiality

Recompute/publish only when one of these changes:

- primary tier or primary task
- visible reason
- visible route label/strength
- aggregate active count
- verified limit/fuel condition
- user pin/follow/mute
- attention lease state
- immersive visibility policy

Repeated ordinary activity heartbeat does not create a new primary animation or notification candidate.

---

## 6. Tie-breakers within a tier

Tie-breakers never cross the tier order.

```text
1. stronger evidence health
2. user-follow / user-pin where the tier allows it
3. unacknowledged attention over leased attention
4. more recent material transition
5. stronger route strength
6. shorter safe label / stable task-key deterministic tie-break
```

### 6.1 Evidence health

Within a tier:

```text
attached > observed > degraded > offline
```

A degraded failed task may remain primary over ordinary running work because failure is a higher tier, but its UI must disclose degraded source confidence when relevant.

### 6.2 Attention lease

After the user successfully opens an exact original context:

```text
exact route success → 5-minute attention lease
```

After a Useful/Strong fallback route succeeds:

```text
route-attempt quiet window → 60 seconds
```

The 60-second route-attempt quiet window suppresses duplicate nudges but does not mean the user solved the task. A new escalation or higher tier may preempt either lease immediately.

---

## 7. Hysteresis and display holds

### 7.1 Immediate preemption

A candidate may preempt immediately when:

- it enters `failed_or_nonquota_hard_block`
- it enters `waiting_user` from a lower tier
- it enters `verified_limit_reached` while current primary is lower tier
- the current primary becomes invalid, stale, muted, or disappears

### 7.2 Minimum hold

For ordinary non-urgent transitions, preserve the primary for a short minimum hold to avoid visual churn.

Initial defaults:

```text
ordinary primary hold: 6 seconds
recent terminal settle-out: 6 seconds
```

The hold does not block an urgent higher-tier preemption.

### 7.3 No rotation

The compact Island must not rotate through multiple running tasks on a timer. When no task has meaningful priority over the others, show aggregate active work.

---

## 8. Aggregate active work

When multiple ordinary tasks are active and none outranks the group:

```text
● 3 agents working
```

An aggregate:

- represents count and shared ordinary state only
- never creates a synthetic merged task/session
- never merges lifecycle truth across providers
- never hides a waiting, failed, or verified limited task

Workspace clusters may help navigation in Peek/Palette but do not merge task identity or lifecycle state.

---

## 9. Peek ordering

Peek shows up to three ranked items.

Ordering:

```text
primary candidate
→ next highest unmuted actionable candidates
→ optionally one aggregate/observed row when useful
```

Each Peek row contains only:

- provider/workspace-safe subject
- state/reason
- duration/fuel only when actionable
- strongest verified route label

Peek never reveals raw logs, prompt text, commands, diffs, transcripts, or code.

---

## 10. Fuel Thread election

Fuel Thread is secondary and independent from the traffic-light state.

### 10.1 Eligibility

Fuel Thread may appear only when:

- a provider-specific source has verified a scoped Fuel capability
- source freshness is within policy
- the value represents either high-confidence risk or verified limit state
- a task/provider association is safe to show

### 10.2 Prohibitions

Fuel Thread must not:

- create a primary red state from a high percentage alone
- aggregate quota windows across providers
- imply task token usage from account quota data
- appear for process-only tasks
- generate a Toast merely for 85%/92% warning thresholds

A verified limit blocking task progress maps to `verified_limit_reached`; the Fuel Thread may accompany it but does not define its priority.

---

## 11. Notification candidate policy

Arbitration emits a candidate. Notification policy decides whether to show Island-only or Toast.

### 11.1 Eligible transitions

- failed_or_nonquota_hard_block entered
- waiting_user entered
- verified_limit_reached entered
- meaningful escalation after lease expiry

### 11.2 Normally silent

- task start
- ordinary progress/activity
- ordinary completion
- high Fuel warning without actual block
- repeated waiting/error while lease or mute is active

### 11.3 Grouping

Multiple eligible transitions within 30 seconds become one aggregate notification candidate. Notification actions may open original context or mute; P0 does not expose provider control decisions.

---

## 12. Immersive and mute policy

When fullscreen, presentation, screen share, game, Focus/Quiet mode, or workspace mute policy is active:

- compact Island may hide
- nonessential motion stops
- Toast candidates are suppressed/queued according to user preference
- task truth continues to update if source exists
- no route is activated automatically

On return, show at most one concise summary. Do not replay every missed event.

---

## 13. Required fixtures

```text
failed + waiting_user + limited + running
→ primary = failed

waiting_user + limited + running
→ primary = waiting_user

limited + pinned running
→ primary = limited

waiting_user + high_fuel_risk
→ primary = waiting_user

three ordinary attached-running tasks
→ aggregate active

process-only observed + attached running
→ attached running

opened exact context + unchanged waiting
→ no duplicate attention candidate during 5-minute lease

strong fallback route + unchanged waiting
→ route-attempt quiet only; task remains unresolved
```

---

## 14. Design invariants

1. One compact Island means one primary story.
2. Tier order is explicit and cannot be reversed by hidden weights.
3. Failed, waiting, limit reached, and Fuel warning are distinct states.
4. Fuel is a resource signal, not a substitute traffic light.
5. User attention leases reduce repetition, never hide escalation.
6. Aggregates improve calmness without fabricating shared task truth.
7. Provider health failures remain separate from task attention.
