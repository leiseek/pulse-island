# Changelog

## 1.0.0-rc1 — in progress

### 2026-07-14: Cluster audit + recovery sprint

- **8 new commits** split single 13K-line mega-patch into reviewable units
- W1 state kernel (domain/protocol/reducer/arbitration/routing/fuel) committed
- W2 native signal shell (UI models, Win32 primitives, HWND backend) committed
- W3 link transport (Shim/Link IPC, persistence, native pipes, Drop Mode) committed
- W4 gate audits + consistency closure docs committed
- Production Island host + packaging manifest committed
- .ai-bridge state refreshed: W4 authorized, decisions/open-questions filled
- **Codex CLI 0.144.1 direct probe executed**: confirmed JSONL event schema (thread.started/turn.started/item.completed/turn.completed), lifecycle mapping partial
- codex hooks subcommand does NOT exist in 0.144.1 — hook integration must use config TOML path
- W4 completion: 2/8 gates complete, 2/8 partial. W5 NOT yet authorized.

### Added

- Bounded Codex Hook sanitization and fail-open Shim process path.
- Persistent Link breadcrumbs, ingress payload framing, and late Island snapshot handshake.
- Production Island transport entry point and per-user package manifest/scripts.
- Isolated Codex CLI surface, read-only execution, and Hook schema probes.

### Verification

- Workspace tests and Clippy pass.
- Release staging build, strict binary manifest validation, and disposable uninstall pass.
- Cluster audit: 3-agent independent audit confirming architecture 8/10, code quality green, W4 scaffold bloat identified.

### Not yet released

Codex Hook install/rollback, lifecycle mapping (multi-turn), Late Attach evidence, native UI rendering, and clean-account installer validation remain open. See docs/pulse-island/27-open-gates.md.