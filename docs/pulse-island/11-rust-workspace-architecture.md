# Pulse Island · Rust Workspace Architecture

**Status:** Implementation architecture baseline  
**Applies to:** Repository structure, crate dependency rules, binary boundaries, test topology  
**Depends on:** `00` through `10` design documents  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Island needs a repository structure that preserves three promises while the product grows:

1. The state engine remains deterministic and testable without Windows UI or live providers.
2. Provider adapters cannot leak raw payloads into persistence, IPC, or rendering.
3. The native UI can evolve without turning Pulse Link into a UI-shaped background service.

The workspace must favor small, explicit crates with one-way dependencies. A future feature should be able to add an adapter or a visual treatment without pulling provider-specific logic into the entire application.

---

## 2. Workspace shape

```text
pulse-island/
├── Cargo.toml
├── crates/
│   ├── pulse-domain/
│   ├── pulse-protocol/
│   ├── pulse-reducer/
│   ├── pulse-fuel/
│   ├── pulse-routing/
│   ├── pulse-arbitration/
│   ├── pulse-persistence/
│   ├── pulse-win32/
│   ├── pulse-link-core/
│   ├── pulse-island-ui/
│   ├── pulse-diagnostics/
│   ├── pulse-testkit/
│   └── adapters/
│       ├── pulse-adapter-process/
│       ├── pulse-adapter-codex/
│       ├── pulse-adapter-claude/
│       └── pulse-adapter-antigravity/
├── apps/
│   ├── pulse-island/
│   ├── pulse-link/
│   └── pulse-link-shim/
├── fixtures/
│   ├── reducer/
│   ├── adapters/
│   ├── routing/
│   └── performance/
├── docs/
│   └── pulse-island/
└── tools/
    └── bench/
```

The physical folder tree may evolve, but the dependency directions in this document are normative.

---

## 3. Dependency direction

```text
apps
  ↓
pulse-island-ui / pulse-link-core
  ↓
pulse-win32 / pulse-persistence / adapters
  ↓
pulse-routing / pulse-arbitration / pulse-fuel / pulse-reducer
  ↓
pulse-protocol / pulse-domain
```

### 3.1 Rules

- `pulse-domain` has no Win32, I/O, provider, SQLite, or renderer dependency.
- `pulse-protocol` has no direct provider dependency and no persistence implementation.
- `pulse-reducer` remains pure or near-pure and cannot call the filesystem, named pipes, timers, or provider APIs.
- `pulse-island-ui` only consumes compact snapshots and `PresentationPlan` outputs.
- Provider adapters cannot write SQLite or send UI messages directly.
- `pulse-win32` provides platform primitives. It does not understand Codex, Claude, task state, or product policy.
- Apps are composition roots only. They wire dependencies; they should not contain business rules.

### 3.2 Forbidden dependency edges

```text
pulse-island-ui → adapters
pulse-island-ui → pulse-persistence
adapters → pulse-island-ui
adapters → pulse-persistence
pulse-reducer → pulse-win32
pulse-domain → any external provider crate
pulse-routing → adapters
```

If a proposed feature needs one of these edges, the design should introduce a narrow trait or protocol type instead of breaking the architecture.

---

## 4. Crate responsibilities

### 4.1 `pulse-domain`

The smallest semantic core.

Owns:

- provider identifiers
- opaque task/session keys
- lifecycle, attention, health, context, and capability enums
- bounded safe summary value types
- task snapshot types
- timestamps and duration wrappers
- invariant-aware constructors

Must not own:

- serde formats for external transport
- SQL schemas
- Windows handles
- mutable stores
- provider event formats

### 4.2 `pulse-protocol`

Versioned, bounded interchange types.

Owns:

- `PulseHookEnvelope`
- `NormalizedEvent`
- named-pipe request/response envelopes
- snapshot delta wire types
- message framing constraints
- protocol version negotiation helpers

Must enforce:

- field caps
- schema version checks
- no unbounded arbitrary payload map
- no raw provider object escape hatch

### 4.3 `pulse-reducer`

The truth-preserving state kernel.

Owns:

- admission validation result types
- identity reconciliation
- field evidence precedence
- coalescing decisions
- lifecycle transition logic
- freshness decay
- state reduction result
- safe snapshot mutation

Primary public shape:

```text
reduce(previous: TaskSnapshot, event: AdmittedEvent, now: Clock)
→ ReductionResult
```

This crate is the main home of deterministic fixtures.

### 4.4 `pulse-fuel`

Usage and quota logic, isolated from lifecycle state.

Owns:

- quota snapshot normalization
- token counter delta handling
- coarse ledger bucket update
- burn-meter calculation
- source provenance labels
- low-fuel threshold decision input

It must not claim an account quota from local task tokens.

### 4.5 `pulse-routing`

Context-route construction and validation policy.

Owns:

- route strength
- anchor validity model
- fallback-chain selection
- user-visible action labels
- route expiration / revalidation decisions

It does not activate Windows windows directly. It describes what should be activated and why.

### 4.6 `pulse-arbitration`

Turns task snapshots into one stable presentation plan.

Owns:

- attention class mapping
- primary selection
- hysteresis and minimum display holds
- Peek selection
- aggregate state selection
- notification candidate generation
- Fuel Thread candidate selection

It must not inspect raw provider events or direct UI state.

### 4.7 `pulse-persistence`

Stores bounded local state only.

Owns:

- breadcrumb store
- SQLite schema/migrations for allowed tables
- retention jobs
- atomic write helpers
- installation backup fragments

Rules:

- no raw event log table
- no transcript table
- no prompt/history table
- storage DTOs are separate from domain types
- all writes use bounded payload sizes

### 4.8 `pulse-win32`

Windows platform adapter.

Owns:

- named-pipe server/client implementation
- current-user ACL utilities
- user-session mutex
- process start-time and parent binding lookup
- window enumeration and safe focus primitives
- DPI, monitor, fullscreen/immersive detection primitives
- global hotkey registration
- performance measurement primitives

It exposes small capability-oriented APIs. It must not become a dumping ground for product logic.

### 4.9 `pulse-link-core`

Pulse Link orchestration.

Owns:

- adapter runtime lifecycle
- ingress event routing
- reducer invocation scheduling
- breadcrumb persistence scheduling
- Island connection/session registry
- Drop Mode vs Island-active mode
- adapter health aggregation

It does not render and does not define lifecycle semantics itself.

### 4.10 `pulse-island-ui`

Native view layer.

Owns:

- HWND/window composition
- DirectComposition scene graph
- Direct2D/DirectWrite drawing
- Signal / Peek / Focus Card / Palette view models
- hit testing
- reduced-motion, high-contrast, and DPI adaptation
- translation of `PresentationPlan` into visuals

It never uses provider-specific types.

### 4.11 `pulse-diagnostics`

User-initiated, privacy-preserving diagnostics.

Owns:

- health summaries
- bounded performance report creation
- redacted diagnostic export
- support-bundle preview manifest

It must not serialize raw provider payloads or unredacted persistence records.

### 4.12 `pulse-testkit`

Shared tests and fixtures.

Owns:

- fixed clock
- fake pipe transport
- fake process/window registry
- reducer sequence harness
- snapshot assertions
- deterministic workload generator
- measurement scenario runner interface

No production crate may depend on `pulse-testkit` outside test/dev dependency contexts.

---

## 5. Binary applications

### 5.1 `apps/pulse-island`

`pulse-island.exe`

Composition root for:

```text
Named Pipe client
+ presentation state store
+ native UI controller
+ global shortcut
+ Context Route action invoker
+ settings access
```

It may start even when Link is absent. In that case it requests a Link wake-up or enters a safe empty/Passive presentation state.

### 5.2 `apps/pulse-link`

`pulse-link.exe`

Composition root for:

```text
single-instance guard
+ pipe servers
+ built-in adapters
+ reducer
+ snapshot store
+ persistence
+ adaptive scheduler
+ Island publisher
```

### 5.3 `apps/pulse-link-shim`

`pulse-link-shim.exe`

A tiny short-lived Hook target.

Owns only:

- narrow Hook envelope input read
- early validation and field limits
- Link wake-if-needed
- local pipe delivery
- fail-open exit behavior

The Shim must not link UI libraries, SQLite, D3D, or provider-specific transcripts/parsers.

---

## 6. Adapter boundaries

Built-in adapters live under `crates/adapters/` and share a minimal internal adapter API.

```text
Adapter → Candidate discovery / NormalizedEvent / CapabilityDelta / RouteCandidate / UsageInput
```

Adapters must not:

- mutate task snapshots
- write persistence directly
- publish to Island pipes
- use UI labels
- own background scheduler policy

### 6.1 Adapter runtime registration

`pulse-link-core` receives a registry of compiled-in adapters. Each adapter declares:

- integration modes
- required capabilities
- source freshness policy
- schema allow-list version
- supported route categories
- test fixture inventory

No dynamic third-party code loading is required for MVP.

---

## 7. Cross-cutting ownership rules

### 7.1 Time

All domain logic receives an explicit clock abstraction. Wall-clock access occurs only at edges.

### 7.2 IDs

- task/session values are opaque domain identifiers.
- raw provider session IDs stay at adapter/protocol edge or encrypted/bounded persistence layer as necessary.
- logs and diagnostics use opaque hashes/short references.

### 7.3 Errors

Error categories are structured:

```text
adapter_error
protocol_error
persistence_error
routing_error
ui_error
platform_error
```

User-visible errors are safe summaries, never raw provider or OS diagnostic strings by default.

### 7.4 Logging

Production logs are category-first and content-minimized.

Allowed examples:

```text
adapter=claude state=degraded reason=pipe_timeout
route=workspace status=failed reason=path_unavailable
```

Disallowed examples:

```text
raw Hook JSON
prompt text
command arguments
provider transcript line
```

---

## 8. Features and build policy

### 8.1 Feature flags

Use compile-time feature flags sparingly for provider adapters and developer instrumentation.

Examples:

```text
adapter-codex
adapter-claude
adapter-antigravity-experimental
diagnostics
perf-instrumentation
```

Feature flags must not change core task semantics. A disabled adapter removes its ingress source; it cannot silently alter state interpretation for other providers.

### 8.2 Build profiles

Maintain separate profiles for:

- developer iteration
- measurement/benchmark
- release

Benchmark builds must retain enough diagnostics to verify memory, CPU, handles, and latency without retaining user content.

---

## 9. Testing topology

| Layer | Test style |
|---|---|
| domain/protocol | unit tests and invariant/property tests |
| reducer/fuel/routing/arbitration | deterministic sequence fixtures |
| persistence | migration, retention, bounded-size tests |
| win32 | focused integration tests on Windows |
| link core | fake pipe/process/adapter integration tests |
| Island UI | view-model tests plus manual native visual acceptance |
| adapters | provider fixture tests and live probe suites |
| full product | Gate A–H scenario harnesses |

### 9.1 Golden snapshot policy

Use structured expected snapshots rather than screenshot-only tests for task truth. Visual screenshots are useful for UI regressions, but they cannot prove lifecycle correctness.

---

## 10. Repository hygiene

- Each crate keeps a short README stating purpose, dependencies, and invariants.
- Each provider adapter keeps a capability matrix and known-degradation list beside its fixtures.
- Generated diagnostics, measurement data, and local provider samples stay outside source control unless explicitly sanitized fixture data.
- No copied real transcript is allowed in fixtures.
- Design files remain under `docs/pulse-island/` and must be updated with material architecture changes.

---

## 11. First scaffold order

1. Cargo workspace root and lint policy.
2. `pulse-domain`, `pulse-protocol`, `pulse-testkit`.
3. `pulse-reducer`, `pulse-arbitration`, `pulse-routing`, `pulse-fuel`.
4. Fixture harness and deterministic state tests.
5. `pulse-win32` minimal platform layer.
6. `pulse-island-ui` Spike A shell.
7. `pulse-link-core` and `pulse-link-shim` Spike C transport.
8. First provider adapter only after Gates A–C.

---

## 12. Design invariants

1. Core task truth compiles and tests without a Windows window or real provider installed.
2. UI cannot depend on provider-specific code.
3. Provider adapters cannot bypass sanitization, reduction, or persistence policy.
4. Binary applications wire components but do not own product logic.
5. Every new crate must have a sharply bounded reason to exist.
6. A shared `utils` crate is prohibited unless it has a specific semantic owner and dependency rationale.
