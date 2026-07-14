$ErrorActionPreference = "Stop"
$probeRoot = Join-Path $env:TEMP ("pulse-codex-readonly-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $probeRoot | Out-Null
$stdout = Join-Path $probeRoot "stdout.jsonl"
$stderr = Join-Path $probeRoot "stderr.txt"
try {
    Push-Location $probeRoot
    try {
        & codex.cmd exec -s read-only --skip-git-repo-check --json "Respond with the single word READY" 1> $stdout 2> $stderr
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    $events = Get-Content -LiteralPath $stdout -Raw
    $hasReady = $events -match 'READY'
    $hasTurn = $events -match 'turn.completed'
    Write-Output "codex_readonly_probe_exit=$exitCode"
    Write-Output "turn_completed=$hasTurn"
    Write-Output "expected_response_observed=$hasReady"
    Write-Output "raw_output_retained=false"
}
finally {
    Remove-Item -LiteralPath $probeRoot -Recurse -Force -ErrorAction SilentlyContinue
}
