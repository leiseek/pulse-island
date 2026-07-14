# Pulse Island

Pulse Island is a Windows-first, observe-only task signal prototype. The current tree contains the bounded truth kernel, persistence, Link/Shim native transport paths, late Island snapshot handshake, privacy/fail-open contracts, and a read-only provider capability probe.

## Current delivery boundary

- Codex CLI and Claude Code are detected through read-only local probes; no provider is release-selected yet.
- Codex Hook-shaped JSON is sanitized to session identity and evidence category, then delivered through Shim → Link with bounded named-pipe frames.
- Link persists bounded breadcrumbs and serves late Island snapshot/handshake state. The CLI spike client is a transport harness, not the final desktop shell.
- Provider configuration, credentials, transcripts, prompts, and raw command output are never retained by the probe harness.

The 1.0 release remains gated on direct provider lifecycle evidence, a selected observe-only adapter, a production Island host, and clean-machine packaging. See [the delivery plan](docs/plans/2026-07-10-v1-delivery-plan.md) for the active sequence.

Current release notes are in [CHANGELOG.md](CHANGELOG.md).

## Verification

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## MVP host smoke check

The production Island host now has a content-free snapshot/status path that can
be used before the native window is attached:

```text
cargo run -p pulse-island -- --diagnostic
cargo run -p pulse-island -- --snapshot --state <path-to-breadcrumbs.snapshot>
cargo run -p pulse-island -- --native-smoke
```

Without `--state`, the host uses `PULSE_LINK_STATE_ROOT`, then the per-user
`LOCALAPPDATA\PulseIsland` state root. Missing state is reported as `empty`;
malformed state is reported as `unavailable` and never causes provider data to
be printed.

`--native-smoke` creates the compact non-activating HWND, applies the bounded
native adapter plan, drains a nonblocking message budget, and destroys the
window. It runs as a real smoke check on the MSVC Windows build; other targets
report that the native check is unavailable.
