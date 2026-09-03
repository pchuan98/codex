[CmdletBinding()]
param(
    [string]$TargetDirectory,
    [string]$OutputDirectory,
    [string]$PackageVersion,
    [switch]$DebugBuild
)

$ErrorActionPreference = 'Stop'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

$scriptsDirectory = Split-Path -Parent $PSCommandPath
$npmPackScript = Join-Path $scriptsDirectory 'npm/pack.ps1'
$npmPushScript = Join-Path $scriptsDirectory 'npm/push.ps1'
$npmDirectory = Join-Path $scriptsDirectory 'npm'

if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $npmDirectory 'dist'
}
$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
$manifestPath = Join-Path $outputPath 'pack-manifest.json'

$platformParameters = @{
    OutputDirectory = $outputPath
    ReleaseBuild = -not $DebugBuild
}
if (-not [string]::IsNullOrWhiteSpace($TargetDirectory)) {
    $platformParameters.TargetDirectory = $TargetDirectory
}
if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
    $platformParameters.PackageVersion = $PackageVersion
}

Write-Step 'Building the native package for this machine'
& $npmPackScript @platformParameters

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw "Platform package manifest was not created: '$manifestPath'."
}
$platformManifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
$resolvedVersion = [string]$platformManifest.version
$platformRecords = @($platformManifest.packages)
if ([string]::IsNullOrWhiteSpace($resolvedVersion) -or $platformRecords.Count -ne 1) {
    throw "Platform package manifest '$manifestPath' is invalid."
}

Write-Step "Building the CPA launcher package with version $resolvedVersion"
& $npmPackScript `
    -OnlyPack `
    -PackageVersion $resolvedVersion `
    -OutputDirectory $outputPath

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw "Launcher package manifest was not created: '$manifestPath'."
}
$launcherManifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
$launcherRecords = @($launcherManifest.packages)
if ($launcherRecords.Count -ne 1 -or $launcherRecords[0].name -ne '@pchuan98/cpa') {
    throw "Launcher package manifest '$manifestPath' is invalid."
}

$combinedManifest = [ordered]@{
    version = $resolvedVersion
    registry = 'https://registry.npmjs.org/'
    packages = @($platformRecords + $launcherRecords)
}
$combinedManifest | ConvertTo-Json -Depth 10 |
    Set-Content -LiteralPath $manifestPath -Encoding utf8

Write-Host ''
Write-Host "CPA package version: $resolvedVersion" -ForegroundColor Green
Write-Host "Package manifest: $manifestPath"
Write-Host 'Publish with:' -ForegroundColor Yellow
if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)) {
    Write-Host ".\DEV\scripts\npm\push.ps1 -ManifestPath '$manifestPath'"
} else {
    Write-Host "pwsh ./DEV/scripts/npm/push.ps1 -ManifestPath '$manifestPath'"
}
