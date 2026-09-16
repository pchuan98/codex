[CmdletBinding()]
param(
    [string]$TargetDirectory,
    [string]$OutputDirectory,
    [Alias('Release')]
    [switch]$ReleaseBuild,
    [string]$PackageVersion,
    [Alias('LauncherOnly')]
    [switch]$OnlyPack
)

$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
    $TargetDirectory = $env:CARGO_TARGET_DIR
}
$registry = 'https://registry.npmjs.org/'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function ConvertTo-NormalizedRepositoryUrl([string]$Url) {
    $normalizedUrl = $Url.Trim().TrimEnd('/')
    if ($normalizedUrl.EndsWith('.git', [System.StringComparison]::OrdinalIgnoreCase)) {
        $normalizedUrl = $normalizedUrl.Substring(0, $normalizedUrl.Length - 4)
    }
    return $normalizedUrl.ToLowerInvariant()
}

function Get-PlatformDefinition([string]$PlatformTarget) {
    switch ($PlatformTarget) {
        'windows-x64' {
            return [pscustomobject]@{
                Target = 'windows-x64'
                PackageName = '@pchuan98/cpa-win32-x64'
                Os = 'win32'
                Cpu = 'x64'
                ExecutableSuffix = '.exe'
            }
        }
        'linux-x64' {
            return [pscustomobject]@{
                Target = 'linux-x64'
                PackageName = '@pchuan98/cpa-linux-x64'
                Os = 'linux'
                Cpu = 'x64'
                ExecutableSuffix = ''
            }
        }
        'linux-arm64' {
            return [pscustomobject]@{
                Target = 'linux-arm64'
                PackageName = '@pchuan98/cpa-linux-arm64'
                Os = 'linux'
                Cpu = 'arm64'
                ExecutableSuffix = ''
            }
        }
        default {
            throw "Unsupported package target '$PlatformTarget'."
        }
    }
}

function Get-PackageArchivePath(
    [string]$PackageName,
    [string]$Version,
    [string]$OutputPath
) {
    $packageStem = $PackageName.TrimStart('@').Replace('/', '-')
    return Join-Path $OutputPath "$packageStem-$Version.tgz"
}

function Test-PublishedPackageVersion([string]$PackageName, [string]$Version) {
    $publishedVersion = & npm view "$PackageName@$Version" version `
        --registry $registry 2>$null
    return $LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace(
        ($publishedVersion -join "`n").Trim()
    )
}

function Get-AvailablePackageVersion(
    [string]$BaseVersion,
    [string]$PackageName
) {
    $versionBase = "$BaseVersion-dev-$(Get-Date -Format 'yyyyMMdd')"

    for ($revision = 0; $revision -lt 1000; $revision++) {
        $candidate = if ($revision -eq 0) { $versionBase } else { "$versionBase.$revision" }
        if (-not (Test-PublishedPackageVersion -PackageName $PackageName -Version $candidate)) {
            return $candidate
        }
    }
    throw "Failed to find an available package version after $versionBase.999."
}

function Invoke-NpmPack(
    [string]$StagePath,
    [string]$PackageName,
    [string]$Version,
    [string]$OutputPath
) {
    & npm pack $StagePath --pack-destination $OutputPath |
        ForEach-Object { Write-Host $_ }
    if ($LASTEXITCODE -ne 0) {
        throw "npm pack failed for $PackageName."
    }
    $archivePath = Get-PackageArchivePath `
        -PackageName $PackageName `
        -Version $Version `
        -OutputPath $OutputPath
    if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
        throw "Expected npm package was not created: '$archivePath'."
    }
    return $archivePath
}

function Build-PlatformPackage(
    [pscustomobject]$Definition,
    [string]$Version,
    [string]$CargoTargetPath,
    [string]$OutputPath
) {
    if ($Definition.Target -ne $script:localTarget) {
        throw "Target '$($Definition.Target)' cannot be built on '$script:localTarget' with the native-only build tools."
    }

    New-Item -ItemType Directory -Force -Path $CargoTargetPath | Out-Null
    $cargoTomlPath = Join-Path $script:repositoryRoot 'codex-rs/Cargo.toml'
    $cargoLockPath = Join-Path $script:repositoryRoot 'codex-rs/Cargo.lock'
    $originalCargoToml = [System.IO.File]::ReadAllBytes($cargoTomlPath)
    $cargoLockExisted = Test-Path -LiteralPath $cargoLockPath -PathType Leaf
    $originalCargoLock = if ($cargoLockExisted) {
        [System.IO.File]::ReadAllBytes($cargoLockPath)
    } else {
        $null
    }

    Write-Step "Temporarily setting the Cargo workspace version to $Version"
    try {
        $cargoTomlText = [System.Text.Encoding]::UTF8.GetString($originalCargoToml)
        $versionPattern = '(?ms)(?<prefix>\[workspace\.package\].*?^version\s*=\s*")(?<version>[^"]+)(?<suffix>")'
        $versionRegex = [regex]::new($versionPattern)
        $versionMatch = $versionRegex.Match($cargoTomlText)
        if (-not $versionMatch.Success) {
            throw 'Failed to find [workspace.package].version in codex-rs/Cargo.toml.'
        }
        if ($versionMatch.Groups['version'].Value -eq $Version) {
            Write-Step "Cargo workspace version is already $Version"
        } else {
            $updatedCargoToml = $versionRegex.Replace(
                $cargoTomlText,
                "`${prefix}$Version`${suffix}",
                1
            )
            [System.IO.File]::WriteAllText(
                $cargoTomlPath,
                $updatedCargoToml,
                [System.Text.UTF8Encoding]::new($false)
            )
        }

        $buildArguments = @('--codex', '--host', '--d', $CargoTargetPath)
        $buildProfile = 'debug'
        if ($script:ReleaseBuild) {
            $buildArguments = @('--codex', '--host', '--release', '--d', $CargoTargetPath)
            $buildProfile = 'release'
        }

        Write-Step "Building $($Definition.Target) in $buildProfile mode"
        & $script:buildScript @buildArguments |
            ForEach-Object { Write-Host $_ }
        if ($LASTEXITCODE -ne 0) {
            throw "$buildProfile build failed for $($Definition.Target)."
        }
    }
    finally {
        Write-Step 'Restoring Cargo.toml and Cargo.lock'
        [System.IO.File]::WriteAllBytes($cargoTomlPath, $originalCargoToml)
        if ($cargoLockExisted) {
            [System.IO.File]::WriteAllBytes($cargoLockPath, $originalCargoLock)
        } elseif (Test-Path -LiteralPath $cargoLockPath) {
            Remove-Item -LiteralPath $cargoLockPath -Force
        }
    }

    $buildOutputDirectory = Join-Path $CargoTargetPath $buildProfile
    $codexPath = Join-Path $buildOutputDirectory "codex$($Definition.ExecutableSuffix)"
    $hostPath = Join-Path $buildOutputDirectory "codex-code-mode-host$($Definition.ExecutableSuffix)"
    foreach ($executable in @($codexPath, $hostPath)) {
        if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
            throw "Expected build output not found: '$executable'."
        }
    }

    $reportedVersion = (& $codexPath --version).Trim()
    if ($LASTEXITCODE -ne 0 -or $reportedVersion -notlike "*$Version*") {
        throw "Built Codex reported '$reportedVersion', expected version '$Version'."
    }

    Write-Step "Staging $($Definition.PackageName)"
    $stagePath = Join-Path ([System.IO.Path]::GetTempPath()) (
        'cpa-platform-' + [guid]::NewGuid().ToString('N')
    )
    New-Item -ItemType Directory -Path $stagePath | Out-Null
    try {
        $nativePath = Join-Path $stagePath 'native'
        New-Item -ItemType Directory -Path $nativePath | Out-Null
        $stagedCodexPath = Join-Path $nativePath ([System.IO.Path]::GetFileName($codexPath))
        $stagedHostPath = Join-Path $nativePath ([System.IO.Path]::GetFileName($hostPath))
        Copy-Item -LiteralPath $codexPath -Destination $stagedCodexPath
        Copy-Item -LiteralPath $hostPath -Destination $stagedHostPath

        if ($script:ReleaseBuild -and $Definition.Os -eq 'linux') {
            if (-not (Get-Command strip -ErrorAction SilentlyContinue)) {
                throw "Required command 'strip' was not found in PATH."
            }
            Write-Step 'Stripping Linux release binaries for npm packaging'
            & strip --strip-all $stagedCodexPath $stagedHostPath
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to strip Linux release binaries."
            }
            $stagedVersion = (& $stagedCodexPath --version).Trim()
            if ($LASTEXITCODE -ne 0 -or $stagedVersion -notlike "*$Version*") {
                throw "Stripped Codex reported '$stagedVersion', expected version '$Version'."
            }
        }

        $platformManifest = [ordered]@{
            name = $Definition.PackageName
            version = $Version
            description = "CPA Codex native binaries for $($Definition.Target)"
            license = 'Apache-2.0'
            os = @($Definition.Os)
            cpu = @($Definition.Cpu)
            files = @('native')
        }
        $platformManifest | ConvertTo-Json -Depth 10 |
            Set-Content -LiteralPath (Join-Path $stagePath 'package.json') -Encoding utf8

        return Invoke-NpmPack `
            -StagePath $stagePath `
            -PackageName $Definition.PackageName `
            -Version $Version `
            -OutputPath $OutputPath
    }
    finally {
        if (Test-Path -LiteralPath $stagePath) {
            Remove-Item -LiteralPath $stagePath -Recurse -Force
        }
    }
}

function Build-LauncherPackage([string]$Version, [string]$OutputPath) {
    Write-Step 'Staging @pchuan98/cpa launcher package'
    $stagePath = Join-Path ([System.IO.Path]::GetTempPath()) (
        'cpa-launcher-' + [guid]::NewGuid().ToString('N')
    )
    New-Item -ItemType Directory -Path $stagePath | Out-Null
    try {
        Copy-Item -LiteralPath (Join-Path $script:npmDirectory 'package.json') -Destination $stagePath
        Copy-Item -LiteralPath (Join-Path $script:npmDirectory 'bin') -Destination $stagePath -Recurse

        $packageJsonPath = Join-Path $stagePath 'package.json'
        $packageJson = Get-Content -Raw -LiteralPath $packageJsonPath | ConvertFrom-Json
        $packageJson.version = $Version
        $packageJson.optionalDependencies = [ordered]@{
            '@pchuan98/cpa-win32-x64' = $Version
            '@pchuan98/cpa-linux-x64' = $Version
        }
        $packageJson | ConvertTo-Json -Depth 10 |
            Set-Content -LiteralPath $packageJsonPath -Encoding utf8

        return Invoke-NpmPack `
            -StagePath $stagePath `
            -PackageName '@pchuan98/cpa' `
            -Version $Version `
            -OutputPath $OutputPath
    }
    finally {
        if (Test-Path -LiteralPath $stagePath) {
            Remove-Item -LiteralPath $stagePath -Recurse -Force
        }
    }
}

function Write-PackManifest(
    [string]$Version,
    [array]$PackageRecords,
    [string]$OutputPath
) {
    $manifest = [ordered]@{
        version = $Version
        registry = $script:registry
        packages = @($PackageRecords)
    }
    $manifestPath = Join-Path $OutputPath 'pack-manifest.json'
    $manifest | ConvertTo-Json -Depth 10 |
        Set-Content -LiteralPath $manifestPath -Encoding utf8
    Write-Host "Package manifest: $manifestPath"
}

Write-Step 'Checking required commands'
foreach ($commandName in @('git', 'node', 'npm')) {
    if (-not (Get-Command $commandName -ErrorAction SilentlyContinue)) {
        throw "Required command '$commandName' was not found in PATH."
    }
}

$npmDirectory = Split-Path -Parent $PSCommandPath
$scriptsDirectory = Split-Path -Parent $npmDirectory
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $scriptsDirectory '../..'))
$buildScript = Join-Path $scriptsDirectory 'build.ps1'

$isWindowsPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)
$isLinuxPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Linux
)
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

if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $npmDirectory 'dist'
}
$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $outputPath | Out-Null

Push-Location $repositoryRoot
try {
    Write-Step 'Inspecting Git worktree and upstream remote'
    $worktreeState = (& git status --porcelain) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to inspect the Git worktree.'
    }
    if (-not [string]::IsNullOrWhiteSpace($worktreeState)) {
        Write-Warning 'The Git worktree is dirty. The package is intended for local verification.'
    }

    $officialUpstreamUrl = 'https://github.com/openai/codex'
    $remoteNames = @(& git remote)
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to inspect Git remotes.'
    }
    if ($remoteNames -notcontains 'upstream') {
        Write-Step "Adding missing upstream remote: $officialUpstreamUrl"
        & git remote add upstream $officialUpstreamUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to add the upstream remote '$officialUpstreamUrl'."
        }
        $upstreamUrl = $officialUpstreamUrl
    } else {
        $upstreamUrl = (& git remote get-url upstream).Trim()
        if ($LASTEXITCODE -ne 0) {
            throw 'Failed to read the upstream remote URL.'
        }
    }
    if (
        (ConvertTo-NormalizedRepositoryUrl $upstreamUrl) -ne
        (ConvertTo-NormalizedRepositoryUrl $officialUpstreamUrl)
    ) {
        throw "The upstream remote points to '$upstreamUrl'; expected '$officialUpstreamUrl'."
    }

    $selectedTarget = $localTarget
    $selectedDefinition = if ($OnlyPack) {
        $null
    } else {
        Get-PlatformDefinition -PlatformTarget $selectedTarget
    }

    if ([string]::IsNullOrWhiteSpace($PackageVersion)) {
        Write-Step 'Reading the current upstream npm version'
        $baseVersion = (& npm view '@openai/codex' version --registry $registry).Trim()
        if ($LASTEXITCODE -ne 0 -or $baseVersion -notmatch '^\d+\.\d+\.\d+$') {
            throw "The upstream npm version '$baseVersion' is not a stable semantic version."
        }
        $versionPackageName = if ($OnlyPack) {
            '@pchuan98/cpa'
        } else {
            $selectedDefinition.PackageName
        }
        $PackageVersion = Get-AvailablePackageVersion `
            -BaseVersion $baseVersion `
            -PackageName $versionPackageName
        Write-Step "Selected package version $PackageVersion for $versionPackageName"
    } elseif ($PackageVersion -notmatch '^\d+\.\d+\.\d+-[0-9A-Za-z.-]+$') {
        throw "Invalid package version '$PackageVersion'."
    } else {
        Write-Step "Using requested package version $PackageVersion"
    }

    $manifestPath = Join-Path $outputPath 'pack-manifest.json'
    if (Test-Path -LiteralPath $manifestPath) {
        Remove-Item -LiteralPath $manifestPath -Force
    }

    $packageRecords = @()
    if ($OnlyPack) {
        $launcherArchive = Build-LauncherPackage -Version $PackageVersion -OutputPath $outputPath
        $packageRecords += [ordered]@{
            name = '@pchuan98/cpa'
            version = $PackageVersion
            file = [System.IO.Path]::GetFileName($launcherArchive)
        }
    } else {
        $cargoTargetPath = if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
            Join-Path $repositoryRoot 'codex-rs/target'
        } else {
            [System.IO.Path]::GetFullPath($TargetDirectory)
        }
        $archivePath = Build-PlatformPackage `
            -Definition $selectedDefinition `
            -Version $PackageVersion `
            -CargoTargetPath $cargoTargetPath `
            -OutputPath $outputPath
        $packageRecords += [ordered]@{
            name = $selectedDefinition.PackageName
            version = $PackageVersion
            file = [System.IO.Path]::GetFileName($archivePath)
        }
    }

    Write-PackManifest `
        -Version $PackageVersion `
        -PackageRecords $packageRecords `
        -OutputPath $outputPath
    Write-Host "CPA package version: $PackageVersion"
    Write-Host "Package output: $outputPath"
}
finally {
    Pop-Location
}
