[CmdletBinding()]
param(
    [string]$PackagePath,
    [string]$ManifestPath
)

$ErrorActionPreference = 'Stop'
$registry = 'https://registry.npmjs.org/'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Test-PublishedPackageVersion([string]$PackageName, [string]$Version) {
    $publishedVersion = & npm view "$PackageName@$Version" version `
        --registry $registry 2>$null
    return $LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace(
        ($publishedVersion -join "`n").Trim()
    )
}

function Publish-Package(
    [string]$ResolvedPackagePath,
    [string]$PackageName,
    [string]$Version
) {
    if (Test-PublishedPackageVersion -PackageName $PackageName -Version $Version) {
        Write-Step "$PackageName@$Version is already published; refreshing latest tag"
        & npm dist-tag add "$PackageName@$Version" latest --registry $registry
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to update the latest tag for $PackageName@$Version."
        }
        return
    }

    Write-Step "Publishing $PackageName@$Version"
    Write-Step "Package: $ResolvedPackagePath"
    Write-Step "Registry: $registry"
    Write-Step 'Dist tag: latest'
    & npm publish $ResolvedPackagePath --registry $registry --tag latest --access public
    if ($LASTEXITCODE -ne 0) {
        throw "npm publish failed for $PackageName@$Version with exit code $LASTEXITCODE."
    }
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "Required command 'npm' was not found in PATH."
}
if (
    -not [string]::IsNullOrWhiteSpace($PackagePath) -and
    -not [string]::IsNullOrWhiteSpace($ManifestPath)
) {
    throw 'Use either -PackagePath or -ManifestPath, not both.'
}

Write-Step "Checking npm login at $registry"
$npmUserOutput = & npm whoami --registry $registry 2>$null
$whoamiExitCode = $LASTEXITCODE
$npmUser = ($npmUserOutput -join "`n").Trim()
if ($whoamiExitCode -ne 0 -or [string]::IsNullOrWhiteSpace($npmUser)) {
    Write-Host 'npm login is required before publishing.' -ForegroundColor Yellow
    Write-Host "Run: npm login --registry $registry"
    exit 1
}
Write-Step "Logged in to npm as $npmUser"

$npmDirectory = Split-Path -Parent $PSCommandPath
$distDirectory = Join-Path $npmDirectory 'dist'

if (-not [string]::IsNullOrWhiteSpace($PackagePath)) {
    $resolvedPackagePath = [System.IO.Path]::GetFullPath($PackagePath)
    if (-not (Test-Path -LiteralPath $resolvedPackagePath -PathType Leaf)) {
        throw "npm package not found: '$resolvedPackagePath'."
    }
    if ([System.IO.Path]::GetExtension($resolvedPackagePath) -ne '.tgz') {
        throw "npm package must be a .tgz file: '$resolvedPackagePath'."
    }

    Write-Step "Publishing explicit package $resolvedPackagePath"
    & npm publish $resolvedPackagePath --registry $registry --tag latest --access public
    if ($LASTEXITCODE -ne 0) {
        throw "npm publish failed with exit code $LASTEXITCODE."
    }
    Write-Host 'npm package published successfully with the latest tag.' -ForegroundColor Green
    return
}

$resolvedManifestPath = if ([string]::IsNullOrWhiteSpace($ManifestPath)) {
    Join-Path $distDirectory 'pack-manifest.json'
} else {
    [System.IO.Path]::GetFullPath($ManifestPath)
}
if (-not (Test-Path -LiteralPath $resolvedManifestPath -PathType Leaf)) {
    throw "Package manifest not found: '$resolvedManifestPath'. Run pack.ps1 first."
}

try {
    $manifest = Get-Content -Raw -LiteralPath $resolvedManifestPath | ConvertFrom-Json
} catch {
    throw "Failed to read package manifest '$resolvedManifestPath': $($_.Exception.Message)"
}
$packageRecords = @($manifest.packages)
if ($packageRecords.Count -eq 0) {
    throw "Package manifest '$resolvedManifestPath' contains no packages."
}

$manifestDirectory = Split-Path -Parent $resolvedManifestPath
foreach ($record in $packageRecords) {
    if (
        [string]::IsNullOrWhiteSpace($record.name) -or
        [string]::IsNullOrWhiteSpace($record.version) -or
        [string]::IsNullOrWhiteSpace($record.file)
    ) {
        throw "Package manifest '$resolvedManifestPath' contains an invalid package record."
    }
    $resolvedPackagePath = [System.IO.Path]::GetFullPath(
        (Join-Path $manifestDirectory $record.file)
    )
    if (-not (Test-Path -LiteralPath $resolvedPackagePath -PathType Leaf)) {
        throw "Manifest package not found: '$resolvedPackagePath'."
    }
    Publish-Package `
        -ResolvedPackagePath $resolvedPackagePath `
        -PackageName $record.name `
        -Version $record.version
}

Write-Host 'All manifest packages were published successfully with the latest tag.' `
    -ForegroundColor Green
