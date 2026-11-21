$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

# Fecha instancia anterior se estiver aberta
Stop-Process -Name "Aster" -ErrorAction SilentlyContinue
Stop-Process -Name "aster-companion" -ErrorAction SilentlyContinue

# Garante cargo no PATH se necessario
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        $env:PATH = "$cargoBin;$env:PATH"
    }
}

Write-Host "Compilando Aster.exe (release)..." -ForegroundColor Cyan
npx tauri build --no-bundle

$builtExe = ".\src-tauri\target\release\aster-companion.exe"
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
