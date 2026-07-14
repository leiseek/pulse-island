$ErrorActionPreference = "Stop"
$repo = (Resolve-Path "$PSScriptRoot\..").Path
Push-Location $repo
try {
    cargo fmt --all -- --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    $gate = cargo run -q -p pulse-island-spike -- --provider-w4-completion-gate
    $gate | Write-Output
    if ($gate -contains "w4_complete=true" -and $gate -contains "w5_start_allowed=true") {
        Write-Output "release_candidate_ready=true"
    }
    else {
        Write-Output "release_candidate_ready=false"
    }
}
finally {
    Pop-Location
}
Write-Output "deterministic_release_gate_commands_completed=true"
