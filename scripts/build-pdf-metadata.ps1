param([string]$Javac = 'javac')
$ErrorActionPreference = 'Stop'
$project = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
& $Javac --release 11 -encoding UTF-8 -cp (Join-Path $project '.tools/Mustang-CLI-2.26.0.jar') -d (Join-Path $project 'src/java') (Join-Path $project 'src/java/BillfluxPdfMetadata.java')
if ($LASTEXITCODE -ne 0) { throw 'Metadaten-Helfer konnte nicht kompiliert werden' }
