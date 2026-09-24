param([switch]$NoBundle, [switch]$SkipPrepare)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'Desktop packaging currently requires Windows with MSVC.' }
$dmdRoot = Split-Path $PSScriptRoot -Parent
if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR, $dmdRoot) }
if (-not $SkipPrepare) { & (Join-Path $PSScriptRoot 'desktop-prepare.ps1') }
$dmdCli = Join-Path $dmdRoot 'apps/desktop/node_modules/.bin/tauri.cmd'
Push-Location $dmdRoot
try {
    & node (Join-Path $PSScriptRoot 'desktop-notices.mjs') (Join-Path $dmdRoot 'crates/dmd-desktop/licenses/dependencies')
    if ($LASTEXITCODE -ne 0) { throw 'Dependency notice collection failed.' }
} finally { Pop-Location }
Push-Location (Join-Path $dmdRoot 'crates/dmd-desktop')
try {
    $dmdArguments = @('build', '--ci', '--target', 'x86_64-pc-windows-msvc')
    if ($NoBundle) { $dmdArguments += '--no-bundle' }
    else { $dmdArguments += @('--bundles', 'nsis') }
    $dmdArguments += @('--', '--locked')
    & $dmdCli @dmdArguments
    if ($LASTEXITCODE -ne 0) { throw 'Windows desktop build failed.' }
} finally { Pop-Location }
