# Pulse Island 1.0 Release Gate

Status: **not ready** (active W4 direct-evidence work)

Operational follow-up is tracked in [27-open-gates.md](27-open-gates.md).

This gate prevents the repository from being described as a shippable 1.0 before the product boundary is actually demonstrated.

## Required evidence

| Gate | Current state | Required proof |
| --- | --- | --- |
| Workspace truth kernel | pass | `cargo test --workspace` |
| Link/Shim bounded ingress | pass | native and process contract suites |
| Persistent late attach | pass | breadcrumb recovery and Island handshake smoke |
| Provider official surface | observed | sanitized Codex/Claude CLI surface probe |
| Provider lifecycle/install rollback | missing | authorized direct fixture |
| Selected observe adapter | missing | W4 `supported_observe` evidence packet |
| Production Island host transport entry | pass | `apps/pulse-island` diagnostic and native connect-sequence entry |
| Production Island native window backend | pass (smoke) | MSVC `--native-smoke` creates non-activating HWND, applies adapter commands, pumps messages, and destroys it |
| Production Island native UI/rendering | missing | installed app renders Signal/Peek/Focus against Link snapshot/delta |
| Per-user packaging/uninstall | missing | clean Windows account install/rollback run |

## Release rule

Only a provider with a complete, redacted direct-evidence packet may be selected. Until then, all provider labels remain `not_probed`; the CLI spike and synthetic fixtures cannot elevate release status.

## Regression command set

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p pulse-island-spike -- --provider-w4-completion-gate
```
