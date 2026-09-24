param([switch]$SkipInstall)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$dmdRoot = Split-Path $PSScriptRoot -Parent
$dmdFrontend = Join-Path $dmdRoot 'apps/desktop'
Push-Location $dmdFrontend
try {
    if (-not $SkipInstall) {
        & npm.cmd ci
        if ($LASTEXITCODE -ne 0) { throw 'Frontend dependency installation failed.' }
    }
    foreach ($task in @('check', 'test', 'build')) {
        & npm.cmd run $task
        if ($LASTEXITCODE -ne 0) { throw "Frontend $task failed." }
    }
    & (Join-Path $dmdFrontend 'node_modules/.bin/tauri.cmd') icon '../../crates/dmd-desktop/icons/app.svg' --output '../../crates/dmd-desktop/icons'
    if ($LASTEXITCODE -ne 0) { throw 'Desktop icon generation failed.' }
} finally { Pop-Location }
