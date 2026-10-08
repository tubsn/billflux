param(
    [string]$ToolsDirectory = (Join-Path (Split-Path $PSScriptRoot -Parent) '.tools')
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$version = '154.0.8037.57'
$expectedHash = 'BCC91B4D0F83A5457FC6CA7941FD65F2349775523D961BE88526C6A66560DC75'
$url = "https://storage.googleapis.com/chrome-for-testing-public/$version/win64/chrome-headless-shell-win64.zip"
$archive = Join-Path $ToolsDirectory 'chrome-headless-shell-win64.zip'
$destination = Join-Path $ToolsDirectory 'chrome-headless-shell'
$executable = Join-Path $destination 'chrome-headless-shell-win64/chrome-headless-shell.exe'

if (Test-Path -LiteralPath $executable -PathType Leaf) { return }
New-Item -ItemType Directory -Path $ToolsDirectory -Force | Out-Null

if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) {
    Write-Host "Lade Chrome Headless Shell $version herunter ..."
    Invoke-WebRequest -Uri $url -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expectedHash) {
    throw 'Der SHA-256-Hash des Chrome-Headless-Shell-Archivs stimmt nicht.'
}
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw 'Chrome Headless Shell wurde nicht korrekt entpackt.'
}
Remove-Item -LiteralPath $archive -Force
