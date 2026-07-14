# Pulse Island 1.0 Delivery Plan — Active Boundary

Updated: 2026-07-14T14:45:00+08:00
Workspace: D:\Workspace\pulse-island
Target agent: Codex (multi-agent cluster mode)
Status: **W4 AUTHORIZED** — Codex CLI direct probe approved. Proceed.

## Current active boundary

W4 Provider Probe Harness, now in **authorized live probe mode**. W0-W3 have gate evidence. W4 scaffold is code-complete; the next step is **authorized Codex CLI direct probe execution** (P0-P4).

All 13,221 lines of uncommitted code have been split into 6 reviewable commits (W1→W5) and committed. Working tree is clean.

## W4 authorization

.ai-bridge/w4-authorized exists → Codex CLI direct probe IS authorized. Do not output 
ot_authorized. Proceed to collect 7 direct gate evidence items per docs/pulse-island/W4-PROBE-HARNESS-AUDIT.md.

## W4 → W5 fast path

1. Install Codex Hook entry for pulse-link-shim.exe
2. Run real Codex CLI task → verify Shim events pass through
3. Verify lifecycle mapping (SessionStart → Running, PermissionRequest → Waiting, Stop → Terminal)
4. Verify Late Attach (close Island, run task, re-open → degraded → fresh evidence)
5. Verify rollback (uninstall Hook → config byte-for-byte preserved)
6. Collect resource measurements
7. Run provider-w4-completion-gate → if all 7 gates pass → w4_complete=true → W5 authorized

## Immediate execution pointer

DO NOT build more W4 scaffold commands. DO NOT output 
ot_authorized. Codex CLI probe is AUTHORIZED. Collect real provider evidence now.