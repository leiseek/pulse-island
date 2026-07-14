# Remaining 1.0 execution gates

This file is intentionally short and operational. It prevents the completed transport/package work from being mistaken for a finished product.

1. Codex Hook transaction: run an authorized isolated install, execute a real provider task, verify Shim events, then rollback and prove unrelated config is byte-for-byte preserved.
2. Lifecycle evidence: map SessionStart, activity, PermissionRequest, and Stop to Link state with a real current Codex build; Stop must not be treated as completion without terminal evidence.
3. Late attach: close Island, run a real Hook-backed task, reopen Island, and verify the persisted degraded snapshot followed by fresh evidence.
4. Native UI: connect `apps/pulse-island` to the existing UI model and Win32 HWND/compositor path; the HWND lifecycle smoke now passes, while Signal/Peek/Focus rendering, accessibility, DPI, and no-focus-theft behavior remain.
5. Release candidate: run clean-account install, upgrade, repair, disable, uninstall, and rollback with the package manifest.

Until all five have direct evidence, the release gate remains `not ready`.
