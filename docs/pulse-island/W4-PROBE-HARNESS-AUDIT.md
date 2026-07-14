# Pulse Island · W4 Provider Probe Harness Audit

**Status:** Active — Codex CLI direct probe in progress  
**Last updated:** 2026-07-14  
**Authorization:** .ai-bridge/w4-authorized present — live probe authorized

## W4 Direct Probe Evidence (2026-07-14)

Codex CLI 0.144.1 real probe executed. Evidence captured in docs/pulse-island/evidence/2026-07-14-codex-direct-probe.md.

### Confirmed facts

- Codex --json outputs JSONL events: 	hread.started, 	urn.started, item.completed (agent_message/command_execution/error), 	urn.completed (usage)
- codex hooks subcommand does NOT exist in 0.144.1 — hook integration via --dangerously-bypass-hook-trust + config TOML
- Single-turn lifecycle mapping: 	hread.started→SessionStarted, 	urn.completed→Completed+usage
- Multi-turn, PermissionRequest, Late Attach, install/rollback still pending

### W4 completion gate

| Gate | Status |
|---|---|
| P0 Official-surface inventory | ✅ |
| P1 Passive process discovery | ✅ |
| P2 Install/rollback | ❌ |
| P3 Lifecycle mapping | ⚠️ partial |
| P4 Late Attach | ❌ |
| P5 Context route | ❌ |
| P6 Fault/privacy | ⚠️ partial |
| P7 Resource budget | ❌ |

2/8 complete, 2/8 partial. W5 NOT yet authorized.

## Previous scaffold (regression evidence)

W4 read-only scaffold commands provide regression evidence for the probe harness structure. These are regression assets, not active work queues.

## Scope Guard (unchanged)

- No live provider Hook install without W4 complete gate pass
- No provider config mutation without explicit authorization
- No provider support claim without direct evidence