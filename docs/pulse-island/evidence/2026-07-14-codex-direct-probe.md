# Codex CLI Direct Probe Evidence

**Date:** 2026-07-14
**Codex version:** 0.144.1
**Probe method:** Read-only codex exec --ephemeral --json non-interactive smoke
**Authorization:** .ai-bridge/w4-authorized marker present

## P0: Official-surface inventory

- ✅ codex --version → codex-cli 0.144.1
- ✅ codex exec --help → full CLI surface documented
- ✅ codex doctor → connectivity/diagnostics surface confirmed
- ✅ codex features → feature flag inspection available
- ✅ codex app-server → experimental app-server surface present (not probed)
- ❌ codex hooks subcommand → **does not exist in 0.144.1**. Hook integration must use --dangerously-bypass-hook-trust flag or Codex config hooks settings.

## P1: Passive process discovery

- ✅ codex exec --ephemeral runs as a child process, exits on completion
- ✅ Process exit code 0 on successful turn
- ⚠️ Model refresh errors (codex_models_manager) logged during startup — benign for probe, may affect production reliability

## P2: Integration install/rollback — PENDING

Hook integration path: Codex hooks are configured via ~/.codex/config.toml or codex exec --dangerously-bypass-hook-trust. The exact hook configuration schema needs to be confirmed against current docs. Install/rollback test deferred until hook config is exercised on a clean test workspace.

## P3: Lifecycle semantics — PARTIAL EVIDENCE

Observed JSONL event types for a single-turn codex exec:

| Event type | JSONL shape | Pulse mapping |
|---|---|---|
| 	hread.started | {"type":"thread.started","thread_id":"..."} | → SessionStarted |
| 	urn.started | {"type":"turn.started"} | → Activity |
| item.completed (error) | {"type":"item.completed","item":{"id":"...","type":"error","message":"..."}} | → Degraded/warning |
| item.completed (agent_message) | {"type":"item.completed","item":{"id":"...","type":"agent_message","text":"..."}} | → Response/completion |
| item.completed (command_execution) | {"type":"item.completed","item":{"id":"...","type":"command_execution","command":"...","exit_code":0,"status":"completed"}} | → Tool use activity |
| 	urn.completed | {"type":"turn.completed","usage":{"input_tokens":...,"output_tokens":...,"cached_input_tokens":...,"reasoning_output_tokens":...}} | → Completed + usage evidence |

Multi-turn behavior and PermissionRequest detection remain TBD.

## P4: Late Attach — PENDING

Requires: Island + Link + Shim binaries working in production path with a real Hook entry. Deferred to Phase 1+2 of delivery plan.

## P5: Context routing — PENDING

Deferred to provider adapter implementation (Phase 3).

## P6: Fault/privacy behavior — PENDING

- ✅ Verified codex exec --json output contains no raw credentials
- ⚠️ codex doctor output contains redacted endpoint credentials — Pulse must not capture doctor output
- Pending: Shim process contract validation with real hook events

## P7: Resource measurements — PENDING

Deferred to Phase 3 adapter implementation with real workload.

## W4 completion gate status

| Gate | Status |
|---|---|
| Official-surface inventory | ✅ DONE |
| Passive process discovery | ✅ DONE |
| Install/rollback real fixture | ❌ PENDING |
| Lifecycle mapping | ⚠️ PARTIAL (single-turn evidence) |
| Late Attach | ❌ PENDING |
| Context route | ❌ PENDING |
| Fault/privacy | ⚠️ PARTIAL |
| Resource budget | ❌ PENDING |

**W4 completion: 2/8 gates have evidence, 2/8 have partial evidence. W4→W5 NOT yet authorized.**

## Next probe steps

1. Test multi-turn codex exec with follow-up prompts to verify multi-turn lifecycle mapping
2. Exercise Codex hook config (~/.codex/config.toml) to test install/rollback in a clean workspace
3. Wire pulse-link-shim.exe as a Codex hook target and verify stdin handoff delivery
4. Complete Late Attach scenario once Link/Shim/Island production path exists