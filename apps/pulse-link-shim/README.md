# Pulse Link Shim

`pulse-link-shim` is the short-lived Codex command-hook edge process. It reads one bounded JSON object from stdin, retains only the allow-listed session/event fields, and exits successfully even when Link is unavailable.

For the Codex adapter, invoke the binary with:

```text
pulse-link-shim.exe --provider codex_cli --scope <install-scope>
```

The command receives the provider Hook object on stdin. It never writes that object to disk, emits it to stdout, places it in a command line/environment variable, or returns an approval decision. `--diagnostic` emits only exit/forward/rejection categories.

The current release does not mutate Codex configuration automatically. Installation and rollback must be performed by the future per-user package after the W4 Hook configuration fixture passes.
