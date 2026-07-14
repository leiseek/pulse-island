# Pulse Island 1.0 Delivery Plan

**Date:** 2026-07-10  
**Status:** Proposed execution plan  
**Goal:** Ship a Windows 11 1.0 with one genuinely supported Observe-first provider, rather than three simulated integrations.

## Delivery decision

1.0 is a narrow product release: install Pulse Island per-user; explicitly enable one provider integration; start the provider normally; open Pulse later; see only evidence-backed task state and a safe context return action; disable or uninstall without affecting the provider.

The candidate race is limited to Codex CLI and Claude Code. Codex CLI is the first probe lane because it is present in the local environment, but it is **not selected** until direct evidence passes. Antigravity is excluded from 1.0 because it is absent locally and has no direct probe evidence.

The release excludes multi-provider parity, task control, approval UI, transcript/history parsing, exact route claims without proof, Fuel without a separately validated source, cloud sync, and automatic updates.

## Current baseline

- W0-W3 have accepted gate audits. They are regression suites, not active feature queues.
- W4 is code-complete only as a read-only evidence harness. All providers are `not_probed`.
- W4/W5 preflight reports 21 missing direct gates: 7 for each provider.
- `cargo test --workspace` passed on 2026-07-10.
- The working tree contains a large uncommitted W1-W4 implementation batch; it must be reviewed and committed in small, verified units before release work proceeds.
- The W3 shipping entry points are now wired to bounded Shim → Link ingress, persistent handoff, and late Island attach paths. The remaining W4 boundary is direct provider lifecycle evidence; no provider configuration is mutated by the runtime.

## Critical-path sequence

### Phase 0: Recover a trustworthy baseline (1 day)

**Files:** `Cargo.toml`, `.ai-bridge/current-plan.md`, `docs/pulse-island/W1-GATE-AUDIT.md`, `docs/pulse-island/W2-GATE-AUDIT.md`, `docs/pulse-island/W3-GATE-AUDIT.md`, `docs/pulse-island/W4-PROBE-HARNESS-AUDIT.md`

1. Split the existing uncommitted work into reviewable commits: W1 truth kernel, W2 UI shell, W3 Link transport, then W4 harness/docs.
2. Run format, lint, workspace tests, and relevant MSVC-only Windows harnesses for each commit.
3. Replace the stale root README statement (“W0 plus first W1 slice only”) with the accepted W0-W4-harness status after the commits are verified.

**Exit:** clean or intentionally staged working tree; reproducible baseline commands recorded; docs describe the same boundary as code.

### Phase 1: Make the Link and Shim binaries executable (3-5 days)

**Files:** `apps/pulse-link-shim/src/main.rs`, `apps/pulse-link/src/main.rs`, `apps/pulse-link-shim/tests/`, `apps/pulse-link/tests/`, `crates/pulse-link-core/`, `crates/pulse-win32-link/`

1. Add failing process-level contract tests proving that Shim reads one bounded stdin envelope, invokes the native ingress path, and returns fail-open exit code `0` on both delivery and rejection.
2. Wire `pulse-link-shim.exe` to the existing preflight, current-user transport names, bounded timeout, existing-Link delivery, and inherited-handoff wake path; never write stdin to disk or output it.
3. Add failing process-level Link tests for wake startup, one ingress acknowledgement, reducer/checkpoint progression, Island snapshot serving, grace exit, and native-handle cleanup.
4. Wire `pulse-link.exe` to the existing runtime/transport seams, with bounded process lifetime and no synthetic status output in normal Hook mode.
5. Run the existing MSVC C0-C9 harnesses plus new real-process smoke tests. Do not call W3 production-ready until the executable paths, rather than library seams alone, pass.

**Exit:** a synthetic sanitized envelope can traverse the actual Shim and Link binaries, produce a bounded breadcrumb/snapshot, and leave no process or handle residue; all failure paths remain fail-open.

### Phase 2: Time-boxed direct provider probe race (3-5 days)

**Files:** `docs/pulse-island/15-provider-capability-probe.md`, `docs/pulse-island/16-codex-cli-probe-card.md`, `docs/pulse-island/17-claude-code-probe-card.md`, `docs/pulse-island/W4-PROBE-HARNESS-AUDIT.md`, `apps/pulse-island-spike/src/lib.rs`

1. Obtain explicit authorization before touching provider configuration or running live provider tasks.
2. Run the same seven direct gates for Codex and Claude in a synthetic workspace: official-surface evidence, real install/update/uninstall, lifecycle mapping, Late Attach, context route, fault/privacy behavior, and resource measurements.
3. Redact outputs through the existing evidence bundle contract; retain no prompts, transcripts, provider config, credentials, terminal buffers, or private endpoint traffic.
4. Score only completed evidence. Select one provider only if it reaches `supported_observe`; otherwise stop and publish a no-go report.

**Exit:** one signed-off W4 evidence packet and `provider-w5-start-preflight` selects exactly one provider; or an explicit 1.0 no-go decision.

### Phase 3: Build the selected narrow adapter (4-6 days)

**Files:** `crates/pulse-<provider>-adapter/`, `apps/pulse-link/`, `apps/pulse-link-shim/`, `crates/pulse-domain/`, `crates/pulse-reducer/`, `crates/pulse-routing/`, `crates/pulse-testkit/tests/`

1. Start with failing adapter contract tests for supported ingress, identity, freshness, late attachment, degradation, and uninstall rollback.
2. Implement only evidence-approved fields and mappings. Unknown, stale, quiet, and malformed input must downgrade rather than invent task state.
3. Add real integration fixtures for Link wake, Island-late attach, restart recovery, provider failure, Safe Mode, and config rollback.
4. Keep context action at Workspace-ready unless the probe earned stronger routing.

**Exit:** selected provider has `supported_observe`; all unsupported capabilities remain visibly unavailable; Hook failures are fail-open.

### Phase 4: Make the actual desktop app usable (3-5 days)

**Files:** `apps/pulse-island/` (new production host), `crates/pulse-island-ui/`, `crates/pulse-win32/`, `crates/pulse-win32-hwnd/`, `apps/pulse-link-spike-client/`

1. Replace spike-only launch flow with the `apps/pulse-island` production host entry point connected to the Link snapshot protocol; continue native rendering work behind that boundary.
2. Complete the minimum native rendering path for Signal, Peek, Focus Card, accessibility labels, no-focus-theft behavior, and degradation/offline states.
3. Add end-to-end desktop tests for start-late, reconnect, focus behavior, DPI, immersive suppression, and a visible manual smoke checklist.
4. Measure Gate A UI limits and Gate C Drop Mode limits on the release machine; feature-gate any capability that misses its budget.

**Exit:** a user can run the installed Island against the selected adapter—not a CLI-only simulation—and all P0 UI states are visibly verified.

### Phase 5: Package, lifecycle, and release hardening (3-4 days)

**Files:** `packaging/pulse-island-1.0.manifest.json`, `packaging/README.md`, installer implementation (pending), `docs/pulse-island/20-onboarding-settings-and-capability-disclosure.md`, `docs/pulse-island/21-install-update-uninstall-contract.md`, `docs/pulse-island/22-reliability-recovery-and-diagnostics.md`, `README.md`

1. Create a per-user package containing Island, Link, Shim, version metadata, and uninstaller; no service, PATH change, or automatic provider integration.
2. Implement explicit onboarding/disclosure and enable/disable/repair flows for the selected provider.
3. Verify clean install, upgrade, interrupted install, repair, disable, uninstall, and rollback on a clean Windows account/VM.
4. Produce content-free diagnostics and a release checklist covering privacy, Safe Mode, crash recovery, resource limits, and provider non-interference.

**Exit:** clean-machine install-to-uninstall passes without provider damage; release notes state precise capability ceilings.

### Phase 6: 1.0 release candidate and launch (2 days)

**Files:** `README.md`, `CHANGELOG.md` (new), `docs/pulse-island/26-v1-release-gate.md` (new)

1. Freeze scope; accept only release-blocking fixes.
2. Run the full deterministic suite, Windows native suites, adapter real-fixture suite, performance measurements, and clean-machine smoke test.
3. Review all user-visible capability wording against evidence, then tag/package 1.0.

**Exit:** every 1.0 claim has a test or redacted direct-evidence artifact; unsupported features are absent or labelled unavailable.

## Work intentionally deferred after 1.0

1. A second provider adapter, selected by a new W4 evidence race.
2. Exact terminal/original-task routing, only with formal task linkage.
3. Fuel, quota, and burn meter, each with an independent source gate.
4. Toast policy, automatic updates, enterprise deployment, controls, cloud/team features.

## Operating rules that prevent another stall

1. Every phase has one owner, one exit artifact, and a maximum time box; a failed probe becomes a decision, not more scaffolding.
2. Do not add W4 diagnostics unless they remove a concrete missing direct gate.
3. Do not reopen W0-W3 without a fresh regression failure.
4. Commit each accepted gate independently; no multi-gate uncommitted batch.
5. Keep one source of truth for active boundary: this plan during 1.0, mirrored in `.ai-bridge/current-plan.md` and the documentation index.
6. A 1.0 release is allowed with one provider only. Provider parity is not a launch condition.

## 1.0 definition of done

- One selected provider reaches `supported_observe` through direct, redacted W4 evidence.
- Island, Link, and Shim work together in a clean per-user installation.
- Late attach, degradation, privacy, fail-open, enable/disable, and uninstall behavior pass real fixtures.
- Native UI and Link resource budgets are measured and pass, or the affected nonessential capability is disabled.
- The full test suite, Windows-native tests, and release smoke test are green from the release candidate.
