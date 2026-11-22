$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

# Fecha instancia anterior se estiver aberta
Stop-Process -Name "Aster" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "aster-companion" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# Garante cargo no PATH se necessario
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        $env:PATH = "$cargoBin;$env:PATH"
    }
}

# Compila em %TEMP% para evitar lock do Windows Defender/OneDrive no Desktop (os error 32 / 5)
if (-not $env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR = Join-Path $env:TEMP "aster-target"
}
$env:CARGO_BUILD_JOBS = "1"
Write-Host "Compilando Aster.exe (release) em $env:CARGO_TARGET_DIR..." -ForegroundColor Cyan

$maxTries = 4
$ok = $false
for ($try = 1; $try -le $maxTries; $try++) {
    npx tauri build --no-bundle
    if ($LASTEXITCODE -eq 0) {
        $ok = $true
        break
    }
    if ($try -lt $maxTries) {
        Write-Host "Lock temporario no build (tentativa $try/$maxTries). Aguardando 2s e continuando incrementalmente..." -ForegroundColor Yellow
        Start-Sleep -Seconds 2
    }
}
if (-not $ok) {
    throw "Falha ao compilar Aster.exe (exit code $LASTEXITCODE)"
}

$builtExe = Join-Path $env:CARGO_TARGET_DIR "release\aster-companion.exe"
if (-not (Test-Path $builtExe)) {
    $builtExe = ".\src-tauri\target\release\aster-companion.exe"
}
if (-not (Test-Path $builtExe)) {
    throw "Executavel nao encontrado em $builtExe"
}

# Fecha instancia aberta logo antes de sobrescrever o Aster.exe
Stop-Process -Name "Aster" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "aster-companion" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400

Copy-Item -Force $builtExe ".\Aster.exe"
Write-Host ""
Write-Host "Pronto! Executavel criado em: $((Resolve-Path '.\Aster.exe').Path)" -ForegroundColor Green
