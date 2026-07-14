# Pulse Island · W4 Provider Probe Harness Audit

**Status:** In progress, read-only scaffold active  
**Scope:** Provider capability report scaffolding only  
**Last updated:** 2026-07-08

This audit tracks W4 implementation against `15-provider-capability-probe.md` and the provider-specific Probe Cards. W4 is the active implementation boundary.

## Current W4 Slice

Implemented:

- `pulse-island-spike --provider-probe-manifest` exports a machine-checkable W4 manifest.
- The manifest names Codex CLI, Claude Code, and Antigravity as `not_probed`.
- The manifest requires version, environment category, integration mode, capability matrix, known limitations, resource figures, and release recommendation fields.
- The manifest explicitly forbids live Hook installation, provider configuration mutation, provider adapter creation, network/App Server queries, transcript/session file parsing, and production route activation.
- `pulse-island-spike --provider-probe-report=<provider>` exports a sanitized per-provider read-only inventory report.
- The initial report records `tested_version=not_collected`, `environment_category=not_collected`, `integration_mode=read_only_inventory`, and `release_recommendation=not_probed`.
- The initial capability matrix records all capabilities as `not_probed`.
- Reports record `raw_provider_content=false` and `raw_provider_configuration=false`.
- `pulse-island-spike --provider-config-transaction-fixture=<provider>` exports a synthetic user-config transaction fixture without reading or writing real provider configuration.
- The synthetic transaction fixture proves install/update/uninstall, unrelated entry preservation, ordering preservation, Pulse-signature-only targeting, and interrupted-install rollback reporting.
- `pulse-island-spike --provider-probe-scorecard` exports the first-adapter scorecard scaffold.
- With no live probe evidence, the scorecard reports `selection_status=no_adapter_selected`, `first_adapter_candidate=none`, all providers at `total_score=0`, and all providers at `not_probed`.
- `pulse-island-spike --provider-resource-fixture=<provider>` exports synthetic resource measurement categories for Drop Mode memory/CPU, active Link memory/CPU, event-to-snapshot latency, adapter event rate, breadcrumb size, and Link exit behavior.
- The resource fixture is category-only and reports `resource_budget_claim=not_measured`; it does not claim budget pass/fail.
- `pulse-island-spike --provider-evidence-register=<provider>` exports a sanitized evidence register summary.
- The sanitized evidence register records official-surface inventory and synthetic-fixture categories without raw source locations, raw provider content, raw provider configuration, or enabled capability claims.
- `pulse-island-spike --provider-official-evidence-source-locator=<provider>` exports the official source locator scaffold required by `15-provider-capability-probe.md`. It names the required metadata fields for source type, redacted source location ID, publication/update date, tested provider version, supported capability claim, and known constraints while retaining no raw source locations or raw documentation.
- `pulse-island-spike --provider-probe-summary-fixture=<provider>` exports sanitized probe-result summaries that can mark individual capability rows as `probed_candidate` or `not_probed` without retaining raw payloads.
- `pulse-island-spike --provider-probe-scorecard=sanitized-fixture` scores sanitized summaries while still reporting `selection_status=no_adapter_selected`, `first_adapter_candidate=none`, and `w5_adapter_creation_authorized=false`.
- `pulse-island-spike --provider-hard-disqualifiers=sanitized-fixture` evaluates W4 hard gates and reports W5 blocked when install/rollback, Late Attach, terminal truth, or resource budget evidence is missing.
- `pulse-island-spike --provider-evidence-gap-summary` aggregates direct evidence gaps across Codex CLI, Claude Code, and Antigravity. It reports 21 missing direct gates in total, keeps W4 incomplete, keeps W5 blocked, and does not execute live provider actions.
- `pulse-island-spike --provider-probe-audit` exports an aggregate W4 status covering manifest, reports, synthetic config transactions, resource fixtures, evidence registers, official evidence source locators, probe summaries, scorecard, hard disqualifier state, read-only local probe-run presence, read-only local probe summary presence, read-only resource measurement plan presence, Probe Card execution plan presence, evidence retention policy presence, authorization preflight blocking, missing-capability rationale presence, release decision log presence, direct gate packet presence, direct evidence import checklist presence, authorized evidence runbook presence, sanitized evidence output template presence, sanitized evidence bundle validator presence, release elevation preflight blocking, W5 observe adapter contract scaffold presence, and W4 completion-gate blocking.
- `pulse-island-spike --provider-surface-inventory` exports the provider Probe Card declared surface inventory for Codex CLI, Claude Code, and Antigravity.
- `pulse-island-spike --provider-probe-readiness` exports W4 readiness and remaining live gates: scaffold is ready, Probe Card execution plans, evidence retention policy, authorization preflight, missing-capability rationale, release decision logs, evidence registers, official evidence source locators, direct gate packets, direct evidence import checklists, authorized evidence runbooks, sanitized evidence output templates, sanitized evidence bundle validators, release elevation preflight, W5 observe adapter contract scaffold, and completion gate are ready, live probe is not complete, W5 remains not ready, and next allowed work is direct gate evidence collection when authorized.
- A read-only local provider CLI preflight was run on 2026-07-10: Codex CLI and Claude Code were observed through their Windows command shims (raw versions are intentionally not retained), and `antigravity` was not found on PATH. The probe now resolves `.cmd`/`.exe` command wrappers instead of misclassifying installed Windows CLIs as absent.
- The same read-only run confirmed Codex `exec --help` and `app-server --help` surfaces are callable without starting a provider task, reading configuration, or enabling a capability claim; these are P0/P1 surface evidence only.
- An authorized Codex read-only non-interactive smoke completed one synthetic turn on 2026-07-10. Only sanitized process/turn/response categories were retained; this is execution evidence, not Hook or lifecycle support evidence, so release status remains `not_probed`.
- The Shim process contract now exercises the documented Codex event spellings, including `UserPromptSubmit`, and verifies successful fail-open handling without echoing session identity.
- `pulse-island-spike --provider-local-environment-manifest=read-only-fixture` exports a sanitized local environment manifest that records provider command presence/version categories without retaining command paths, provider config, account data, or support claims.
- `pulse-island-spike --provider-live-probe-dry-run=read-only-fixture` exports the next read-only probe actions without executing provider tasks.
- `pulse-island-spike --provider-live-probe-run=read-only-local` executes the approved local version-category checks and exports only sanitized categories. It does not retain raw versions, command paths, provider configuration, account data, transcript/session data, or support claims.
- `pulse-island-spike --provider-live-probe-summary=read-only-local` converts the read-only local probe run into sanitized evidence summaries without capability elevation.
- `pulse-island-spike --provider-resource-measurement-plan=read-only-local` exports the read-only resource measurement plan without executing measurements or starting provider tasks.
- `pulse-island-spike --provider-probe-card-execution-plan=<provider>` exports provider Probe Cards as phase-based execution plans without running provider tasks.
- `pulse-island-spike --provider-evidence-retention-policy` exports the W4 retention policy: sanitized reports/matrices/fixtures may enter the repo; raw prompts, transcripts, credentials, terminal buffers, private endpoint traffic, source code, and raw provider config may not.
- `pulse-island-spike --provider-live-authorization-preflight=<provider>` exports the live-action preflight and remains `not_authorized` by default.
- `pulse-island-spike --provider-missing-capability-rationale=<provider>` explains every absent W4 capability with a non-blank reason while keeping release labels at `not_probed`.
- `pulse-island-spike --provider-release-decision-log=<provider>` exports the sanitized W4 release decision log. Current outcome is `defer_provider_pending_direct_evidence`; W5 remains blocked.
- `pulse-island-spike --provider-capability-matrix=<provider>` exports the full `15-provider-capability-probe.md` capability matrix scaffold with all 15 capability rows populated as `not_probed`, no blank cells, no support labels, no winner, no raw provider content, and no raw provider configuration.
- `pulse-island-spike --provider-release-label-evaluation=sanitized-fixture` evaluates release-label eligibility from sanitized fixture state only. `process_observed`, `experimental_attached`, `supported_observe`, `supported_fuel`, and `supported_control` all remain ineligible until direct gates are present; W5 remains blocked.
- `pulse-island-spike --provider-probe-phase-status=<provider>` exports the P0-P8 probe phase status matrix. P0 official-surface inventory is scaffolded as read-only; P1-P8 remain not executed and require authorization. The output does not start provider tasks, read/write provider config, retain raw provider content, or enable capability claims.
- `pulse-island-spike --provider-w5-start-preflight` exports the W5 start decision preflight. Current output has `w4_complete=false`, `w5_start_allowed=false`, `selected_provider=none`, and blocks provider adapter creation until a provider reaches `supported_observe` with direct evidence.
- `pulse-island-spike --provider-direct-gate-packet=<provider>` exports a per-provider direct evidence packet covering official evidence, real install/update/uninstall fixture, live lifecycle mapping, Late Attach, context route, fault/privacy, and live resource gates. It does not execute provider actions.
- `pulse-island-spike --provider-direct-evidence-import-checklist=<provider>` exports the direct-evidence import checklist. It requires authorized local artifacts for each direct gate, rejects sanitized fixtures and read-only version evidence as release-elevating inputs, rejects raw provider content/configuration, and keeps W5 blocked.
- `pulse-island-spike --provider-authorized-evidence-runbook=<provider>` exports the authorized direct-evidence runbook scaffold. It defines manual authorized steps for synthetic workspace preparation, local-only provider config backup, Probe Card phase execution, direct gate artifact collection, required redaction, and release-elevation preflight while still executing no provider action and keeping release elevation plus W5 adapter creation blocked.
- `pulse-island-spike --provider-sanitized-evidence-output-template=<provider>` exports the sanitized evidence output template for authorized direct evidence after redaction. It declares the only repo-safe artifacts and keeps raw prompts/transcripts, raw provider configuration, customer source, and credentials out of the repository.
- `pulse-island-spike --provider-sanitized-evidence-bundle-validator=<provider>` exports the sanitized bundle validator scaffold. It requires the repo-safe report, capability matrix, sanitized event mapping fixtures, category test results, known limitations, and release decision artifacts, rejects raw prompts/transcripts, raw provider config, customer source, credentials, raw terminal buffers, and private endpoint traffic, and does not import artifacts or claim direct evidence.
- `pulse-island-spike --provider-release-elevation-preflight=<provider>` exports the per-provider release elevation preflight for `supported_observe`. Current output keeps the provider at `not_probed`, reports direct evidence missing and hard disqualifiers uncleared, and blocks release elevation plus W5 adapter creation.
- `pulse-island-spike --provider-w5-observe-adapter-contract=<provider>` exports the W5 observe adapter contract scaffold without creating an adapter. It requires a future `supported_observe` release label, enumerates the only observe capabilities a future adapter may use, excludes external session control, approval UI, transcript/history parsing, raw-prompt task titles, exact route claims without exact evidence, terminal claims without terminal evidence, and unscoped Fuel sources, and keeps `w5_adapter_creation_authorized=false`.
- `pulse-island-spike --provider-w4-completion-gate` exports the W4 completion/W5 start gate: W4 scaffold and direct gate packets are ready, but W4 remains incomplete and W5 remains blocked until direct evidence exists.

The local CLI preflight and read-only local probe run are environment evidence only. They do not prove Hook behavior, install/rollback safety, lifecycle mapping, Late Attach, context routing, resource budget, or provider support.

Not yet implemented in W4:

- Live provider probe execution.
- Live resource measurements.
- Release-label elevation from direct evidence. Sanitized-fixture evaluation is implemented only as a blocking gate.
- W5 adapter creation.

## Verification Commands

Latest verified W4 slice commands:

```text
cargo test -p pulse-island-spike w4_provider_probe
cargo test -p pulse-island-spike w4_
cargo run -p pulse-island-spike -- --gate-audit
cargo run -p pulse-island-spike -- --provider-probe-manifest
cargo run -p pulse-island-spike -- --provider-probe-report=codex_cli
cargo run -p pulse-island-spike -- --provider-config-transaction-fixture=claude_code
cargo run -p pulse-island-spike -- --provider-probe-scorecard
cargo run -p pulse-island-spike -- --provider-resource-fixture=codex_cli
cargo run -p pulse-island-spike -- --provider-evidence-register=codex_cli
cargo run -p pulse-island-spike -- --provider-official-evidence-source-locator=codex_cli
cargo run -p pulse-island-spike -- --provider-probe-summary-fixture=claude_code
cargo run -p pulse-island-spike -- --provider-probe-scorecard=sanitized-fixture
cargo run -p pulse-island-spike -- --provider-hard-disqualifiers=sanitized-fixture
cargo run -p pulse-island-spike -- --provider-evidence-gap-summary
cargo run -p pulse-island-spike -- --provider-probe-audit
cargo run -p pulse-island-spike -- --provider-surface-inventory
cargo run -p pulse-island-spike -- --provider-probe-readiness
cargo run -p pulse-island-spike -- --provider-local-environment-manifest=read-only-fixture
cargo run -p pulse-island-spike -- --provider-live-probe-dry-run=read-only-fixture
cargo run -p pulse-island-spike -- --provider-live-probe-run=read-only-local
cargo run -p pulse-island-spike -- --provider-live-probe-summary=read-only-local
cargo run -p pulse-island-spike -- --provider-resource-measurement-plan=read-only-local
cargo run -p pulse-island-spike -- --provider-probe-card-execution-plan=claude_code
cargo run -p pulse-island-spike -- --provider-evidence-retention-policy
cargo run -p pulse-island-spike -- --provider-live-authorization-preflight=codex_cli
cargo run -p pulse-island-spike -- --provider-missing-capability-rationale=claude_code
cargo run -p pulse-island-spike -- --provider-release-decision-log=codex_cli
cargo run -p pulse-island-spike -- --provider-capability-matrix=codex_cli
cargo run -p pulse-island-spike -- --provider-release-label-evaluation=sanitized-fixture
cargo run -p pulse-island-spike -- --provider-probe-phase-status=claude_code
cargo run -p pulse-island-spike -- --provider-w5-start-preflight
cargo run -p pulse-island-spike -- --provider-direct-gate-packet=codex_cli
cargo run -p pulse-island-spike -- --provider-direct-evidence-import-checklist=claude_code
cargo run -p pulse-island-spike -- --provider-authorized-evidence-runbook=codex_cli
cargo run -p pulse-island-spike -- --provider-sanitized-evidence-output-template=codex_cli
cargo run -p pulse-island-spike -- --provider-sanitized-evidence-bundle-validator=claude_code
cargo run -p pulse-island-spike -- --provider-release-elevation-preflight=codex_cli
cargo run -p pulse-island-spike -- --provider-w5-observe-adapter-contract=claude_code
cargo run -p pulse-island-spike -- --provider-w4-completion-gate
Get-Command codex, claude, antigravity
codex --version
claude --version
antigravity --version
```

## Scope Guard

| Boundary | Current status |
|---|---|
| No live provider Hook install | Passing |
| No provider config mutation | Passing |
| No provider adapter creation | Passing |
| No network/App Server query | Passing |
| No transcript/session file parsing | Passing |
| No provider support claim | Passing |
| No raw provider content/configuration in reports | Passing |
| Synthetic config fixtures only | Passing |
| No first-adapter selection without evidence | Passing |
| Resource fixture categories only | Passing |
| Sanitized evidence register only | Passing |
| Official evidence source locator scaffold retains no raw documentation or source URLs | Passing |
| Sanitized probe summaries only | Passing |
| Sanitized scorecard does not select W5 adapter | Passing |
| Hard disqualifiers block W5 without full evidence | Passing |
| Evidence gap summary aggregates all missing direct gates without live actions | Passing |
| Aggregate W4 audit available | Passing |
| Probe Card surface inventory exported without support claims | Passing |
| Readiness output keeps live probe and W5 gates explicit | Passing |
| Read-only local CLI presence preflight recorded as environment evidence only | Passing |
| Sanitized local environment manifest excludes paths/config/account data | Passing |
| Read-only live probe dry-run does not execute provider tasks | Passing |
| Read-only local probe run records only version/environment categories | Passing |
| Read-only local probe summary avoids capability elevation | Passing |
| Read-only resource measurement plan does not execute provider tasks | Passing |
| Probe Cards are exported as executable phase plans without running providers | Passing |
| Evidence retention policy blocks raw provider artifacts from the repo | Passing |
| Live authorization preflight defaults to blocked | Passing |
| Missing capabilities have explicit non-blank rationale | Passing |
| Release decision logs defer provider selection until direct evidence exists | Passing |
| Full capability matrix has all protocol-template rows without blank/elevated cells | Passing |
| Release-label evaluation blocks sanitized fixtures from elevating provider support | Passing |
| Probe phase status matrix keeps P1-P8 not-executed until authorization | Passing |
| W5 start preflight blocks adapter creation until W4 direct evidence selects a supported provider | Passing |
| Direct gate packets are machine-checkable without executing live actions | Passing |
| Direct evidence import checklist rejects weak or raw evidence before release elevation | Passing |
| Authorized evidence runbook defines manual direct-evidence steps without executing provider actions | Passing |
| Sanitized evidence output template keeps repo artifacts content-free and redacted | Passing |
| Sanitized evidence bundle validator rejects raw artifacts before repo import | Passing |
| Release elevation preflight blocks supported_observe until direct evidence and hard gates pass | Passing |
| W5 observe adapter contract scaffold does not create or authorize an adapter | Passing |
| W4 completion gate keeps W5 blocked until direct evidence exists | Passing |

## Next Work

1. Collect direct gate evidence only when explicitly authorized: live provider probe execution, live resource measurement, install/rollback real fixture, Late Attach real result, and terminal-truth real result.
2. Keep lifecycle, install/rollback, Late Attach, terminal truth, and resource-budget elevation gated until each has direct evidence.
3. Keep provider adapters, live Hooks, provider config mutation, and production route activation gated until W4/W5 evidence explicitly authorizes them.
