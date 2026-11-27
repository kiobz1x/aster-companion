@echo off
chcp 65001 >nul
title Aster Companion - Assistente de Instalacao
color 0D
cd /d "%~dp0"

:MENU
cls
echo.
echo  ============================================================
echo     *   A S T E R   C O M P A N I O N   *
echo         Assistente de Instalacao e Extras (Windows 11)
echo  ============================================================
echo.
echo   Este arquivo e seguro e oficial da Aster Companion.
echo   Ele apenas configura atalhos e recursos opcionais sem
echo   precisar digitar comandos no terminal.
echo.
echo   Suas configuracoes (.env), memorias e skins proprias
echo   NUNCA sao apagadas por este assistente.
echo.
echo  ------------------------------------------------------------
echo   [1] Criar atalho da Aster na Area de Trabalho
echo   [2] Instalar Microfone Offline (Python 3.12 + Whisper)
echo   [3] Instalar Importador de Skins Beta (Python 3.12 + rembg)
echo   [4] Instalar TUDO (Atalho + Microfone + Importador de Skins)
echo   [5] Abrir a Aster agora
echo   [0] Sair
echo  ------------------------------------------------------------
echo.
set /p "OPT=  Escolha uma opcao e aperte Enter: "

if "%OPT%"=="1" goto ATALHO
if "%OPT%"=="2" goto MIC
if "%OPT%"=="3" goto SKINS
if "%OPT%"=="4" goto TUDO
if "%OPT%"=="5" goto ABRIR
if "%OPT%"=="0" exit /b 0
goto MENU

:ENSURE_PY312
echo.
echo  [Aster] Verificando Python 3.12...
py -3.12 --version >nul 2>&1
if %errorlevel% neq 0 (
    echo  [Aster] Python 3.12 nao encontrado. Instalando via winget...
    echo  [Aster] Se o Windows pedir permissao, clique em Sim.
    winget install --id Python.Python.3.12 -e --source winget --accept-package-agreements --accept-source-agreements
    if %errorlevel% neq 0 (
        echo.
        echo  [ERRO] Nao foi possivel instalar o Python 3.12 automaticamente.
        echo  Baixe manualmente em: https://www.python.org/downloads/release/python-31210/
        exit /b 1
    )
    echo  [Aster] Python 3.12 instalado!
)
if not exist ".venv\Scripts\python.exe" (
    echo  [Aster] Criando ambiente isolado (.venv)...
    py -3.12 -m venv .venv
    if %errorlevel% neq 0 (
        echo  [ERRO] Falha ao criar .venv. Feche esta janela, abra de novo e tente novamente.
        exit /b 1
    )
)
exit /b 0

:DO_ATALHO
echo.
echo  [Aster] Criando atalho na Area de Trabalho...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws = New-Object -ComObject WScript.Shell; $desk = [Environment]::GetFolderPath('Desktop'); $sc = $ws.CreateShortcut((Join-Path $desk 'Aster Companion.lnk')); $sc.TargetPath = (Join-Path '%~dp0' 'Aster.exe'); $sc.WorkingDirectory = '%~dp0'; $sc.IconLocation = (Join-Path '%~dp0' 'Aster.exe') + ',0'; $sc.Save()"
echo  [OK] Atalho "Aster Companion" criado na Area de Trabalho!
exit /b 0

:ATALHO
call :DO_ATALHO
echo.
pause
goto MENU

:MIC
call :ENSURE_PY312
if %errorlevel% neq 0 (
    echo.
    pause
    goto MENU
)
echo.
echo  [Aster] Instalando dependencias do Microfone Offline (faster-whisper)...
.\.venv\Scripts\python.exe -m pip install --upgrade pip
if exist "requirements.txt" (
    .\.venv\Scripts\pip.exe install -r requirements.txt
) else (
    .\.venv\Scripts\pip.exe install faster-whisper
)
echo.
echo  [OK] Microfone offline pronto! (Dica: se voce usa Groq em Cerebro IA, o microfone ja funciona direto na nuvem sem precisar disso.)
echo.
pause
goto MENU

:SKINS
call :ENSURE_PY312
if %errorlevel% neq 0 (
    echo.
    pause
    goto MENU
)
echo.
echo  [Aster] Instalando dependencias do Importador de Skins (rembg + onnxruntime)...
.\.venv\Scripts\python.exe -m pip install --upgrade pip
.\.venv\Scripts\pip.exe install rembg onnxruntime pillow numpy av
echo.
echo  [OK] Importador de Skins (Beta) pronto! Agora voce pode usar o botao "Importar (Beta)" em Visual -> Skins.
echo.
pause
goto MENU

:TUDO
call :DO_ATALHO
call :ENSURE_PY312
if %errorlevel% neq 0 (
    echo.
    pause
    goto MENU
)
echo.
echo  [Aster] Instalando Microfone Offline + Importador de Skins...
.\.venv\Scripts\python.exe -m pip install --upgrade pip
if exist "requirements.txt" (
    .\.venv\Scripts\pip.exe install -r requirements.txt
) else (
    .\.venv\Scripts\pip.exe install faster-whisper
)
.\.venv\Scripts\pip.exe install rembg onnxruntime pillow numpy av
echo.
echo  ============================================================
echo   [OK] Tudo pronto! Voce ja pode abrir a Aster Companion.
echo  ============================================================
echo.
pause
goto MENU

:ABRIR
if exist "Aster.exe" (
    start "" "%~dp0Aster.exe"
    exit /b 0
) else (
    echo.
    echo  [ERRO] Aster.exe nao encontrado nesta pasta.
    echo.
    pause
    goto MENU
)
