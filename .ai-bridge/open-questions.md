# Open Questions

## Q-001: Codex CLI Hook schema stability
**Status:** OPEN
**Question:** Codex CLI Hook event JSON schema (UserPromptSubmit, etc.) is based on observed behavior in version 0.142.4. Is the schema documented/stable across versions?
**Impact:** W4 direct probe must verify minimum 2 versions produce identical event shapes.
**Owner:** Probe evidence collection

## Q-002: Installer technology choice
**Status:** OPEN
**Question:** Should the 1.0 installer use MSI, Squirrel, a custom PowerShell script, or Windows App SDK MSIX?
**Impact:** Phase 5 packaging design. PowerShell script is simplest for MVP but lacks auto-update.
**Owner:** Phase 5 planning

## Q-003: DPI-aware rendering in Direct2D
**Status:** OPEN
**Question:** Direct2D/DirectComposition rendering has not been wired yet. Will the compositor handle per-monitor DPI natively, or does Pulse need manual DPI scaling in the render pipeline?
**Impact:** Phase 4 native UI rendering. pulse-win32 already has pure DPI scaling primitives.
**Owner:** Phase 4 implementation

## Q-004: serde_json in pulse-link-shim
**Status:** OPEN
**Question:** `pulse-link-shim` uses `serde_json` for Hook payload parsing. This bypasses `pulse-protocol`'s `preflight_frame()` check. Is the dual-parsing path acceptable, or should shim preflight through protocol first?
**Impact:** Privacy boundary enforcement. Currently the shim does its own validation; preflight would add defense-in-depth.
**Owner:** W3 audit or W5 adapter contract