param(
    [switch]$KeepBuildCache,
    [switch]$CleanBuildCache
)

$ErrorActionPreference = 'Stop'
if ($KeepBuildCache -and $CleanBuildCache) { throw 'Bitte nur eine Cache-Option angeben.' }
$project = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$bundle = Join-Path $project 'dist\Billflux'
$tools = Join-Path $project '.tools'
function Copy-ChangedTree($Source, $Destination) {
    $sourceRoot = (Resolve-Path -LiteralPath $Source).Path.TrimEnd('\')
    foreach ($file in Get-ChildItem -LiteralPath $sourceRoot -Recurse -File) {
        $relative = $file.FullName.Substring($sourceRoot.Length).TrimStart('\')
        $target = Join-Path $Destination $relative
        $current = Get-Item -LiteralPath $target -ErrorAction SilentlyContinue
        if (-not $current -or $current.Length -ne $file.Length -or $current.LastWriteTimeUtc -lt $file.LastWriteTimeUtc) {
            New-Item -ItemType Directory -Force -Path (Split-Path $target -Parent) | Out-Null
            Copy-Item -LiteralPath $file.FullName -Destination $target -Force
        }
    }
}
Write-Host '[1/3] Benötigte Werkzeuge prüfen ...'
& (Join-Path $PSScriptRoot 'install-renderer.ps1') -ToolsDirectory $tools
$renderer = Join-Path $tools 'chrome-headless-shell\chrome-headless-shell-win64'

foreach ($path in @(
    (Join-Path $tools 'Mustang-CLI-2.26.0.jar'),
    (Join-Path $tools 'PDFA_def.ps'),
    (Join-Path $tools 'srgb.icc'),
    (Join-Path $tools 'gs\Library\bin\gswin64c.exe'),
    (Join-Path $renderer 'chrome-headless-shell.exe')
)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Datei fehlt: $path" }
}

Push-Location $project
try {
    Write-Host '[2/3] App bauen (Cargo verwendet den vorhandenen Build-Cache) ...'
    cargo build --release --offline
    if ($LASTEXITCODE -ne 0) { throw 'Billflux-Build fehlgeschlagen' }
} finally {
    Pop-Location
}

Write-Host '[3/3] Portable App zusammenstellen ...'
foreach ($directory in @(
    $bundle,
    (Join-Path $bundle 'templates'),
    (Join-Path $bundle 'bin\gs'),
    (Join-Path $bundle 'bin\jre11'),
    (Join-Path $bundle 'bin\chrome-headless-shell')
)) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }

try {
    Copy-Item -LiteralPath (Join-Path $project 'target\release\billflux.exe') -Destination (Join-Path $bundle 'Billflux.exe') -Force
} catch {
    throw 'Billflux.exe im Dist-Ordner ist noch geöffnet. Bitte die App schließen und run-build.cmd erneut starten.'
}
Copy-Item -LiteralPath (Join-Path $project 'LICENSE.md') -Destination (Join-Path $bundle 'LICENSE.md') -Force
Copy-Item -LiteralPath (Join-Path $project 'assets\licenses\AGPL-3.0.txt') -Destination (Join-Path $bundle 'bin\gs\AGPL-3.0.txt') -Force
Copy-Item -LiteralPath (Join-Path $project 'assets\fonts\Fira_Sans\OFL.txt') -Destination (Join-Path $bundle 'bin\OFL-FiraSans.txt') -Force
Copy-Item -LiteralPath (Join-Path $project 'assets\fonts\Fira_Sans_Condensed\OFL.txt') -Destination (Join-Path $bundle 'bin\OFL-FiraSansCondensed.txt') -Force
$oldCli = Join-Path $bundle 'Billflux-CLI.exe'
if (Test-Path -LiteralPath $oldCli) { Remove-Item -LiteralPath $oldCli -Force }

$exampleTemplate = Join-Path $bundle 'templates\example'
$seedTemplates = if ((Test-Path -LiteralPath (Join-Path $exampleTemplate 'invoice.html')) -and
    (Test-Path -LiteralPath (Join-Path $exampleTemplate 'style.css'))) { @() } else { @('example') }
foreach ($template in $seedTemplates) {
    New-Item -ItemType Directory -Force -Path (Join-Path $bundle "templates\$template") | Out-Null
    foreach ($file in @('invoice.html', 'style.css')) {
        $source = Join-Path $project "templates\$template\$file"
        $destination = Join-Path $bundle "templates\$template\$file"
        if (-not (Test-Path -LiteralPath $destination)) {
            Copy-Item -LiteralPath $source -Destination $destination
        }
    }
}

Copy-Item -LiteralPath (Join-Path $tools 'Mustang-CLI-2.26.0.jar') -Destination (Join-Path $bundle 'bin') -Force
Copy-Item -LiteralPath (Join-Path $tools 'PDFA_def.ps') -Destination (Join-Path $bundle 'bin') -Force
Copy-Item -LiteralPath (Join-Path $tools 'srgb.icc') -Destination (Join-Path $bundle 'bin') -Force
Copy-ChangedTree (Join-Path $tools 'gs\Library') (Join-Path $bundle 'bin\gs\Library')
Copy-ChangedTree (Join-Path $tools 'jre11') (Join-Path $bundle 'bin\jre11')
$rendererDestination = Join-Path $bundle 'bin\chrome-headless-shell'
Copy-ChangedTree $renderer $rendererDestination
if (-not (Test-Path -LiteralPath (Join-Path $rendererDestination 'chrome-headless-shell.exe') -PathType Leaf)) {
    throw 'Chrome Headless Shell fehlt in der Distribution'
}
$oldChrome = Join-Path $bundle 'bin\chrome'
if (-not ([IO.Path]::GetFullPath($oldChrome)).StartsWith(([IO.Path]::GetFullPath($bundle)).TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Ungültiger Chrome-Zielpfad'
}
if (Test-Path -LiteralPath $oldChrome) { Remove-Item -LiteralPath $oldChrome -Recurse -Force }

if ($CleanBuildCache) {
    Push-Location $project
    try { cargo clean } finally { Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw 'Distribution wurde gebaut, aber der Build-Cache konnte nicht entfernt werden' }
    $generated = Join-Path $project 'gen'
    if (-not ([IO.Path]::GetFullPath($generated)).StartsWith(([IO.Path]::GetFullPath($project)).TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Ungültiger Schema-Zielpfad'
    }
    if (Test-Path -LiteralPath $generated) { Remove-Item -LiteralPath $generated -Recurse -Force }
}

Write-Host "Portabler Ordner: $bundle"
