# Pulse Island · W3 Gate Audit

**Status:** Accepted for W4 provider-probe start  
**Scope:** W3 Link / Shim / Drop Mode with synthetic envelopes and fake clients only  
**Last updated:** 2026-07-07

This audit tracks implementation against `14-spike-c-link-transport-drop-mode.md`. W3 is accepted for W4 sequencing and remains a regression gate, not the active implementation boundary.

## Current W3 Slice

Implemented:

- `pulse-link-core` with content-free local frame header encoding/decoding.
- Spike C payload caps for Hook ingress, Island control, and full snapshots.
- Header rejection before payload parsing for bad magic, unsupported version, unknown message kind, and oversized payload.
- `pulse-link-shim` bounded input preflight with Safe Mode no-op behavior.
- Shim fail-open behavior for oversized input, forbidden content, malformed/preflight rejection, and Link delivery failure.
- Protocol preflight now accepts the Spike C 8 KiB ingress boundary instead of the older 4 KiB W1-era cap.
- `pulse-link-core` pure lifecycle state model for `NotRunning`, `Starting`, `Warm`, `Active`, `IslandActive`, `DropMode`, `GracePeriod`, and `CheckpointAndExit`.
- Lifecycle contract coverage for wake/ready, first valid active event, Island attach/detach, last active terminal, grace expiry, checkpoint exit, and grace cancellation by a new valid event.
- `pulse-persistence` bounded breadcrumb abstraction with active-task cap 128, recent-terminal cap 20, per-task size cap 1 KiB, total snapshot cap 256 KiB, lifecycle bucket validation, and in-memory complete-replacement store.
- `pulse-persistence` file-backed atomic breadcrumb store with complete replacement snapshot writes, same-directory temporary file cleanup, bounded load/checkpoint behavior, missing-file empty snapshot behavior, and oversized-checkpoint rejection that leaves the previous valid snapshot intact.
- `pulse-link-core` fake Island protocol session for `Hello`, `GetSnapshot`, `Subscribe`, monotonic `SnapshotDelta`, and revision-gap full-snapshot recovery.
- `pulse-win32` content-free local object name derivation for Link mutex, ingress pipe, Island pipe, and ready event without exposing raw install id, raw user SID, or raw logon session.
- `pulse-win32` pure single-instance ownership model for first owner, existing owner reuse, bounded stale mutex/pipe retries, fail-open stale exhaustion, and per-logon-session scoping.
- `pulse-link-core` initial handoff launch plan that uses `--wake-if-needed --handoff-stdin`, inherited stdin, no environment payload, no temp file name, and rejects oversized Hook payloads before launch planning.
- `pulse-win32-link` unsafe-boundary crate for W3 Link transport, with safe preflight for mutex, ingress pipe, Island pipe, and inherited handoff pipe setup.
- `pulse-win32-link` native backend executor that advances handle state only after preflight and native API success, plus MSVC-only `windows-sys` adapter for `CreateMutexW`, `CreateNamedPipeW`, `CreatePipe`, and `CloseHandle`.
- `pulse-win32-link` native shutdown cleanup that closes handoff write/read handles, Island pipe, ingress pipe, and mutex in reverse ownership order, clears successfully closed handles, and retains failed close handles for bounded retry diagnostics.
- `pulse-win32-link` Island client pipe connection seam with safe preflight requiring the Island pipe server before client connection, duplicate-client rejection, and MSVC `CreateFileW` adapter for connecting a fake Island client to the scoped Island pipe.
- `pulse-win32-link` MSVC-only OS-backed transport smoke harness that creates real scoped mutex, ingress pipe, Island pipe, inherited handoff pipe, connects a fake Island client, and closes all six owned handles without residue.
- `pulse-win32-link` MSVC-only real ingress named-pipe frame/ack harness: a client writes a `LinkFrameHeader` byte frame to the ingress pipe server, the server reads exact bytes, writes a one-byte acknowledgement, the client reads it back, and client/server/mutex handles are closed.
- `pulse-win32-link` MSVC-only real ingress named-pipe multi-frame loop: one client/server pipe connection can round-trip multiple fixed frames, acknowledge every frame, and close client/server/mutex handles without residue.
- `pulse-win32-link` MSVC-only real Island named-pipe request/response loop: one client/server Island pipe connection can round-trip content-free request/response byte messages and close client/server/mutex handles without residue.
- `apps/pulse-link-shim` native transport seam using `pulse-win32-link`: existing Link ingress is reused when acknowledged, first wake creates an inherited handoff pipe before launch request, native handoff setup failure remains fail-open, and payload metadata does not leak through argv/env/temp fields.
- `apps/pulse-link` native startup seam using `pulse-win32-link`: Link creates/acquires the scoped mutex, ingress pipe server, and Island pipe server before reporting startup readiness.
- `apps/pulse-link` native transport runtime seam retains startup handles and invokes `pulse-win32-link` shutdown cleanup after the final checkpoint.
- `apps/pulse-link` MSVC-only OS-backed ingress reducer harness: a real ingress named-pipe frame/ack round trip decodes a content-free Hook frame header, rejects non-Hook/non-header-only input before mutation, drives the synthetic reducer, writes a complete replacement checkpoint, and closes all native handles.
- `apps/pulse-link` MSVC-only OS-backed ingress reducer loop: multiple real ingress frames can be acknowledged on one connection, malformed headers are rejected before reducer mutation, later valid frames still reduce/checkpoint, and native handles close without residue.
- `apps/pulse-link` pure synthetic runner that connects admitted events, the reducer, lifecycle transitions, and injectable breadcrumb stores, including a file-backed runtime restart/recovery path that restores active breadcrumbs as degraded until fresh evidence arrives.
- `apps/pulse-link` Drop Mode grace driver with Spike C fixed 90-second deadline, caller-owned clock, new-event grace cancellation, final checkpoint write at expiry, and C8 scenario coverage through the driver instead of manual lifecycle forcing.
- `apps/pulse-link` MSVC-only OS-backed C8 residue harness: starts real Link transport handles, drives terminal synthetic reducer state into the 90-second grace driver, writes the final checkpoint at expiry, transitions to `NotRunning`, closes mutex/ingress/Island handles, and verifies a short-lived child process exits with no child residue.
- `apps/pulse-link` MSVC-only OS-backed Spike C C0-C9 aggregate harness: covers all ten Spike C scenarios, binds transport-specific paths to real mutex/named-pipe/handoff/residue evidence, preserves fail-open/provider-neutral behavior, and confirms zero retained native handles across OS-backed slices.
- `apps/pulse-link-spike-client` fake Island client wrapper for attach and delta receipt, plus a native Island pipe connection seam before the pure fake session attach flow.
- `apps/pulse-link-spike-client` pipe-backed fake Island message loop seam that requires a connected Island pipe before startup and handles `Hello`, `GetSnapshot`, `Subscribe`, monotonic `SnapshotDelta`, and revision-gap full-snapshot recovery without raw event replay.
- `apps/pulse-link-spike-client` MSVC-only OS-backed Island protocol loop: real Island pipe request/response bytes are bound to the fake Island `Hello`, `GetSnapshot`, `Subscribe`, monotonic delta, and revision-gap recovery sequence, with native handle cleanup.
- `apps/pulse-link` pure synthetic C0-C9 scenario harness covering existing Link delivery, first Hook wake, parallel Shim race, Link unavailable, malformed/oversized ingress, Drop Mode breadcrumb, Island attach/detach/reattach, restart recovery as degraded, grace exit, and bounded event storm. C1/C2 now use the pure ownership/handoff models instead of bare counters.

Not yet implemented in W3:

- None blocking W4 provider-probe start. Provider adapters, live Hook installation, provider config mutation, and production route activation remain gated by W4/W5.

## Verification Commands

Latest verified W3 slice commands:

```text
cargo test -p pulse-link-core
cargo test -p pulse-link-core --test handoff_contract
cargo test -p pulse-link-shim
cargo test -p pulse-link-shim --test shim_fail_open
cargo test -p pulse-protocol
cargo test -p pulse-persistence
cargo test -p pulse-persistence --test breadcrumb_contract
cargo test -p pulse-link
cargo test -p pulse-link --test link_runner_contract
cargo test -p pulse-link --test link_native_transport_contract
cargo test -p pulse-link --test link_scenarios_contract
cargo test -p pulse-link-spike-client
cargo test -p pulse-link-spike-client --test spike_client_contract
cargo test -p pulse-win32 --test link_names_contract
cargo test -p pulse-win32 --test link_ownership_contract
cargo test -p pulse-win32-link --test link_transport_contract
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32-link windows_sys_os_transport_harness_creates_pipe_mutex_handoff_and_cleans_up'
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-win32-link windows_sys_ingress_pipe_round_trips_frame_header_and_ack'
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_ingress_frame_ack_drives_reducer_and_checkpoint'
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_ingress_frame_ack_loop_rejects_bad_frame_without_stopping_reducer'
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_grace_exit_closes_transport_and_leaves_no_child_residue'
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link windows_sys_c0_c9_harness_covers_all_spike_c_scenarios'
cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustup run stable-x86_64-pc-windows-msvc cargo test -p pulse-link-spike-client windows_sys_island_pipe_protocol_loop_handles_attach_delta_and_gap'
rustup run stable-x86_64-pc-windows-msvc cargo check -p pulse-win32-link
```

## Scope Guard

| Boundary | Current status |
|---|---|
| Synthetic inputs only | Passing |
| No live provider Hook install | Passing |
| No provider config mutation | Passing |
| No provider adapter | Passing |
| No network | Passing |
| No UI/GPU in Link/Shim | Passing |

## Next Work

1. Continue W4 Provider Probe Harness without installing live Hooks or mutating provider configuration.
2. Keep W3 OS-backed C0-C9, residue, and protocol harnesses as regression gates.
