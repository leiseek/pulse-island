# Native Island smoke evidence

**Date:** 2026-07-11  
**Target:** `stable-x86_64-pc-windows-msvc`  
**Command:** `cargo +stable-x86_64-pc-windows-msvc run -p pulse-island -- --native-smoke`

## Result

```text
native_smoke=passed applied_commands=6 pump_removed=1 destroyed=true
```

The production host created the content-free compact HWND, applied the bounded
native adapter command sequence, drained one queued message without blocking,
and destroyed the window. No provider process, configuration, or payload was
used.

This evidence closes the HWND lifecycle smoke portion only. It does not claim
that Signal, Peek, or Focus Card rendering is complete.
