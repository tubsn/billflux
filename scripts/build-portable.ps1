param(
    [switch]$KeepBuildCache
)

$ErrorActionPreference = 'Stop'
$project = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$bundle = Join-Path $project 'dist\Billflux'
$tools = Join-Path $project '.tools'
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
    cargo build --release --offline
    if ($LASTEXITCODE -ne 0) { throw 'Billflux-Build fehlgeschlagen' }
} finally {
    Pop-Location
}

foreach ($directory in @(
    $bundle,
    (Join-Path $bundle 'templates'),
    (Join-Path $bundle 'bin\gs'),
    (Join-Path $bundle 'bin\jre11'),
    (Join-Path $bundle 'bin\chrome-headless-shell'),
    (Join-Path $bundle 'database'),
    (Join-Path $bundle 'logs')
)) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }

Copy-Item -LiteralPath (Join-Path $project 'target\release\billflux.exe') -Destination (Join-Path $bundle 'Billflux.exe') -Force
$oldCli = Join-Path $bundle 'Billflux-CLI.exe'
if (Test-Path -LiteralPath $oldCli) { Remove-Item -LiteralPath $oldCli -Force }

$existingTemplates = @(Get-ChildItem -LiteralPath (Join-Path $bundle 'templates') -Directory | Where-Object {
    (Test-Path -LiteralPath (Join-Path $_.FullName 'invoice.html')) -and
    (Test-Path -LiteralPath (Join-Path $_.FullName 'style.css'))
})
$seedTemplates = if ($existingTemplates.Count -eq 0) { @('standard', 'example') } else { @() }
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

foreach ($template in $seedTemplates) {
    foreach ($family in @('Fira_Sans', 'Fira_Sans_Condensed')) {
        $source = Join-Path $project "fonts\$family"
        $destination = Join-Path $bundle "templates\$template\fonts\$family"
        New-Item -ItemType Directory -Force -Path $destination | Out-Null
        foreach ($file in @('OFL.txt')) {
            $target = Join-Path $destination $file
            if (-not (Test-Path -LiteralPath $target)) {
                Copy-Item -LiteralPath (Join-Path $source $file) -Destination $target
            }
        }
        Get-ChildItem -LiteralPath $source -File | Where-Object { $_.Name -match '(-Regular|-Bold)\.ttf$' } | ForEach-Object {
            $target = Join-Path $destination $_.Name
            if (-not (Test-Path -LiteralPath $target)) {
                Copy-Item -LiteralPath $_.FullName -Destination $target
            }
        }
    }
}

Copy-Item -LiteralPath (Join-Path $tools 'Mustang-CLI-2.26.0.jar') -Destination (Join-Path $bundle 'bin') -Force
Copy-Item -LiteralPath (Join-Path $tools 'PDFA_def.ps') -Destination (Join-Path $bundle 'bin') -Force
Copy-Item -LiteralPath (Join-Path $tools 'srgb.icc') -Destination (Join-Path $bundle 'bin') -Force
Copy-Item -LiteralPath (Join-Path $tools 'gs\Library') -Destination (Join-Path $bundle 'bin\gs') -Recurse -Force
Get-ChildItem -LiteralPath (Join-Path $tools 'jre11') -Directory | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $bundle 'bin\jre11') -Recurse -Force
}
$rendererDestination = Join-Path $bundle 'bin\chrome-headless-shell'
Get-ChildItem -LiteralPath $renderer -Force | Copy-Item -Destination $rendererDestination -Recurse -Force
if (-not (Test-Path -LiteralPath (Join-Path $rendererDestination 'chrome-headless-shell.exe') -PathType Leaf)) {
    throw 'Chrome Headless Shell fehlt in der Distribution'
}
$oldChrome = Join-Path $bundle 'bin\chrome'
if (-not ([IO.Path]::GetFullPath($oldChrome)).StartsWith(([IO.Path]::GetFullPath($bundle)).TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Ungültiger Chrome-Zielpfad'
}
if (Test-Path -LiteralPath $oldChrome) { Remove-Item -LiteralPath $oldChrome -Recurse -Force }

if (-not $KeepBuildCache) {
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
