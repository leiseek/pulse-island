# Packaging boundary

`pulse-island-1.0.manifest.json` is the release-candidate contract for the eventual per-user Windows package. It deliberately does not claim that an installer already exists.

The package must contain the Link and Shim binaries, keep state under the per-user Local AppData root, avoid services and PATH changes, and remove only Pulse-owned state and an exact Pulse Hook signature during uninstall. Provider processes and unrelated configuration must remain untouched.

Before packaging is enabled, the release gate must pass the selected provider evidence, production Island host, and clean-account install/rollback checks.

After a staged package exists, run `./packaging/validate-package.ps1 -Manifest <staging>\pulse-island-1.0.manifest.json -BinaryRoot <staging>` to validate the manifest policy and exact package contents without installing anything. The workspace `target\release` directory may contain development/spike executables and is not itself a package root.

To produce a disposable release-candidate staging directory, run `./packaging/build-package.ps1`. The script builds only the three current runtime binaries, copies the manifest, validates the staged contents, and performs no installation or provider configuration change.

The staged package can be removed with `./packaging/uninstall-package.ps1 -InstallRoot <path>`. State deletion is opt-in via `-RemoveState`; only the owned `breadcrumbs.snapshot` file is removed, while sibling files and directories are preserved. Provider configuration and provider processes are never touched.

Run `./packaging/probe-codex-hook-schema.ps1` for the isolated parser-only Codex Hook schema check. It never persists configuration or executes a Hook.

Run `./packaging/probe-codex-readonly.ps1` to repeat the sanitized read-only task probe. A nonzero provider exit is retained as a category and does not expose raw stderr.

Run `./packaging/run-release-gate.ps1` for the deterministic formatting, workspace test, Clippy, and W4 completion-gate sequence.
