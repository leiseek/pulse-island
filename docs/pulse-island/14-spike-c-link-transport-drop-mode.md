# Pulse Island · Spike C: Link Wake-Up, Local Transport, and Drop Mode

**Status:** Executable spike plan  
**Goal:** Prove that Pulse Link can be started on demand by a safe local shim, accept bounded Hook envelopes, retain only compact breadcrumbs, serve late Island attachment, and exit cleanly without affecting an Agent when anything fails.  
**Depends on:** `01-privacy-data-boundaries.md`, `03-event-reduction-engine.md`, `06-pulse-link-runtime-architecture.md`, `08-integration-hook-protocol.md`, `11-rust-workspace-architecture.md`, `13-spike-b-state-kernel.md`  
**Last updated:** 2026-07-01

---

## 1. Spike question

Can a per-user Pulse Link runtime provide this complete local-only lifecycle without a real provider adapter?

```text
Synthetic Hook event
→ short-lived Pulse Shim
→ start or reuse exactly one Pulse Link
→ Link validates and reduces event
→ Link writes a bounded breadcrumb
→ Island may attach later and receive a safe snapshot
→ Island disconnects
→ Link returns to Drop Mode
→ last active session ends
→ Link exits after grace period
```

All failure paths must be harmless to the synthetic Agent caller. The shim must fail open and exit successfully for ordinary observation events even when Link is absent, unavailable, malformed, or crashes.

---

## 2. Success outcome

At the end of Spike C, the team can state:

> Pulse Link is an on-demand, single-user local event bridge. It can recover from the Island being absent, retain only a bounded current-session breadcrumb, and disappear after work ends. It does not become an always-on daemon, a transcript store, or a blocking dependency of the Agent.

Spike C proves transport and lifecycle only. It does **not** prove a real Codex, Claude, or Antigravity integration.

---

## 3. Strict scope

### In scope

- `pulse-link.exe` single-instance runtime
- `pulse-link-shim.exe` short-lived ingress executable
- versioned Hook-envelope validation
- per-user/per-logon-session named-pipe transport
- bounded incoming frame parsing
- Link wake-if-needed behavior
- Spike B reducer integration using synthetic events
- in-memory active snapshot registry
- bounded breadcrumb persistence abstraction
- fake Island client that requests a full snapshot and subscribes to deltas
- Island-connected and Drop Mode lifecycle transitions
- no-session grace timer and exit
- resource, race, and fail-open measurement

### Explicitly out of scope

- real provider Hook installation
- provider configuration writes
- actual Codex / Claude / Antigravity process discovery
- provider token/usage reads
- SQLite as a required final storage backend
- Island native UI rendering
- Win32 window activation
- notifications
- approval or decision bridge
- control / stop / steer behavior
- network access

The spike uses only synthetic, already-sanitized provider-like envelopes and a fake Island client.

---

## 4. Minimum crate and binary slice

```text
pulse-domain
pulse-protocol
pulse-reducer
pulse-fuel
pulse-arbitration
pulse-persistence
pulse-win32
pulse-link-core
pulse-testkit
apps/pulse-link
apps/pulse-link-shim
apps/pulse-link-spike-client
fixtures/link
```

`pulse-island-ui` is not required. `pulse-link-spike-client` is a small CLI subscriber that acts as an Island substitute.

---

## 5. Process and trust topology

```text
┌─────────────────────┐
│ Synthetic Hook host │
│ or test runner      │
└──────────┬──────────┘
           │ bounded stdin payload
           v
┌─────────────────────┐
│ pulse-link-shim.exe │
│ short-lived         │
└──────────┬──────────┘
           │ local framed envelope
           │ starts Link only if needed
           v
┌────────────────────────────────────┐
│ pulse-link.exe                      │
│ per-user, per-logon-session runtime │
│                                    │
│ ingress → reducer → snapshot store │
│                    ↓               │
│             breadcrumb store        │
└──────────┬─────────────────────────┘
           │ current-user local pipe
           v
┌─────────────────────────┐
│ spike Island client      │
│ snapshot + subscribe     │
└─────────────────────────┘
```

The synthetic Hook host, Shim, Link, and Island client run under the same Windows user and logon session for this spike.

No component opens a network socket.

---

## 6. Link lifecycle state machine

### 6.1 Process lifecycle

```text
NotRunning
  │ shim or Island wake request
  v
Starting
  │ owns mutex, starts pipe endpoints, restores breadcrumb
  v
Warm
  │ first valid event or recovered active breadcrumb
  ├─────────────────────────────┐
  v                             │
Active                           │ no active sessions
  │ Island attaches              │ after recovery/grace
  v                              │
IslandActive                     │
  │ Island disconnects           │
  v                              │
DropMode ────────────────┐       │
  │ last task terminal   │       │
  v                      │       │
GracePeriod              │       │
  │ new event            │       │
  └──→ Warm/Active       │       │
  │ grace expires        │       │
  v                      │       │
CheckpointAndExit ───────┴───────┘
  v
NotRunning
```

### 6.2 State definitions

| State | Allowed work | Prohibited work |
|---|---|---|
| `Starting` | mutex ownership, pipe setup, initial-envelope handoff, breadcrumb load | UI, provider polling, long recovery scan |
| `Warm` | input acceptance, reducer startup, bounded identity/snapshot recovery | usage history, UI rendering |
| `Active` | reducer, compact state, bounded breadcrumb writes | UI rendering unless a client attaches |
| `IslandActive` | publish snapshot deltas, respond to snapshot request, enable bounded active-mode hooks later | raw event export, uncontrolled history replay |
| `DropMode` | compact event reduction, breadcrumb update, session terminal tracking | token timeline scan, quota refresh loop, D3D/DirectComposition, UI publishing |
| `GracePeriod` | wait for new relevant events, persist terminal snapshot | new recurring scans, long idle work |
| `CheckpointAndExit` | final bounded write, close pipes, release handles | background retry loops |

`Idle` is not a long-lived running state. When no task exists and no grace timer is active, Link exits.

---

## 7. Single-instance and naming model

Pulse Link must never create duplicate runtimes within the same user/logon session.

### 7.1 Namespace inputs

Names are derived from:

- a random per-installation identifier stored in the Pulse-owned local configuration root
- a non-secret hash of the current user SID
- current logon-session identifier
- protocol major version

The raw user SID and raw session identifier must not appear in diagnostics or user-visible logs.

### 7.2 Objects

```text
Mutex:
  Local\PulseIsland.Link.<install-hash>.<session-hash>.v1

Ingress pipe:
  \\.\pipe\PulseIsland.<install-hash>.<session-hash>.ingress.v1

Island pipe:
  \\.\pipe\PulseIsland.<install-hash>.<session-hash>.island.v1

Ready event (optional startup synchronization):
  Local\PulseIsland.LinkReady.<install-hash>.<session-hash>.v1
```

### 7.3 Ownership rule

1. A starting Link attempts to acquire the mutex.
2. If it owns the mutex, it creates pipe servers and becomes the single Link.
3. If mutex already exists, a Shim treats Link as potentially alive and attempts ingress pipe connection.
4. If a stale mutex/pipe condition is detected, retry is bounded. The Shim still fails open rather than waiting indefinitely.

Per-logon-session scoping intentionally avoids cross-session background behavior. An Agent in a separate RDP/console logon session receives an independent Link instance for that session.

---

## 8. Shim design

### 8.1 Responsibility

`pulse-link-shim.exe` is deliberately tiny. It performs only:

1. Read a bounded Hook input from stdin or a specified safe input handle.
2. Parse and validate the outer protocol envelope.
3. Reject forbidden/oversized data before forwarding.
4. Attempt delivery to existing Link.
5. Start Link if no existing Link accepts the envelope.
6. Exit with fail-open semantics.

It must not link:

- D3D, DirectComposition, Direct2D, or DirectWrite
- SQLite or full persistence logic
- provider transcript parsers
- provider-specific activity interpretation
- full state reducer or UI code

### 8.2 Input boundary

Initial spike input is UTF-8 JSON only for easy fixture authoring, read from stdin with a hard maximum of 8 KiB.

The actual wire format between Shim and Link can be a framed compact binary representation, but JSON parsing is permitted at the fixture boundary because it is bounded and short-lived.

Input behavior:

```text
read at most 8 KiB
→ reject if stream exceeds limit
→ parse protocol envelope
→ enforce field and forbidden-key rules
→ construct validated frame
```

The shim never writes the raw stdin content to disk or logs.

### 8.3 Fail-open exit policy

For ordinary observation events, Shim returns exit code `0` after any of the following:

- Link receives and acknowledges the frame.
- Link is unavailable or fails to start.
- Input is malformed or too large.
- Pipe connection times out.
- Link acknowledgement is invalid.

The only non-zero exit codes are reserved for explicit user-run diagnostic commands, not normal Hook operation.

A malformed Hook envelope may increment a bounded diagnostic category counter only after Link is available. It must not cause an Agent task to fail.

### 8.4 Delivery time budget

The Shim has a small fixed budget so it cannot become noticeable in an Agent lifecycle callback.

Initial target:

| Stage | Budget |
|---|---:|
| parse + outer validation | <= 10 ms P95 |
| connect existing Link | <= 25 ms |
| start Link path, non-blocking handoff initiated | <= 150 ms |
| total normal shim lifetime | <= 250 ms P95 |
| hard timeout before fail-open exit | 400 ms |

These are spike targets. A slow machine may make Link continue initializing after the Shim exits, but that must not delay the Agent beyond the hard timeout.

---

## 9. Initial-event handoff

The first event must not be placed on the Link command line, because even a sanitized summary should not become process-list-visible.

### 9.1 Existing Link path

```text
Shim
→ connect ingress pipe
→ send frame
→ wait for small acknowledgement window
→ exit 0
```

### 9.2 New Link path

```text
Shim
→ create anonymous inherited handoff pipe
→ spawn pulse-link --wake-if-needed --handoff-stdin
→ write one validated frame to inherited pipe
→ close write end
→ exit 0 within timeout

Link
→ acquire mutex
→ initialize ingress pipe
→ read initial handoff frame
→ reduce / checkpoint as needed
```

The frame never appears in:

- command-line arguments
- environment variables
- temporary file names
- shell history
- diagnostics output

If Link cannot start, Shim closes the handoff pipe and exits 0. The event may be lost, but the provider remains unaffected and a subsequent supported event can create a new breadcrumb.

---

## 10. Local pipe protocol

### 10.1 Framing

All local messages are length-prefixed and versioned.

```text
FrameHeader
├── magic: [u8; 4]
├── protocol_major: u16
├── protocol_minor: u16
├── message_kind: u16
├── flags: u16
├── request_id: u64
├── payload_length: u32
└── reserved: u32
```

Payloads are parsed only after length validation.

### 10.2 Limits

| Message | Maximum payload |
|---|---:|
| Hook ingress envelope | 8 KiB |
| Island control request | 8 KiB |
| Snapshot delta | 8 KiB |
| Full snapshot | 128 KiB |
| Health report | 8 KiB |

No message supports an arbitrary untyped nested payload field.

### 10.3 Ingress messages

```text
HookEnvelope
ShimHealthHint (diagnostic category only)
```

### 10.4 Island messages

Island request:

```text
Hello
GetSnapshot
Subscribe
Unsubscribe
Ping
RequestLinkWake
```

Link response:

```text
HelloAck
FullSnapshot
SnapshotDelta
LinkHealth
ProtocolError
```

No provider-control command exists in Spike C.

### 10.5 Ordering and recovery

- Link assigns a monotonically increasing `snapshot_revision` to each compact snapshot update.
- Island tracks last accepted revision per Link session.
- On gap, disconnect, or reconnect, Island requests a new `FullSnapshot`.
- Snapshot delivery is at-least-once from the UI perspective; Island deduplicates by task key and revision.
- Raw Hook event replay is never offered.

---

## 11. Pipe security

### 11.1 ACL

Both pipe servers use a security descriptor granting access only to:

- the current user SID
- LocalSystem only if required by Windows infrastructure, otherwise omitted

No network or broad authenticated-user ACL is allowed.

### 11.2 Client validation

Where Windows APIs allow, Link validates the connecting client process and token belong to the expected current user/logon session before accepting control-style Island messages.

Ingress frames from Shims are still schema-validated. Same-user local processes are inside the product's local trust boundary, but malformed data is never trusted.

### 11.3 No executable RPC

The local protocol cannot:

- execute arbitrary commands
- launch arbitrary processes
- read arbitrary file paths
- request provider raw payloads
- invoke arbitrary route targets

Only named, bounded message variants are accepted.

---

## 12. Breadcrumb store

### 12.1 Spike choice

Spike C uses a `BreadcrumbStore` trait with an atomic bounded snapshot-file backend. SQLite remains a later implementation choice for broader retention and usage rollups; it is not needed to prove Drop Mode.

```text
trait BreadcrumbStore {
    fn load(&self) -> Result<BreadcrumbSet, StoreError>;
    fn checkpoint(&self, set: &BreadcrumbSet) -> Result<(), StoreError>;
    fn clear_expired(&self, now: Clock) -> Result<(), StoreError>;
}
```

The backend writes a complete replacement snapshot to a temporary Pulse-owned file and atomically replaces the previous snapshot. It never appends raw event records.

### 12.2 Data budget

Breadcrumb persistence contains only current/recent compact state.

```text
BreadcrumbSet
├── protocol_version
├── written_at
├── active_tasks[0..128]
├── recent_terminal_tasks[0..20]
└── aggregate_diagnostic_counters
```

Per task, retain only fields allowed by the privacy document:

```text
provider
opaque_task_key
opaque_session_ref_or_hash
process_id
process_started_at
workspace_stable_hash
short_workspace_display_name (optional)
coarse_lifecycle
health_state
context_state
capability_summary
safe_task_title (bounded)
last_event_summary (bounded)
safe_error_summary (bounded)
started_at
last_activity_at
terminal_at (optional)
last_verified_at
source_freshness
best_route_hint (bounded, no raw command line)
```

Hard limits:

| Limit | Value |
|---|---:|
| active task count | 128 |
| recent terminal count | 20 |
| maximum on-disk snapshot | 256 KiB |
| maximum per task serialized size | 1 KiB |
| raw event retention | 0 |
| prompt/transcript/diff retention | 0 |

### 12.3 Overflow policy

If the cap is reached:

1. Preserve hard-block and user-needed tasks.
2. Preserve user-pinned/followed tasks.
3. Preserve attached tasks with recent activity.
4. Evict oldest observed-only and expired tasks first.
5. Add only an aggregate overflow diagnostic counter.

Pulse never compensates by writing a larger unbounded file.

### 12.4 Checkpoint policy

Write immediately for:

- new task identity accepted
- waiting state entered or cleared
- failure or limit entered
- terminal state entered
- best context route materially changes

Coalesce ordinary running/activity updates:

- in-memory update immediately
- global dirty flush at most every 10 seconds
- at most one ordinary persistence write per task every 30 seconds

All writes are atomic. A failed write leaves the previous valid snapshot intact and marks Link diagnostics degraded. It must not affect the synthetic Agent.

---

## 13. Drop Mode rules

Drop Mode starts when Link has no Island subscribers.

### 13.1 Allowed

- accept validated envelopes
- invoke Spike B reducer
- maintain compact active task snapshots
- maintain process/session identity hints
- checkpoint bounded breadcrumbs
- determine task terminal state from synthetic events
- run low-frequency grace/expiry timers

### 13.2 Forbidden

- no D3D, DirectComposition, Direct2D, DirectWrite, or UI window creation
- no historical session scan
- no quota polling loop
- no token ledger or burn chart construction
- no provider launch or attach loop
- no raw event queue persistence
- no periodic full filesystem scan
- no network request

### 13.3 Resource policy

Drop Mode must keep only:

- the Link process
- one reducer state map under caps
- pipe listeners
- small timer handles
- compact breadcrumb store buffer

It must not allocate per-task UI structures, graphics resources, or unbounded channel buffers.

---

## 14. Island attach and detach

### 14.1 Late attach flow

```text
Fake Island starts
→ connect Island pipe
→ Hello
→ GetSnapshot
→ Link loads in-memory state, or breadcrumb state if freshly started
→ Link returns FullSnapshot
→ Island Subscribe
→ future compact SnapshotDelta messages arrive
```

A late-attached Island receives current bounded task state only. It does not receive a transcript or raw event history.

### 14.2 Detached behavior

```text
Fake Island disconnects
→ Link removes subscriber
→ Link enters DropMode immediately when no subscribers remain
→ no change to task state
→ no new UI-oriented work
```

### 14.3 Link recovered from breadcrumb

If Link is started by Island with no live adapter source in Spike C:

- restored active tasks are marked degraded until a fresh synthetic event revalidates them
- terminal tasks remain recent-terminal only through retention policy
- no state is promoted to attached merely because it exists on disk

This is the required late-attach honesty model.

---

## 15. Grace period and exit

### 15.1 Active session definition

A task counts as active when lifecycle is one of:

```text
starting
running
waiting_user
paused
stalled
```

A `failed`, `limited`, `completed`, or `terminated` task is not active for Link lifetime after its immediate checkpoint has completed.

### 15.2 Exit rule

```text
last active task ends
→ checkpoint terminal state
→ start 90-second grace period
→ new relevant event arrives: cancel grace, resume Warm/Active
→ no relevant event: checkpoint, close clients/pipes, exit
```

The initial grace duration is configurable internally but fixed at 90 seconds for Spike C measurement.

### 15.3 Island-open with no tasks

An Island connection alone does not keep Link alive indefinitely. If no active tasks exist after recovery and grace, Link sends a final empty/current snapshot, then exits. Island remains capable of requesting a future wake.

---

## 16. Synthetic scenario catalog

### C0: Existing Link delivery

```text
Link already running
→ Shim sends session_started
→ Link acknowledges
→ fake Island requests snapshot
```

Expected:

- one task visible
- no new Link process
- task state from reducer

### C1: First Hook wakes Link

```text
No Link process
→ Shim receives session_started
→ starts Link with inherited handoff
→ Link checkpoints breadcrumb
→ Shim exits 0
→ fake Island attaches later
```

Expected:

- exactly one Link
- safe late snapshot available
- no command-line/event leakage

### C2: Parallel Shim race

```text
No Link process
→ 50 Shims start concurrently
→ mixed activity + waiting + terminal synthetic events
```

Expected:

- one Link process only
- valid events eventually reduced or safely dropped under bounded policy
- no shim exceeds hard timeout
- no synthetic host failure

### C3: Link unavailable

```text
Shim cannot connect and Link executable is intentionally unavailable
```

Expected:

- Shim exits 0 within hard timeout
- synthetic Agent host continues
- no persistent retry daemon appears

### C4: Malformed/oversized ingress

```text
invalid version
forbidden field
9 KiB payload
invalid length prefix
```

Expected:

- rejected before reducer
- no raw payload persists
- Shim exits 0 for Hook mode
- Link remains healthy for subsequent valid frames

### C5: Drop Mode breadcrumb

```text
Link receives start + activity
→ no Island attaches
→ ordinary activity storm
→ waiting
→ terminal
```

Expected:

- only material checkpoints
- bounded snapshot file
- no UI/GPU work
- terminal task survives restart as recent terminal state

### C6: Island attach, detach, reattach

```text
Link in Drop Mode
→ fake Island attach + subscribe
→ receive full snapshot and deltas
→ disconnect
→ Drop Mode
→ reconnect
```

Expected:

- no task duplication
- monotonic revision behavior
- no raw event replay

### C7: Link restart recovery

```text
Link records active breadcrumb
→ process terminated intentionally
→ Link restarts via Island wake
→ returns restored snapshot
→ fresh event arrives
```

Expected:

- restored task begins degraded
- fresh valid event restores attached/appropriate health
- no false completion or failure

### C8: Grace exit

```text
start → completed
→ wait 90 seconds simulated/accelerated
```

Expected:

- final checkpoint
- Link exits
- no leftover pipe/mutex/child process

### C9: Event storm

```text
100k activity events + 100k token-like synthetic updates
```

Expected:

- Link memory stays bounded
- only material snapshot deltas published
- terminal/waiting/failure frames are prioritized
- no unbounded disk growth

---

## 17. Measurement plan

### 17.1 Metrics

Collect only local performance metadata:

- Link private working set
- Link working set
- CPU time delta
- thread and handle counts
- ingress acceptance/rejection counts by category
- shim lifetime
- Link startup time
- snapshot request latency
- snapshot-delta latency
- breadcrumb file size
- number of checkpoint writes
- number of Link process launches
- GPU use indicator, expected zero

No metric contains session title, workspace path, or Hook payload content.

### 17.2 Performance gates

| Metric | Required target |
|---|---:|
| Drop Mode P95 private working set | <= 10 MB |
| Drop Mode P99 private working set | <= 12 MB |
| Drop Mode hard ceiling | <= 16 MB |
| Drop Mode average CPU with no events | <= 0.03% |
| Drop Mode GPU allocation | 0 |
| Shim ordinary lifetime P95 | <= 250 ms |
| Shim hard timeout | <= 400 ms |
| ingress to compact snapshot update P95 | <= 40 ms while Link is warm |
| fake Island full snapshot first response P95 | <= 80 ms |
| fake Island delta after reduced state P95 | <= 40 ms |
| breadcrumb file size | <= 256 KiB |
| Link duplicate processes under C2 | 0 |
| Link residue after C8 | 0 process, 0 owned pipe server, 0 mutex handle leak |

The 10 MB Drop Mode target applies to `pulse-link.exe` only. It does not include a transient Shim that has already exited.

### 17.3 Long-run test

Run C5/C6 mixed traffic for four hours with synthetic event cadence and repeated Island attach/detach.

Acceptance:

- memory growth <= 5 MB from post-warm baseline
- breadcrumb size remains capped
- handle/thread counts remain within a documented stable range
- no unexpected Link restart loop
- no synthetic Agent host stall due to Shim behavior

---

## 18. Security and privacy acceptance

- [ ] Ingress event is not visible in Link command line, environment, or temporary filename.
- [ ] Raw stdin payload is never written to logs or breadcrumb store.
- [ ] Pipe ACL rejects a different Windows user.
- [ ] Unknown message type is rejected safely.
- [ ] Frame length is validated before allocation.
- [ ] Full snapshot contains only compact safe task fields.
- [ ] Breadcrumb file remains within cap under event storm.
- [ ] Link failure does not modify synthetic Agent exit behavior.
- [ ] No network socket is opened.
- [ ] No D3D/UI resource is created by Link or Shim.

---

## 19. Exit criteria

Spike C passes only when:

1. C0–C9 pass on a supported Windows 11 test machine.
2. One and only one Link process appears during concurrent Shim races.
3. Shim failure is demonstrably fail-open for all ordinary Hook scenarios.
4. Island can attach after a Hook-started task has begun and receive a compact current snapshot.
5. Island disconnect returns Link to Drop Mode without task loss.
6. Link exits after its 90-second grace period with no leaked process/pipe/mutex ownership.
7. Breadcrumb persistence remains compact, atomic, and content-minimized.
8. Drop Mode meets the memory, CPU, GPU, and no-network gates.
9. A Link restart restores only degraded breadcrumb state until fresh evidence arrives.
10. No Spike C component reads a real provider transcript or modifies a provider configuration.

---

## 20. Failure interpretation

| Failure | Correct response |
|---|---|
| Shim slows synthetic Agent | reduce blocking work or shorten timeout; never move work into provider callback |
| duplicate Links | repair mutex/ready handshake before adapters |
| event lost during first start | improve inherited handoff path; do not switch to command-line payload or unbounded spool file |
| Drop Mode memory too high | remove libraries/allocations, do not relax 16 MB ceiling |
| breadcrumb grows | tighten caps/eviction, never add append-only logs |
| Island reconnect gives wrong state | fix revision/full-snapshot recovery, not UI caching |
| Link stays alive idle | repair lifecycle state/grace ownership, not background polling |
| malformed input crashes Link | harden framing/admission before any provider test |

---

## 21. Follow-on handoff

After Spike C passes, the first real provider work can begin as a **provider capability probe**, not immediately a full adapter.

Recommended order:

```text
1. Codex CLI probe
2. Claude Code probe
3. Choose the first adapter based on measured official integration quality
4. Build a narrow adapter that targets only proven P0 capabilities
```

No adapter may skip the integration install/rollback, late-attach, truthfulness, context-route, privacy, and resource gates defined in the roadmap.

---

## 22. Design invariants

1. Link is born from work and exits after work. It is not a startup daemon.
2. Shim is a fail-open messenger, not an Agent gatekeeper.
3. Drop Mode keeps a breadcrumb, not a surveillance history.
4. A late Island sees compact current truth, never raw event replay.
5. Link recovery downgrades stale state until fresh evidence proves it again.
6. Every transport and persistence cap is deliberate and testable.
7. A missing or broken Pulse component must reduce observation only, never alter Agent behavior.
