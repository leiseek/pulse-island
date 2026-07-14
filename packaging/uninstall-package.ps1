param(
    [Parameter(Mandatory = $true)]
    [string]$InstallRoot,
    [switch]$RemoveState
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path -LiteralPath $InstallRoot).Path
$manifestPath = Join-Path $root "pulse-island-1.0.manifest.json"
if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "package manifest not found"
}
$spec = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
foreach ($binary in $spec.binaries) {
    $path = Join-Path $root $binary
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force
    }
}
Remove-Item -LiteralPath $manifestPath -Force

if ($RemoveState) {
    $stateRoot = Join-Path $env:LOCALAPPDATA "PulseIsland"
    $ownedState = Join-Path $stateRoot "breadcrumbs.snapshot"
    if (Test-Path -LiteralPath $ownedState -PathType Leaf) {
        Remove-Item -LiteralPath $ownedState -Force
    }
}

Write-Output "package_uninstall_completed=true"
Write-Output "provider_configuration_touched=false"
Write-Output "provider_processes_stopped=false"
