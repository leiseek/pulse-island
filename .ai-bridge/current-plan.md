# Pulse Island W0/W1 Foundation

Updated: 2026-07-01T07:28:03.891Z
Workspace: D:\Workspace\myldd
Target agent: Codex (codex)

## Plan

Implement only W0 plus the first slice of W1 from `docs/pulse-island/24-implementation-work-packages.md`.

Read first, in this order:
1. `docs/pulse-island/25-consistency-closure.md`
2. `docs/pulse-island/24-implementation-work-packages.md`
3. `docs/pulse-island/11-rust-workspace-architecture.md`
4. `docs/pulse-island/13-spike-b-state-kernel.md`
5. `docs/pulse-island/01-privacy-data-boundaries.md`

Scope:
- Create the Rust workspace and provider-neutral core crates only: `pulse-domain`, `pulse-protocol`, `pulse-testkit`, and the smallest `pulse-reducer`, `pulse-routing`, `pulse-fuel`, `pulse-arbitration` slice required for deterministic fixtures.
- Implement bounded domain/protocol types for separate axes: provider release status, task health, route capability, feature capability, lifecycle, attention, and safe summary/error categories.
- Implement admission before mutation: byte-length validation before allocation, forbidden field rejection, no arbitrary payload map.
- Implement deterministic reducer/arbitration fixtures for: basic lifecycle; waiting truthfulness; terminal protection; PID identity; Fuel separation; canonical failed > waiting > limited ordering; Exact vs Strong route labels; process-only cannot become running/waiting/terminal/Fuel-aware; Strict terminal retention; malformed/oversized input.
- Add a fixed clock and test helpers.

Hard constraints:
- No provider adapter crates, provider configuration changes, live Hooks, Codex/Claude/Antigravity file reads, network, SQLite, Win32 UI, named pipes, D3D, or browser/UI runtime.
- Do not implement a generic file scanner, transcript/session JSONL parser, command-line/environment collector, token estimator, or UI control system.
- Safe task titles are unavailable by default. Use generic provider/workspace labels until a provider Probe Card explicitly permits a source.
- Treat `25-consistency-closure.md` as normative whenever earlier files differ.
- Do not create a generic utils crate.

Acceptance:
- `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` pass without external providers or network.
- Same fixtures plus fixed clock yield identical task snapshots and presentation plans.
- No raw task content is stored, logged, or included in test fixtures.
- Include the new required cross-layer tests from section 13 of `25-consistency-closure.md` that fall within W1.

Do not begin W2/W3 or change any provider configuration.

## Implementation contract

- Work from this plan in small, reviewable steps.
- Keep edits scoped to the requested task and existing project conventions.
- Run focused verification before handing work back.
- Update .ai-bridge/agent-status.md with files touched, checks run, results, blockers, and review notes.
- Save the final review diff to .ai-bridge/implementation-diff.patch when practical.
- Append notable execution events to .ai-bridge/execution-log.jsonl when the implementation agent supports logging.
