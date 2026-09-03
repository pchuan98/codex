[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$BuildArguments
)

$ErrorActionPreference = 'Stop'

$BuildCodex = $false
$BuildHost = $false
$Release = $false
$TargetDirectory = $null
$RequestedTarget = $null

for ($argumentIndex = 0; $argumentIndex -lt $BuildArguments.Count; $argumentIndex++) {
    switch ($BuildArguments[$argumentIndex]) {
        '--codex' {
            $BuildCodex = $true
        }
        '--host' {
            $BuildHost = $true
        }
        '--release' {
            $Release = $true
        }
        '--d' {
            $argumentIndex++
            if ($argumentIndex -ge $BuildArguments.Count -or [string]::IsNullOrWhiteSpace($BuildArguments[$argumentIndex])) {
                throw '--d requires a target directory.'
            }
            $TargetDirectory = $BuildArguments[$argumentIndex]
        }
        '--target' {
            $argumentIndex++
            if ($argumentIndex -ge $BuildArguments.Count -or [string]::IsNullOrWhiteSpace($BuildArguments[$argumentIndex])) {
                throw '--target requires linux-x64 or linux-arm64.'
            }
            $RequestedTarget = $BuildArguments[$argumentIndex]
            if ($RequestedTarget -notin @('linux-x64', 'linux-arm64')) {
                throw "Unsupported target '$RequestedTarget'. Expected linux-x64 or linux-arm64."
            }
        }
        default {
            throw "Unknown argument '$($BuildArguments[$argumentIndex])'."
        }
    }
}

if (-not $BuildCodex -and -not $BuildHost) {
    throw 'Select at least one build target with --codex or --host.'
}

$isWindowsPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)
$isLinuxPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Linux
)
if (-not $isWindowsPlatform -and -not $isLinuxPlatform) {
    throw 'This script supports Windows and Linux only.'
}

$osArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$localTarget = if ($isWindowsPlatform -and $osArchitecture -eq 'X64') {
    'windows-x64'
} elseif ($isLinuxPlatform -and $osArchitecture -eq 'X64') {
    'linux-x64'
} elseif ($isLinuxPlatform -and $osArchitecture -eq 'Arm64') {
    'linux-arm64'
} else {
    throw "Unsupported local platform architecture: $osArchitecture."
}

$scriptDirectory = Split-Path -Parent $PSCommandPath
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $scriptDirectory '../..'))
$cargoDirectory = Join-Path $repositoryRoot 'codex-rs'
$cargoManifest = Join-Path $cargoDirectory 'Cargo.toml'
if (-not (Test-Path -LiteralPath $cargoManifest -PathType Leaf)) {
    throw "Cargo workspace not found at '$cargoDirectory'."
}

if (-not [string]::IsNullOrWhiteSpace($RequestedTarget) -and $RequestedTarget -ne $localTarget) {
    if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
        $TargetDirectory = Join-Path $cargoDirectory "target/docker/$RequestedTarget"
    }
    $dockerParameters = @{
        Operation = 'build'
        Target = $RequestedTarget
        TargetDirectory = [System.IO.Path]::GetFullPath($TargetDirectory)
        BuildCodex = $BuildCodex
        BuildHost = $BuildHost
        ReleaseBuild = $Release
    }
    & (Join-Path $scriptDirectory 'docker.ps1') @dockerParameters
    if ($LASTEXITCODE -ne 0) {
        throw "Docker build failed with exit code $LASTEXITCODE."
    }
    return
}

foreach ($commandName in @('cargo', 'rustc')) {
    if (-not (Get-Command $commandName -ErrorAction SilentlyContinue)) {
        throw "Required command '$commandName' was not found in PATH."
    }
}

$rustcInfo = (& rustc -vV) -join "`n"
if ($LASTEXITCODE -ne 0) {
    throw 'Failed to inspect the active Rust toolchain.'
}
$expectedRustHost = switch ($localTarget) {
    'windows-x64' { 'x86_64-pc-windows-msvc' }
    'linux-x64' { 'x86_64-unknown-linux-(gnu|musl)' }
    'linux-arm64' { 'aarch64-unknown-linux-(gnu|musl)' }
}
if ($rustcInfo -notmatch "(?m)^host: $expectedRustHost`$") {
    throw "The active Rust toolchain is not supported on this platform. rustc -vV reported:`n$rustcInfo"
}

if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
    $TargetDirectory = Join-Path $cargoDirectory 'target'
}
$targetPath = [System.IO.Path]::GetFullPath($TargetDirectory)
New-Item -ItemType Directory -Force -Path $targetPath | Out-Null

$rustHostMatch = [regex]::Match($rustcInfo, '(?m)^host: (?<host>[^\r\n]+)$')
if (-not $rustHostMatch.Success) {
    throw 'Failed to read the rustc host target.'
}
$rustHost = $rustHostMatch.Groups['host'].Value
$v8ReleaseBase = 'https://github.com/openai/codex/releases/download/rusty-v8-v150.4.0'
$bindingName = "src_binding_ptrcomp_sandbox_release_$rustHost.rs"
$bindingPath = Join-Path $targetPath $bindingName

if (-not (Test-Path -LiteralPath $bindingPath -PathType Leaf)) {
    $bindingUri = "$v8ReleaseBase/$bindingName"
    Write-Host "Downloading V8 binding to '$bindingPath'..."
    Invoke-WebRequest -Uri $bindingUri -OutFile $bindingPath
}

$archiveName = if ($isWindowsPlatform) {
    "rusty_v8_ptrcomp_sandbox_release_$rustHost.lib.gz"
} else {
    "librusty_v8_ptrcomp_sandbox_release_$rustHost.a.gz"
}
$env:RUSTY_V8_ARCHIVE = "$v8ReleaseBase/$archiveName"
$env:RUSTY_V8_SRC_BINDING_PATH = $bindingPath

$env:CARGO_TARGET_DIR = $targetPath

$cargoArguments = @('build')
if ($Release) {
    $cargoArguments += '--release'
}
if ($BuildCodex) {
    $cargoArguments += @('--bin', 'codex')
}
if ($BuildHost) {
    $cargoArguments += @('--bin', 'codex-code-mode-host')
}

Write-Host "Cargo target directory: $targetPath"
Write-Host "Running: cargo $($cargoArguments -join ' ')"

$migrationFiles = Get-ChildItem -LiteralPath (Join-Path $cargoDirectory 'state') `
    -Recurse -File -Filter '*.sql' |
    Where-Object { $_.Directory.Name -like '*migrations' }

$lineEndingName = if ($isWindowsPlatform) { 'CRLF' } else { 'LF' }
$gitDirectoryOutput = & git -C $repositoryRoot rev-parse --git-dir 2>$null
if ($LASTEXITCODE -eq 0) {
    $gitDirectory = [System.IO.Path]::GetFullPath(
        (Join-Path $repositoryRoot ($gitDirectoryOutput.Trim()))
    )
    $infoDirectory = Join-Path $gitDirectory 'info'
    $attributesPath = Join-Path $infoDirectory 'attributes'
    New-Item -ItemType Directory -Force -Path $infoDirectory | Out-Null

    $attributesText = if (Test-Path -LiteralPath $attributesPath -PathType Leaf) {
        Get-Content -Raw -LiteralPath $attributesPath
    } else {
        ''
    }
    $attributesText = [regex]::Replace(
        $attributesText,
        '(?ms)^# codex-cpa-build:start\r?\n.*?^# codex-cpa-build:end\r?\n?',
        ''
    ).TrimEnd("`r", "`n")
    $gitEol = if ($isWindowsPlatform) { 'crlf' } else { 'lf' }
    $attributesBlock = @"
# codex-cpa-build:start
codex-rs/state/**/*.sql text eol=$gitEol
# codex-cpa-build:end
"@
    $updatedAttributes = if ([string]::IsNullOrWhiteSpace($attributesText)) {
        "$attributesBlock`n"
    } else {
        "$attributesText`n$attributesBlock`n"
    }
    [System.IO.File]::WriteAllText(
        $attributesPath,
        $updatedAttributes,
        [System.Text.UTF8Encoding]::new($false)
    )
    Write-Host "Configured local Git attributes for $lineEndingName migrations."
}

Write-Host "Ensuring embedded SQL migrations use $lineEndingName..."
$changedMigrationCount = 0
foreach ($migrationFile in $migrationFiles) {
    $originalBytes = [System.IO.File]::ReadAllBytes($migrationFile.FullName)
    $migrationText = [System.Text.Encoding]::UTF8.GetString($originalBytes)
    $migrationText = $migrationText.Replace("`r`n", "`n").Replace("`r", "`n")
    if ($isWindowsPlatform) {
        $migrationText = $migrationText.Replace("`n", "`r`n")
    }
    $normalizedBytes = [System.Text.UTF8Encoding]::new($false).GetBytes($migrationText)
    $originalBase64 = [Convert]::ToBase64String($originalBytes)
    $normalizedBase64 = [Convert]::ToBase64String($normalizedBytes)
    if ($originalBase64 -ne $normalizedBase64) {
        [System.IO.File]::WriteAllBytes($migrationFile.FullName, $normalizedBytes)
        $changedMigrationCount++
    }
}
Write-Host "Migration files changed: $changedMigrationCount"
if ($LASTEXITCODE -eq 0 -and $null -ne $gitDirectory) {
    $migrationPaths = $migrationFiles | ForEach-Object {
        [System.IO.Path]::GetRelativePath($repositoryRoot, $_.FullName).Replace('\', '/')
    }
    & git -C $repositoryRoot add --refresh -- @migrationPaths
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to refresh the Git index after migration normalization.'
    }
}

Push-Location $cargoDirectory
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}

$buildProfile = if ($Release) { 'release' } else { 'debug' }
Write-Host "Build output: $(Join-Path $targetPath $buildProfile)"
