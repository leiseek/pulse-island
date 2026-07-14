$ErrorActionPreference = "Stop"
$probeHome = Join-Path $env:TEMP ("pulse-codex-schema-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $probeHome | Out-Null
$stdout = Join-Path $probeHome "stdout.txt"
$stderr = Join-Path $probeHome "stderr.txt"
$config = 'hooks=[{type="command",command="pulse-link-shim.exe",timeout_ms=1000}]'
try {
    $env:CODEX_HOME = $probeHome
    $process = Start-Process -FilePath "codex.cmd" -ArgumentList @("exec", "--help", "-c", $config) -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru -WindowStyle Hidden
    $completed = $process.WaitForExit(15000)
    if (-not $completed) {
        Stop-Process -Id $process.Id -Force
        throw "Codex schema probe timed out"
    }
    $errorText = Get-Content -LiteralPath $stderr -Raw
    if ($process.ExitCode -ne 0 -or $errorText -match "(?i)error|unknown") {
        throw "Codex Hook candidate schema rejected"
    }
    Write-Output "codex_hook_schema_parser_accepted=true"
    Write-Output "config_persisted=false"
    Write-Output "hook_executed=false"
}
finally {
    Remove-Item -LiteralPath $probeHome -Recurse -Force -ErrorAction SilentlyContinue
}
