# Forward all arguments to the newly built Debug binary through the CPA launcher.
$ErrorActionPreference = 'Stop'
$runArguments = $args
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    throw "Required command 'node' was not found in PATH."
}
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$targetPath = if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
    Join-Path $repositoryRoot 'codex-rs/target'
} else {
    [System.IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
}
& (Join-Path $PSScriptRoot 'build.ps1') --codex --host --d $targetPath
if ($LASTEXITCODE -ne 0) { throw 'Debug build failed.' }
$suffix = if ($IsWindows) { '.exe' } else { '' }
$outputPath = $targetPath
if ($IsLinux) {
    $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    $rustTarget = if ($architecture -eq 'X64') { 'x86_64-unknown-linux-musl' } else { 'aarch64-unknown-linux-musl' }
    $outputPath = Join-Path $targetPath $rustTarget
}
$codexPath = Join-Path $outputPath "debug/codex$suffix"
& node (Join-Path $PSScriptRoot 'run.mjs') $codexPath @runArguments
exit $LASTEXITCODE
