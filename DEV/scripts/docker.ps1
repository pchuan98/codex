[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('build', 'pack')]
    [string]$Operation,

    [Parameter(Mandatory = $true)]
    [ValidateSet('linux-x64', 'linux-arm64')]
    [string]$Target,

    [string]$TargetDirectory,
    [string]$OutputDirectory,
    [switch]$BuildCodex,
    [switch]$BuildHost,
    [switch]$ReleaseBuild,
    [string]$PackageVersion,
    [switch]$PlatformOnly
)

$ErrorActionPreference = 'Stop'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Restore-WindowsMigrationLineEndings([string]$RepositoryRoot) {
    if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows
    )) {
        return
    }

    Write-Step 'Restoring CRLF migrations for the Windows worktree'
    $migrationFiles = Get-ChildItem -LiteralPath (Join-Path $RepositoryRoot 'codex-rs/state') `
        -Recurse -File -Filter '*.sql' |
        Where-Object { $_.Directory.Name -like '*migrations' }
    foreach ($migrationFile in $migrationFiles) {
        $originalBytes = [System.IO.File]::ReadAllBytes($migrationFile.FullName)
        $migrationText = [System.Text.Encoding]::UTF8.GetString($originalBytes)
        $migrationText = $migrationText.Replace("`r`n", "`n").Replace("`r", "`n")
        $migrationText = $migrationText.Replace("`n", "`r`n")
        $normalizedBytes = [System.Text.UTF8Encoding]::new($false).GetBytes($migrationText)
        if (
            [Convert]::ToBase64String($originalBytes) -ne
            [Convert]::ToBase64String($normalizedBytes)
        ) {
            [System.IO.File]::WriteAllBytes($migrationFile.FullName, $normalizedBytes)
        }
    }

    $gitDirectoryOutput = & git -C $RepositoryRoot rev-parse --git-dir 2>$null
    if ($LASTEXITCODE -ne 0) {
        return
    }
    $gitDirectory = [System.IO.Path]::GetFullPath(
        (Join-Path $RepositoryRoot ($gitDirectoryOutput.Trim()))
    )
    $attributesPath = Join-Path $gitDirectory 'info/attributes'
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
    $attributesBlock = @"
# codex-cpa-build:start
codex-rs/state/**/*.sql text eol=crlf
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

    $migrationPaths = $migrationFiles | ForEach-Object {
        [System.IO.Path]::GetRelativePath($RepositoryRoot, $_.FullName).Replace('\', '/')
    }
    & git -C $RepositoryRoot add --refresh -- @migrationPaths
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to refresh the Git index after restoring Windows migrations.'
    }
}

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    throw "Required command 'docker' was not found in PATH."
}

& docker info --format '{{.ServerVersion}}' | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw 'Docker is not running. Start Docker Desktop and try again.'
}

$scriptDirectory = Split-Path -Parent $PSCommandPath
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $scriptDirectory '../..'))
$dockerfilePath = Join-Path $scriptDirectory 'Dockerfile.linux-build'

if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
    $TargetDirectory = Join-Path $repositoryRoot "codex-rs/target/docker/$Target"
}
$targetPath = [System.IO.Path]::GetFullPath($TargetDirectory)
New-Item -ItemType Directory -Force -Path $targetPath | Out-Null
$cargoRegistryPath = Join-Path $targetPath '.cargo-cache/registry'
$cargoGitPath = Join-Path $targetPath '.cargo-cache/git'
New-Item -ItemType Directory -Force -Path $cargoRegistryPath | Out-Null
New-Item -ItemType Directory -Force -Path $cargoGitPath | Out-Null

if ($Operation -eq 'pack') {
    if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
        $OutputDirectory = Join-Path $repositoryRoot 'DEV/scripts/npm/dist'
    }
    $outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
    New-Item -ItemType Directory -Force -Path $outputPath | Out-Null
}

$dockerPlatform = switch ($Target) {
    'linux-x64' { 'linux/amd64' }
    'linux-arm64' { 'linux/arm64' }
}

if ($Target -eq 'linux-arm64') {
    $builderInfo = (& docker buildx inspect --bootstrap) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to inspect Docker Buildx platforms.'
    }
    if ($builderInfo -notmatch '(?m)^Platforms:.*\blinux/arm64\b') {
        Write-Step 'Installing Docker ARM64 emulation support'
        & docker run --privileged --rm tonistiigi/binfmt:latest --install arm64
        if ($LASTEXITCODE -ne 0) {
            throw 'Failed to install Docker ARM64 emulation support.'
        }
        $builderInfo = (& docker buildx inspect --bootstrap) -join "`n"
        if ($LASTEXITCODE -ne 0 -or $builderInfo -notmatch '(?m)^Platforms:.*\blinux/arm64\b') {
            throw 'Docker Buildx still does not report linux/arm64 support.'
        }
    }
}

$dockerfileHash = (Get-FileHash -LiteralPath $dockerfilePath -Algorithm SHA256).Hash.ToLowerInvariant()
$imageName = "cpa-linux-build:$Target-$($dockerfileHash.Substring(0, 12))"

& docker image inspect $imageName *> $null
if ($LASTEXITCODE -ne 0) {
    Write-Step "Pulling Debian base image for $dockerPlatform"
    & docker pull --platform $dockerPlatform debian:bookworm-slim
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to pull the Docker base image.'
    }

    Write-Step "Configuring reusable build image $imageName"
    & docker build `
        --platform $dockerPlatform `
        --file $dockerfilePath `
        --tag $imageName `
        $scriptDirectory
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to build the Docker build image.'
    }
} else {
    Write-Step "Using cached build image $imageName"
}

$dockerArguments = @(
    'run',
    '--rm',
    '--platform', $dockerPlatform,
    '--mount', "type=bind,source=$repositoryRoot,target=/workspace",
    '--mount', "type=bind,source=$targetPath,target=/target",
    '--mount', "type=bind,source=$cargoRegistryPath,target=/root/.cargo/registry",
    '--mount', "type=bind,source=$cargoGitPath,target=/root/.cargo/git"
)

if ($Operation -eq 'pack') {
    $dockerArguments += @('--mount', "type=bind,source=$outputPath,target=/output")
}

$dockerArguments += @(
    '--env', 'npm_config_registry=https://registry.npmjs.org/',
    $imageName,
    'pwsh', '-NoLogo', '-NoProfile', '-Command'
)

if ($Operation -eq 'build') {
    $commandParts = @("& '/workspace/DEV/scripts/build.ps1'")
    if ($BuildCodex) {
        $commandParts += '--codex'
    }
    if ($BuildHost) {
        $commandParts += '--host'
    }
    if ($ReleaseBuild) {
        $commandParts += '--release'
    }
    $commandParts += @('--d', "'/target'")
} else {
    $commandParts = @(
        "& '/workspace/DEV/scripts/npm/pack.ps1'",
        '-TargetDirectory', "'/target'",
        '-OutputDirectory', "'/output'"
    )
    if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
        $commandParts += @('-PackageVersion', "'$PackageVersion'")
    }
    if ($PlatformOnly) {
        $commandParts += @('-Target', "'$Target'", '-PlatformOnly')
    }
    if ($ReleaseBuild) {
        $commandParts += '-ReleaseBuild'
    }
}
$dockerArguments += ($commandParts -join ' ')

Write-Step "Running $Operation for $Target in Docker"
try {
    & docker @dockerArguments
    $dockerExitCode = $LASTEXITCODE
}
finally {
    Restore-WindowsMigrationLineEndings -RepositoryRoot $repositoryRoot
}
if ($dockerExitCode -ne 0) {
    throw "Docker $Operation failed with exit code $dockerExitCode."
}
