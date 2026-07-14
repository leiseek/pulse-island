# Pulse Island · W1 Gate Audit

**Status:** Accepted for W2/W3 sequencing
**Scope:** W1 State Truth Kernel only
**Last updated:** 2026-07-07

This audit maps W1 requirements from `13-spike-b-state-kernel.md`, `24-implementation-work-packages.md`, and `25-consistency-closure.md` to current repository evidence.

## Verification Commands

Latest verified commands:

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets
cargo tree --workspace --depth 1
```

## W0 Foundation Evidence

| Requirement | Evidence | Status |
|---|---|---|
| Provider-neutral workspace exists | Root `Cargo.toml` workspace; crates under `crates/` | Passing |
| No provider/UI/Win32/network/SQLite dependency in core crates | `cargo tree --workspace --depth 1` shows only workspace deps | Passing |
| Core strings are bounded and content-minimizing | `BoundedText`, `SAFE_TEXT_MAX_BYTES`, forbidden marker tests | Passing |
| No generic utils crate | Workspace members list contains only named domain/protocol/testkit/W1 crates | Passing |

## W1 Fixture Map

| Requirement | Evidence | Status |
|---|---|---|
| Basic lifecycle | `lifecycle_and_terminal_protection`, `same_fixture_and_clock_yield_identical_snapshot_and_plan` | Passing |
| Process-only discovery ceiling | `process_only_antigravity_remains_observed_without_task_semantics`, `process_only_antigravity_has_observed_ceiling` | Passing |
| Waiting truthfulness | `waiting_truthfulness_and_clear`, `waiting_summary_survives_generic_activity_until_clear` | Passing |
| Stale waiting recovery | `stale_waiting_can_recover_to_running_on_fresh_activity` | Passing |
| Terminal protection | `storm_activity_does_not_revert_terminal_state`, `failed_terminal_summary_survives_late_completed_event` | Passing |
| Process exit without terminal evidence | `process_exit_without_terminal_evidence_goes_offline_not_completed` | Passing |
| Identity/PID safety | `pid_identity_uses_start_time_and_splits_conflicts` | Passing |
| Freshness and degradation | `freshness_degrades_then_fresh_activity_recovers` | Passing |
| Summary privacy | `waiting_summary_survives_generic_activity_until_clear`, `failed_terminal_summary_survives_late_completed_event` | Passing |
| Fuel separation | `stale_and_revoked_clear_block_without_lifecycle_coupling`, `fuel_buckets_are_provider_scoped_and_never_aggregated`, `fuel_revoked_removes_verified_limit_claim_without_inventing_running`, `fuel_revoked_leaves_lifecycle_unchanged` | Passing |
| Token sample safety | `token_samples_never_create_negative_or_duplicate_burn` | Passing |
| Arbitration tier order | `canonical_ordering`, `fuel_risk_below_waiting_and_peek_capped`, `resource_stall_ranks_below_fuel_risk_and_above_running` | Passing |
| Arbitration tie-breaks | `same_tier_prefers_newer_activity` | Passing |
| Route labels by strength and kind | `route_labels_require_matching_route_strength`, `strong_is_never_exact`, `route_kind_labels_must_match_strength` | Passing |
| Route freshness downgrade | `stale_exact_route_downgrades_to_useful_workspace_action`, `stale_exact_route_downgrades_to_fallback_label` | Passing |
| Privacy-profile retention | `privacy_profiles_produce_explicit_breadcrumb_retention_decisions`, `strict_terminal_transition_is_not_retained_for_restart` | Passing |
| Protocol hardening | `malformed_or_unapproved_input_is_rejected_before_snapshot_mutation`, `repeated_invalid_preflight_is_deterministic_and_content_free`, `secret_like_bounded_text_and_frames_are_rejected` | Passing |
| Island reconnect shape | `island_reconnect_protocol_is_snapshot_and_delta_only` | Passing |
| Safe Mode Shim ingress | `safe_mode_shim_ingress_prevents_link_wake_and_forwarding` | Passing |
| Feature capability axis | `feature_capabilities_are_declared_without_upgrading_release_status` | Passing |
| Storm bounds | `hundreds_of_tasks_keep_primary_and_peek_bounded`, `storm_activity_does_not_revert_terminal_state` | Passing |

## Sequencing Decision

W1 has no known Missing or Incomplete audit rows. Its truth fixtures are sufficient for the canonical sequence to proceed through W2 and into W3.

1. Re-run the full verification commands after any further changes.
2. Confirm this audit still has no `Missing` or `Incomplete` rows.
3. W3 Link/Shim/Drop Mode may proceed under `14-spike-c-link-transport-drop-mode.md`; live provider Hook installation and provider adapter work remain later-gated.
