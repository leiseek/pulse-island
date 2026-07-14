param(
    [string]$Manifest = "$PSScriptRoot\pulse-island-1.0.manifest.json",
    [string]$BinaryRoot = "$PSScriptRoot\..\target\release"
)

$ErrorActionPreference = "Stop"
$spec = Get-Content -LiteralPath $Manifest -Raw | ConvertFrom-Json
if ($spec.install_scope -ne "per_user" -or $spec.requires_service -or $spec.modifies_path) {
    throw "package policy mismatch"
}

$missing = @(
    $spec.binaries | Where-Object {
        -not (Test-Path -LiteralPath (Join-Path $BinaryRoot $_))
    }
)
if ($missing.Count -gt 0) {
    throw "missing release binaries: $($missing -join ', ')"
}
$allowed = @($spec.binaries)
$unexpected = @(Get-ChildItem -LiteralPath $BinaryRoot -Filter "*.exe" -File | Where-Object {
    $allowed -notcontains $_.Name
})
if ($unexpected.Count -gt 0) {
    throw "unexpected executable in package root: $($unexpected.Name -join ', ')"
}

Write-Output "package_manifest_valid=true"
Write-Output "package_version=$($spec.version)"
Write-Output "binary_count=$($spec.binaries.Count)"
Write-Output "provider_integration=$($spec.provider_integration)"
