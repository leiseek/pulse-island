# Pulse Island · Native Island UI System

**Status:** UI architecture baseline  
**Applies to:** `pulse-island.exe`, rendering, interaction, accessibility, window lifecycle  
**Depends on:** `00-product-foundation.md`, `04-multi-agent-arbitration.md`, `05-context-routing.md`, `06-pulse-link-runtime-architecture.md`  
**Last updated:** 2026-07-01

---

## 1. Purpose

Pulse Island is a desktop signal, not a permanent dashboard. Its UI must make a current agent state legible in under a second, remain beautiful under frequent state changes, and cost almost nothing when quiet.

This document defines how the native Windows UI renders the output of Multi-Agent Arbitration without reinterpreting task state.

```text
Pulse Link snapshots
→ Arbitration PresentationPlan
→ Island view model
→ native render tree
→ Windows compositor
```

The Island never reads raw provider events and never independently sorts tasks.

---

## 2. Non-negotiable UI principles

1. **One primary story.** The compact island expresses only the current primary narrative.
2. **State before statistics.** Red/yellow/green/off communicates task attention before Fuel or system information.
3. **Fuel is secondary.** Fuel Thread is subordinate to task state and never replaces it.
4. **Progressive disclosure.** Signal → Peek → Focus Card. No mini IDE.
5. **No focus theft.** The compact island is non-activating; focused input surfaces are explicit.
6. **Calm motion.** Animation expresses state change, never continuous spectacle.
7. **Native all the way down.** No browser runtime, WebView, HTML/CSS layout engine, Electron, or Tauri.
8. **Cost follows attention.** Hidden and idle UI must stop unnecessary rendering work.

---

## 3. Native graphics stack

Primary stack:

```text
Rust 2024
+ windows-rs
+ Win32 HWND / WndProc
+ D3D11 device
+ DirectComposition
+ Direct2D
+ DirectWrite
```

### 3.1 Responsibilities

| Layer | Responsibility |
|---|---|
| Win32 | Window creation, input, positioning, DPI, hotkeys, lifecycle. |
| D3D11 | Shared device and composition-compatible surfaces. |
| DirectComposition | Property animation and compositor-driven transforms/opacity. |
| Direct2D | Compact vector drawing, shape fills, subtle materials. |
| DirectWrite | Text layout, glyph rasterization, truncation, localization. |

The UI owns no browser process and no JavaScript runtime.

---

## 4. Window model

### 4.1 Compact island window

The compact island is a borderless Win32 popup:

```text
WS_POPUP
WS_EX_TOPMOST
WS_EX_TOOLWINDOW
WS_EX_NOACTIVATE
WS_EX_LAYERED (only if required by chosen composition path)
```

Behavior:

- stays above ordinary windows
- is omitted from Alt+Tab
- does not take keyboard focus on passive hover
- uses `WM_NCHITTEST` to keep noninteractive transparent areas click-through
- handles only explicit interactive regions as client hit targets
- supports per-monitor DPI

The entire window must never be permanently click-through because Peek, drag, pin, and controls need real hit targets.

### 4.2 Focused surfaces

Peek, Focus Card, and Command Palette may use separate coordinated windows or a carefully managed expansion surface. They gain focus only after explicit user invocation:

- click
- keyboard shortcut
- keyboard navigation initiated by user

### 4.3 Placement

Default placement is user-configurable and remembers per-monitor logical coordinates. Placement is clamped to the current work area after display topology changes.

Pulse must not use a fake notch or occupy the Windows taskbar region. It is a floating desktop signal.

---

## 5. View hierarchy

```text
IslandRoot
├── SignalSurface
│   ├── StateGlyph
│   ├── PrimaryLabel
│   ├── SecondaryReason
│   ├── ActiveCountBadge
│   └── FuelThread
├── PeekSurface (on demand)
│   └── PeekTaskRows[0..3]
├── FocusCardSurface (on demand)
│   ├── TaskIdentity
│   ├── StateReason
│   ├── LastTrustworthyEvent
│   ├── FuelSummary
│   ├── ContextRouteAction
│   └── SecondaryActions
└── CommandPaletteSurface (explicit shortcut)
```

Each layer has a discrete visibility state. Hidden layers allocate no per-frame layout work.

---

## 6. Signal surface

### 6.1 Compact layout

The compact surface contains at most:

```text
[state glyph] [subject] [reason or fuel fact] [+N]
```

Examples:

```text
● Codex · Running tests · 5h 92%
! Claude · Needs confirmation · +2
× Codex · Build failed
● 3 agents working · Fuel low
```

If width is constrained, preserve in this order:

1. state glyph
2. provider/workspace subject
3. short reason
4. active count
5. secondary fuel text

Text must truncate safely, never overflow or resize the island unpredictably.

### 6.2 State glyph

The glyph represents the Arbitration primary class, not raw per-task color.

| State | Motion / material intent |
|---|---|
| Running | restrained low-amplitude compositor breathing |
| Waiting | soft periodic pulse, no aggressive flash |
| Failed / limited | two-step attention pulse, then stable state |
| Completed | brief confirmation settle-out |
| Observed / degraded | muted static treatment |
| Idle | hidden or parked, no sustained drawing |

Color is never the sole representation. Peek and Focus Card always display an accessible text label.

### 6.3 Fuel Thread

Fuel Thread is a thin bottom or side-edge treatment. It shows only when Arbitration elects a trustworthy low-fuel candidate.

It must not:

- create a second traffic light
- blink continuously
- imply a quota percentage when data is unavailable
- override a waiting or failure state

Non-color cues include line length and a restrained texture change for elevated risk.

---

## 7. Peek surface

Peek is a short attention queue, not a task manager.

### 7.1 Invocation

- click compact island
- hover after a short intentional dwell, if user enables hover expansion
- keyboard shortcut

### 7.2 Content rules

- maximum three ranked items
- no raw logs, transcripts, plans, diffs, or code
- one reason per task
- include duration/fuel only when it changes actionability
- show context-route affordance, not a button cluster

### 7.3 Interaction

- click row: open Focus Card for that task
- click route affordance: invoke best verified route
- pin/follow/mute are secondary contextual actions, not always visible
- Escape closes Peek without altering focus elsewhere

Peek does not re-run Arbitration. It renders the current PresentationPlan.

---

## 8. Focus Card

Focus Card gives enough trustworthy context to decide whether to return to the original tool.

### 8.1 Required content

```text
provider
safe task title or generic label
workspace display name when permitted
current state and reason
session duration
one last trustworthy event
Fuel summary with source labels
best context action
```

### 8.2 Route label fidelity

The primary action label is supplied by Context Routing:

- `Open original task`
- `Focus terminal tab`
- `Open workspace`
- `Open official usage`
- `Show process details`

The UI must not rewrite these labels into a stronger claim.

### 8.3 Control affordances

P0 Focus Card does not include Approve, Reject, Resume, Stop, or Steer buttons. A future capability may add a control affordance only when:

- adapter advertises an available formal control capability
- user has enabled it
- control safety review exists
- the action has clear context and native fallback

---

## 9. Command Palette

Command Palette is explicit, keyboard-focused navigation.

### 9.1 Shortcut

A user-configurable global shortcut opens the palette. It is the only Pulse surface that may capture keyboard focus by default.

### 9.2 Initial commands

```text
Open active task
Open workspace
Show lowest fuel window
Open provider usage
Show active agents
Pin task
Follow task
Mute task/workspace
Open Pulse settings
```

No high-risk provider control command appears in P0.

### 9.3 Rendering constraints

- query results are virtualized
- no network search
- no provider transcript indexing
- no command history containing sensitive task content

---

## 10. Animation system

### 10.1 Composition-first animation

Use DirectComposition for opacity, translation, scale, corner/shape transitions, and subtle breathing. The UI thread should not hand-roll a 60 Hz state timer for static visuals.

### 10.2 Animation classes

| Class | Trigger | Duration policy |
|---|---|---|
| Arrival | island becomes visible | short, one-shot |
| State transition | primary narrative changes | short, interruptible |
| Attention pulse | waiting/failure | bounded, sparse, settle to static |
| Expansion | Peek/Focus opens | short, compositor-driven |
| Fuel cue | low-fuel threshold crossed | one subtle entry transition |
| Completion | task completes | brief confirmation then clear |

### 10.3 Reduced motion

Respect Windows accessibility preferences and a Pulse-specific Reduced Motion setting.

Reduced Motion behavior:

- replace pulses with static emphasis
- replace scale/slide expansion with opacity changes or immediate state
- retain text/state changes

---

## 11. Rendering budget

### 11.1 Static state

When the island has no state transition and no active compositor animation:

- no continuous redraw
- no app-side frame timer
- no layout recalculation
- no GPU submissions beyond compositor necessity

### 11.2 Cached primitives

Cache only small, bounded assets:

- state glyph geometry
- common DirectWrite layouts for short labels where safe
- D2D brushes/materials
- compact path geometry

Invalidate caches on DPI, theme, font, or state-layout change. No unbounded text layout cache.

### 11.3 Surface limits

- one shared D3D11 device per Island process
- no per-task full render surface
- no large offscreen charts in P0
- Focus Card detail uses clipped/virtualized rows

---

## 12. Accessibility

Pulse must be usable without interpreting color or motion.

### 12.1 Accessible information

Expose accessible names/roles for:

- primary state
- primary subject
- reason
- active task count
- low-fuel condition
- Peek rows
- Focus Card route action

### 12.2 Keyboard

- global shortcut opens palette
- Escape collapses opened surfaces
- arrow navigation and Enter operate Peek/Palette items
- focus order is deterministic

### 12.3 High contrast and scaling

- detect Windows high-contrast setting
- use system-respecting contrast tokens
- support text scaling and per-monitor DPI
- never rely on blur alone for separation

---

## 13. Immersive-state behavior

When fullscreen, presentation, screen sharing, gaming, or Focus mode policy says hide:

- compact island hides
- nonessential animation stops
- Peek/Focus Card dismisses without altering task state
- no automatic route activation occurs
- user can still invoke Palette if permitted by their settings

When returning from immersive state, Arbitration may provide one concise summary. The UI does not replay every missed event.

---

## 14. UI acceptance tests

1. Compact island states are understandable in a one-second visual scan.
2. A task error, user-needed state, and fuel-low state have distinct non-color labels in Peek/Focus.
3. Primary island never timer-rotates between tasks.
4. Opened UI surfaces never parse provider data themselves.
5. No island focus theft occurs from passive state changes.
6. Idle/static state produces no continuous app-side redraw loop.
7. Reduced Motion removes pulsing while retaining meaning.
8. High-DPI and multi-monitor transitions preserve layout and hit regions.
9. Every route action label matches Context Router strength.
10. Compact island remains usable within target width without clipping core state.

---

## 15. Design invariants

1. Island renders a decision, not a dashboard.
2. Signal is primary; Fuel and resources are secondary.
3. Motion is information, never decoration for its own sake.
4. UI surfaces cannot upgrade task truth or route certainty.
5. Idle visual cost approaches zero.
6. Accessibility is not an optional alternate design.
