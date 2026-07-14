# Decisions

## D-001: W4 authorization model
**Date:** 2026-07-14
**Decision:** W4 provider probe is authorized through a `.ai-bridge/w4-authorized` marker file. Agent must check this file before executing live provider actions; if absent, output `not_authorized` and stop.
**Rationale:** Previous default-deny without exit path created a self-locking deadlock (auth preflight → not_authorized → no evidence → W4 incomplete → auth still denied). The marker file gives humans a single explicit action to unlock the gate.

## D-002: Single-provider 1.0
**Date:** 2026-07-01 (per 07-10 delivery plan)
**Decision:** 1.0 ships with one selected provider (Codex CLI first probe lane). Provider parity is not a launch condition.
**Rationale:** Narrow scope forces evidence-backed claims. A second provider is a separate W4 probe race after 1.0.

## D-003: ControlAction empty enum at P0
**Date:** 2026-07-07
**Decision:** `ControlAction` is an empty enum in W2-W4. P0 products must not produce any control actions. The type exists as a forward-compatible seam for W5-W6 provider control features.
**Rationale:** Observe-first posture. No control until `supported_control` release label is earned through provider-specific probe evidence.

## D-004: unsafe isolation in pulse-win32-hwnd and pulse-win32-link
**Date:** 2026-07-07
**Decision:** Workspace-level `unsafe_code = "forbid"` applies to all core crates. Only `pulse-win32-hwnd` and `pulse-win32-link` override to `allow` with `unsafe_op_in_unsafe_fn = "deny"`. All other crates remain zero-unsafe.
**Rationale:** Win32 platform FFI is inherently unsafe. The two crates explicitly scope the unsafe boundary. Production apps (pulse-island) call through safe wrappers via MSVC cfg gates.