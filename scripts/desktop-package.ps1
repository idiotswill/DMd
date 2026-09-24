param([string]$OutputDirectory = '')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$dmdRoot = Split-Path $PSScriptRoot -Parent
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $dmdRoot 'artifacts/desktop' }
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a new, empty package output directory.' }
$dmdTarget = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $dmdRoot 'target' }
$dmdRelease = Join-Path $dmdTarget 'x86_64-pc-windows-msvc/release'
$dmdExe = Join-Path $dmdRelease 'dmd-desktop.exe'
if (-not (Test-Path -LiteralPath $dmdExe)) { throw 'Build the Windows release executable first.' }
$dmdInstallers = @(Get-ChildItem -LiteralPath (Join-Path $dmdRelease 'bundle/nsis') -Filter '*-setup.exe')
if ($dmdInstallers.Count -ne 1) { throw 'Expected exactly one NSIS installer from this build.' }
Push-Location $dmdRoot
try {
    $dmdCommit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot identify packaged source commit.' }
    & git diff --quiet HEAD
    if ($LASTEXITCODE -ne 0) { throw 'Tracked source changes must be committed before packaging.' }
    $dmdPortable = Join-Path $OutputDirectory 'portable'
    New-Item -ItemType Directory -Path (Join-Path $dmdPortable 'content'), (Join-Path $dmdPortable 'licenses') -Force | Out-Null
    Copy-Item -LiteralPath $dmdExe -Destination (Join-Path $dmdPortable 'DMd.exe')
    Copy-Item -LiteralPath (Join-Path $dmdRoot 'content/srd-5.2.1') -Destination (Join-Path $dmdPortable 'content') -Recurse
    Copy-Item -LiteralPath (Join-Path $dmdRoot 'LICENSE') -Destination (Join-Path $dmdPortable 'licenses/DMd-LICENSE.txt')
    Copy-Item -LiteralPath (Join-Path $dmdRoot 'crates/dmd-desktop/licenses/dependencies') -Destination (Join-Path $dmdPortable 'licenses') -Recurse
    Copy-Item -LiteralPath $dmdInstallers[0].FullName -Destination $OutputDirectory
    $dmdBuild = [ordered]@{
        commit = $dmdCommit
        target = 'x86_64-pc-windows-msvc'
        profile = 'release'
        rust = (& rustc --version)
        node = (& node --version)
        built_at_utc = [DateTime]::UtcNow.ToString('o')
        installer = $dmdInstallers[0].Name
    }
    $dmdBuild | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory 'build-info.json') -Encoding utf8
    Copy-Item -LiteralPath (Join-Path $dmdRoot 'docs/runbooks/desktop-package-readme.txt') -Destination (Join-Path $OutputDirectory 'README.txt')
    $dmdOutputRoot = (Resolve-Path -LiteralPath $OutputDirectory).Path
    $dmdChecksums = Get-ChildItem -LiteralPath $dmdOutputRoot -File -Recurse | Sort-Object FullName | ForEach-Object {
        $relative = [IO.Path]::GetRelativePath($dmdOutputRoot, $_.FullName).Replace('\', '/')
        $digest = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$digest  $relative"
    }
    $dmdChecksums | Set-Content -LiteralPath (Join-Path $OutputDirectory 'SHA256SUMS.txt') -Encoding utf8
} finally { Pop-Location }
