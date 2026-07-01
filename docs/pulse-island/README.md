# Pulse Island · Design Map

**Status:** Living product and implementation design package  
**Platform:** Windows 11, native Rust  
**Product posture:** Observe first. Control only when formally supported and explicitly enabled.  
**Consistency baseline:** `25-consistency-closure.md` is the normative correction layer for all earlier documents.

---

## What this product is

Pulse Island is a small desktop attention layer for agentic coding work. It tells a developer:

1. What agent work is active when a trustworthy source proves it.
2. Whether anything needs attention.
3. Whether trusted usage or system pressure may affect the work.
4. Where to return for the strongest verified original context.

It is not a replacement IDE, terminal, transcript viewer, agent orchestrator, or permission manager.

---

## Read this first

Before using any earlier document as an implementation source, read:

1. `25-consistency-closure.md` for cross-document decisions and terminology.
2. `24-implementation-work-packages.md` for the canonical implementation sequence.
3. `14-spike-c-link-transport-drop-mode.md` for concrete Link, Shim, local IPC, breadcrumb, and Drop Mode behavior.
4. `15-provider-capability-probe.md` plus the provider-specific Probe Card before making any provider capability claim.

---

## Design reading order

### Product truth and boundaries

| File | Role |
|---|---|
| `00-product-foundation.md` | Product definition, UI layers, primary promises, non-goals. Read with `25`. |
| `01-privacy-data-boundaries.md` | Local-first data limits, redaction, IPC and storage boundaries. Read with `25` retention precedence. |
| `02-agent-state-model.md` | Unified lifecycle, health, context, identity, and capability state. |
| `03-event-reduction-engine.md` | How raw signals become bounded trusted state. |
| `04-multi-agent-arbitration.md` | How one primary island narrative is selected. The tier order is governed by `25`. |
| `05-context-routing.md` | Accurate return-to-context rules and fallback chain. Route labels are governed by `25` and `23`. |
| `25-consistency-closure.md` | Cross-document authority map, capability taxonomy, corrected priorities, retention, Safe Mode, and execution order. |

### Runtime and integrations

| File | Role |
|---|---|
| `06-pulse-link-runtime-architecture.md` | Link architecture overview. `14` is authoritative for executable runtime/IPC detail. |
| `07-adapter-ecosystem.md` | Adapter capability model and provider support ladder. Provider claims remain Probe-gated. |
| `08-integration-hook-protocol.md` | Hook/Shim/Link fail-open contract. |
| `14-spike-c-link-transport-drop-mode.md` | Normative Link wake-up, pipe, breadcrumb, and Drop Mode proof. |
| `15-provider-capability-probe.md` | Standard evidence and release gate for every provider. |
| `23-windows-observation-and-window-binding.md` | Passive observation, process identity, honest window/terminal route evidence. |

### Native experience and trust

| File | Role |
|---|---|
| `09-native-island-ui-system.md` | Signal, Peek, Focus Card, Palette, rendering, accessibility. |
| `20-onboarding-settings-and-capability-disclosure.md` | First run, settings, repair flow, plain-language capability disclosure. |
| `21-install-update-uninstall-contract.md` | Per-user install, safe update, repair, rollback, and precise removal. |
| `22-reliability-recovery-and-diagnostics.md` | Runtime degradation, Safe Mode, diagnostics, crash and resource recovery. |

### Execution and quality gates

| File | Role |
|---|---|
| `10-verification-gates-and-mvp-roadmap.md` | Gate A–H roadmap and release vocabulary. Canonical work order is `24`. |
| `11-rust-workspace-architecture.md` | Crate boundaries, dependency rules, test topology. |
| `12-spike-a-native-signal-benchmark.md` | Native UI resource and behavior proof. |
| `13-spike-b-state-kernel.md` | Deterministic truth-fixture proof. |
| `18-first-adapter-selection-and-bootstrap.md` | Shared bootstrap and evidence-led first adapter selection. |
| `24-implementation-work-packages.md` | Canonical W0–W6 implementation packages, review gates, active code boundary. |

### Provider probe cards

| File | Current posture |
|---|---|
| `16-codex-cli-probe-card.md` | Hook-first observation candidate; task tokens unavailable until a separate formal source is proven; App Server/Fuel track is independent. |
| `17-claude-code-probe-card.md` | Hook-first observation candidate; native permissions remain native; account quota and current-session tokens are unavailable in P0. |
| `19-antigravity-probe-card.md` | Passive/Observed only until a formal official integration surface is proven. |

---

## Hard product invariants

- Pulse never stores prompts, transcripts, diffs, terminal output, raw tool input/output, or secrets by default.
- Pulse does not replace CLI commands, mutate shell PATH, or create a permanent background Link service.
- Link failure reduces observation only. It must never alter Agent behavior.
- `Observed`, `Attached`, `Degraded`, and `Context-ready` are different truths, not marketing labels.
- Provider release status, task health, route capability, and individual feature capability are independent axes.
- Exact context route claims require Exact evidence. A related window is not automatically the original task.
- Fuel never rewrites task traffic-light state unless a verified limit actually blocks work.
- Provider support is capability-by-capability and probe-gated.
- Process-only observation cannot render running, waiting, completed, failed, or Fuel state.
- Privacy profile is a retention ceiling, including terminal-state recovery.
- Safe Mode is enforced at Shim ingress and prevents Link wake while preserving provider-native behavior.

---

## Canonical execution order

```text
W0 Workspace Foundation
→ W1 State Truth Kernel
→ W2 Native Signal Shell
→ W3 Link / Shim / Drop Mode
→ W4 Codex + Claude provider probe race
→ W5 First narrow supported Observe adapter
→ W6 independent Fuel and context enhancements
```

W2 may begin after W0 contracts and the mock `PresentationPlan` seam are stable. W3 begins only after W1 truth fixtures pass.

The local implementation handoff lives in `.ai-bridge/current-plan.md`.

---

## How to update this package

A material implementation or product decision must update:

1. The most specific design document.
2. `25-consistency-closure.md` when it changes a cross-document rule.
3. This map when status or reading order changes.
4. The relevant provider probe card if it changes a capability claim.
5. The implementation handoff when it changes the active work boundary.

Do not add a feature by changing UI copy alone. First define its evidence source, privacy boundary, lifecycle effect, route/capability ceiling, failure behavior, and acceptance test.
