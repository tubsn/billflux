param(
    [string]$ChromeSource = 'C:\Program Files\Google\Chrome\Application'
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
    if ($LASTEXITCODE -ne 0) { throw 'Cargo-Build fehlgeschlagen' }
    cargo build --release --offline --manifest-path desktop/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Desktop-Build fehlgeschlagen' }
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

Copy-Item -LiteralPath (Join-Path $project 'desktop\target\release\billflux-desktop.exe') -Destination (Join-Path $bundle 'Billflux.exe') -Force
Copy-Item -LiteralPath (Join-Path $project 'target\release\billflux-prototype.exe') -Destination (Join-Path $bundle 'Billflux-CLI.exe') -Force
Copy-Item -LiteralPath (Join-Path $project 'templates\standard\invoice.html') -Destination (Join-Path $bundle 'templates\standard\invoice.html') -Force
Copy-Item -LiteralPath (Join-Path $project 'templates\standard\style.css') -Destination (Join-Path $bundle 'templates\standard\style.css') -Force
Copy-Item -LiteralPath (Join-Path $project 'templates\example\invoice.html') -Destination (Join-Path $bundle 'templates\example\invoice.html') -Force
Copy-Item -LiteralPath (Join-Path $project 'templates\example\style.css') -Destination (Join-Path $bundle 'templates\example\style.css') -Force

foreach ($template in @('standard', 'example')) {
foreach ($family in @('Fira_Sans', 'Fira_Sans_Condensed')) {
    $source = Join-Path $project "fonts\$family"
    $destination = Join-Path $bundle "templates\$template\fonts\$family"
    Copy-Item -LiteralPath (Join-Path $source 'OFL.txt') -Destination $destination -Force
    Get-ChildItem -LiteralPath $source -File | Where-Object { $_.Name -match '(-Regular|-Bold)\.ttf$' } | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination $destination -Force
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

Write-Host "Portabler Ordner: $bundle"
