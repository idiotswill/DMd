param([string]$OutputDirectory = '')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$dmdRoot = Split-Path $PSScriptRoot -Parent
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $dmdRoot 'artifacts/desktop' }
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a new, empty package output directory.' }
Push-Location $dmdRoot
try {
    $dmdCommit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot identify packaged source commit.' }
    $dmdChanges = @(& git status --porcelain --untracked-files=normal)
    if ($LASTEXITCODE -ne 0 -or $dmdChanges.Count -ne 0) { throw 'Commit source changes before building a package.' }

    # Packaging always builds from this clean head. It cannot relabel binaries left by
    # another branch, a failed build or an earlier invocation of desktop-build.
    & (Join-Path $PSScriptRoot 'desktop-build.ps1')

    $dmdAfterBuild = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $dmdAfterBuild -ne $dmdCommit) { throw 'Source commit changed during the build; package cancelled.' }
    $dmdChanges = @(& git status --porcelain --untracked-files=normal)
    if ($LASTEXITCODE -ne 0 -or $dmdChanges.Count -ne 0) {
        $dmdChanges | Write-Output
        & git diff --stat
        & git diff -- crates/dmd-desktop/Cargo.toml
        throw 'Source changed during the build; package cancelled.'
    }
    $dmdTarget = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $dmdRoot 'target' }
    $dmdRelease = Join-Path $dmdTarget 'x86_64-pc-windows-msvc/release'
    $dmdExe = Join-Path $dmdRelease 'dmd-desktop.exe'
    if (-not (Test-Path -LiteralPath $dmdExe)) { throw 'The fresh build did not produce the Windows executable.' }
    $dmdInstallers = @(Get-ChildItem -LiteralPath (Join-Path $dmdRelease 'bundle/nsis') -Filter '*-setup.exe')
    if ($dmdInstallers.Count -ne 1) { throw 'Expected exactly one NSIS installer from this build.' }
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
