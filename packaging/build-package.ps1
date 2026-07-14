param(
    [string]$Destination = "$PSScriptRoot\out\pulse-island-1.0.0-rc1"
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path "$PSScriptRoot\..").Path
$release = Join-Path $repo "target\release"

Push-Location $repo
try {
    cargo build --release -p pulse-link-shim -p pulse-link -p pulse-island
} finally {
    Pop-Location
}

New-Item -ItemType Directory -Force -Path $Destination | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "pulse-island-1.0.manifest.json") -Destination $Destination -Force
foreach ($binary in @("pulse-link-shim.exe", "pulse-link.exe", "pulse-island.exe")) {
    Copy-Item -LiteralPath (Join-Path $release $binary) -Destination $Destination -Force
}

& (Join-Path $PSScriptRoot "validate-package.ps1") -Manifest (Join-Path $Destination "pulse-island-1.0.manifest.json") -BinaryRoot $Destination
