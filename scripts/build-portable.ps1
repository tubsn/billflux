param(
    [string]$ChromeSource = 'C:\Program Files\Google\Chrome\Application',
    [switch]$KeepBuildCache
)

$ErrorActionPreference = 'Stop'
$project = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$bundle = Join-Path $project 'dist\Billflux'
$tools = Join-Path $project '.tools'

foreach ($path in @(
    (Join-Path $tools 'Mustang-CLI-2.26.0.jar'),
    (Join-Path $tools 'PDFA_def.ps'),
    (Join-Path $tools 'srgb.icc'),
    (Join-Path $tools 'gs\Library\bin\gswin64c.exe'),
    (Join-Path $ChromeSource 'chrome.exe')
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
    (Join-Path $bundle 'templates\standard\fonts\Fira_Sans'),
    (Join-Path $bundle 'templates\standard\fonts\Fira_Sans_Condensed'),
    (Join-Path $bundle 'templates\example\fonts\Fira_Sans'),
    (Join-Path $bundle 'templates\example\fonts\Fira_Sans_Condensed'),
    (Join-Path $bundle 'vendor\gs'),
    (Join-Path $bundle 'vendor\jre11'),
    (Join-Path $bundle 'vendor\chrome'),
    (Join-Path $bundle 'database'),
    (Join-Path $bundle 'output')
)) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }

Copy-Item -LiteralPath (Join-Path $project 'target\release\billflux.exe') -Destination (Join-Path $bundle 'Billflux.exe') -Force
$oldCli = Join-Path $bundle 'Billflux-CLI.exe'
if (Test-Path -LiteralPath $oldCli) { Remove-Item -LiteralPath $oldCli -Force }

foreach ($template in @('standard', 'example')) {
    foreach ($file in @('invoice.html', 'style.css')) {
        $source = Join-Path $project "templates\$template\$file"
        $destination = Join-Path $bundle "templates\$template\$file"
        if (-not (Test-Path -LiteralPath $destination)) {
            Copy-Item -LiteralPath $source -Destination $destination
        }
    }
}

foreach ($template in @('standard', 'example')) {
    foreach ($family in @('Fira_Sans', 'Fira_Sans_Condensed')) {
        $source = Join-Path $project "fonts\$family"
        $destination = Join-Path $bundle "templates\$template\fonts\$family"
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

Copy-Item -LiteralPath (Join-Path $tools 'Mustang-CLI-2.26.0.jar') -Destination (Join-Path $bundle 'vendor') -Force
Copy-Item -LiteralPath (Join-Path $tools 'PDFA_def.ps') -Destination (Join-Path $bundle 'vendor') -Force
Copy-Item -LiteralPath (Join-Path $tools 'srgb.icc') -Destination (Join-Path $bundle 'vendor') -Force
Copy-Item -LiteralPath (Join-Path $tools 'gs\Library') -Destination (Join-Path $bundle 'vendor\gs') -Recurse -Force
Get-ChildItem -LiteralPath (Join-Path $tools 'jre11') -Directory | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $bundle 'vendor\jre11') -Recurse -Force
}
Copy-Item -Path (Join-Path $ChromeSource '*') -Destination (Join-Path $bundle 'vendor\chrome') -Recurse -Force

if (-not $KeepBuildCache) {
    Push-Location $project
    try { cargo clean } finally { Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw 'Distribution wurde gebaut, aber der Build-Cache konnte nicht entfernt werden' }
    $generated = Join-Path $project 'gen'
    if (Test-Path -LiteralPath $generated) { Remove-Item -LiteralPath $generated -Recurse -Force }
}

Write-Host "Portabler Ordner: $bundle"
