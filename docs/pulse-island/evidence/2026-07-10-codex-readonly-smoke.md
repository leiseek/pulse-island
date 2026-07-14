# Codex CLI Read-only Smoke Evidence

Date: 2026-07-10  
Provider: `codex_cli`  
Environment: synthetic temporary workspace  
Retention: sanitized categories only

## Run

An authorized, non-mutating smoke invocation was run with Codex `exec`, read-only sandbox, repository check disabled, JSON event output enabled, and a one-word response request. The temporary workspace and raw stdout/stderr were deleted immediately after classification.

## Observed categories

| Check | Result |
| --- | --- |
| process started | observed |
| turn started/completed | observed |
| assistant response item | observed |
| expected response token | observed |
| file mutation requested | not observed |
| raw prompt/transcript retained | false |
| provider configuration read/written | false |

Additional read-only CLI surface checks observed the `hook` feature as stable and the App Server help surface as callable. These checks did not enable/disable features or alter configuration.

This proves only that the installed CLI can complete one read-only non-interactive turn. It does not prove Hook installation, lifecycle event delivery, Late Attach, terminal truth, routing, or resource budgets; release status therefore remains `not_probed`.

## Configuration preflight

The current user-level Codex configuration was inspected read-only for shape only. It contains no existing Hook or Pulse entries. No configuration file was changed; this observation is preparation for a separately authorized install/rollback fixture, not Hook installation evidence.

An isolated temporary `CODEX_HOME` accepted a `hooks=[]` CLI configuration override during `codex exec --help`. The temporary home was removed afterward. This confirms only that the current CLI recognizes the configuration key; it does not establish the persisted Hook schema or installation transaction.

A second isolated override using a single command-shaped hook object was also accepted by the CLI parser with no configuration error. The object was never persisted and no hook was executed; exact field semantics and rollback behavior remain unproven.

A third parser-only check accepted the candidate fields `type=command`, `command`, and `timeout_ms` in an isolated override. This is schema reconnaissance only; it is not evidence of persisted installation or execution.
