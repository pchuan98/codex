# Keep arguments unbound so all Codex flags pass through unchanged.
$ErrorActionPreference = 'Stop'
$runArguments = $args
$distPath = Join-Path $PSScriptRoot 'npm/dist'
$manifestPath = Join-Path $distPath 'pack-manifest.json'
if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw 'No package manifest found. Run just pack first.'
}
foreach ($command in @('node', 'npm')) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "Required command '$command' was not found in PATH."
    }
}
$platform = (& node -p 'process.platform + "-" + process.arch').Trim()
if ($LASTEXITCODE -ne 0) {
    throw 'Could not determine the current Node platform.'
}
$manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
$archives = foreach ($name in @("@pchuan98/cpa-$platform", '@pchuan98/cpa')) {
    $records = @($manifest.packages | Where-Object { $_.name -eq $name })
    if ($records.Count -ne 1 -or $records[0].version -ne $manifest.version) {
        throw "Manifest must contain $name at version $($manifest.version). Run just pack first."
    }
    $archive = Join-Path $distPath $records[0].file
    if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) {
        throw "Package archive not found: '$archive'. Run just pack first."
    }
    $archive
}

# Install both exact local archives together so the launcher resolves its native dependency.
# Offline mode prevents fetching any package from a registry; dist is ignored by Git.
$runtimePath = Join-Path $distPath 'run'
Write-Host "Running packaged CPA $($manifest.version) ($platform)" -ForegroundColor Cyan
& npm install --prefix $runtimePath --offline --ignore-scripts --no-audit --no-fund @archives
if ($LASTEXITCODE -ne 0) {
    throw 'Could not prepare the local packaged CPA runtime.'
}
$launcher = Join-Path $runtimePath 'node_modules/@pchuan98/cpa/bin/cpa.js'
& node $launcher @runArguments
exit $LASTEXITCODE
