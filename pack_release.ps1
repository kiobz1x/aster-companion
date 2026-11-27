param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

if (-not $Version) {
    $pkg = Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json
    $Version = $pkg.version
}

$exePath = Join-Path $root "Aster.exe"
if (-not (Test-Path $exePath)) {
    throw "Aster.exe nao encontrado na raiz. Rode 'npm run build:exe' primeiro."
}

$stageRoot = Join-Path $env:TEMP "aster-pack-stage"
$bundleDir = Join-Path $stageRoot "Aster-Companion"
if (Test-Path $stageRoot) { Remove-Item $stageRoot -Recurse -Force }
New-Item -ItemType Directory -Path (Join-Path $bundleDir "skins\natalina") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $bundleDir "docs") -Force | Out-Null

Copy-Item $exePath $bundleDir
Copy-Item (Join-Path $root "Instalar_Extras.bat") $bundleDir
Copy-Item (Join-Path $root "COMANDOS.md") $bundleDir
Copy-Item (Join-Path $root "transcrever.py") $bundleDir
Copy-Item (Join-Path $root "processar_skin.py") $bundleDir
Copy-Item (Join-Path $root "requirements.txt") $bundleDir
Copy-Item (Join-Path $root "skins\natalina\*.png") (Join-Path $bundleDir "skins\natalina")

@"
Coloque aqui arquivos .txt, .md, .json, .csv ou .log com anotacoes ou informacoes que voce quer que a Aster consulte automaticamente quando voce perguntar sobre o assunto.
"@ | Set-Content -Path (Join-Path $bundleDir "docs\LEIA-ME.txt") -Encoding UTF8

$zipName = "Aster-Companion-v$Version-win64.zip"
$zipPath = Join-Path $root $zipName
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }

Compress-Archive -Path $bundleDir -DestinationPath $zipPath -CompressionLevel Optimal
Remove-Item $stageRoot -Recurse -Force

$sizeMb = [math]::Round((Get-Item $zipPath).Length / 1MB, 2)
Write-Host "Pronto! Pacote criado: $zipPath ($sizeMb MB)"
