use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tauri::AppHandle;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const GITHUB_LATEST_URL: &str =
    "https://api.github.com/repos/kiobz1x/aster-companion/releases/latest";
const ALLOWED_DOWNLOAD_PREFIX: &str =
    "https://github.com/kiobz1x/aster-companion/releases/download/";

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub download_url: String,
    pub release_url: String,
}

fn parse_semver(raw: &str) -> (u32, u32, u32) {
    let clean = raw.trim().trim_start_matches(['v', 'V']);
    let mut parts = clean.split('.').map(|p| {
        p.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0)
    });
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    (major, minor, patch)
}

#[tauri::command]
pub async fn check_for_update() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(6))
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| format!("Erro HTTP: {e}"))?;

    let res = client
        .get(GITHUB_LATEST_URL)
        .header("User-Agent", format!("Aster-Companion/{current}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Sem conexão com o GitHub: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("GitHub HTTP {}", res.status()));
    }

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Resposta inválida do GitHub: {e}"))?;

    let tag = body["tag_name"].as_str().unwrap_or("").trim().to_string();
    let latest = tag.trim_start_matches(['v', 'V']).to_string();
    let release_url = body["html_url"]
        .as_str()
        .unwrap_or("https://github.com/kiobz1x/aster-companion/releases")
        .to_string();

    let download_url = body["assets"]
        .as_array()
        .and_then(|arr| {
            arr.iter().find_map(|a| {
                let name = a["name"].as_str().unwrap_or("").to_lowercase();
                if name.ends_with(".zip") {
                    a["browser_download_url"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();

    let has_update = !latest.is_empty() && parse_semver(&latest) > parse_semver(&current);

    Ok(UpdateInfo {
        current_version: current,
        latest_version: if latest.is_empty() {
            env!("CARGO_PKG_VERSION").to_string()
        } else {
            latest
        },
        has_update,
        download_url,
        release_url,
    })
}

#[tauri::command]
pub async fn install_update(app: AppHandle, download_url: String) -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let _ = (app, download_url);
        return Err("Auto-atualização disponível apenas no Windows.".into());
    }

    #[cfg(windows)]
    {
        if !download_url.starts_with(ALLOWED_DOWNLOAD_PREFIX) || !download_url.ends_with(".zip") {
            return Err("Link de atualização inválido ou fora do repositório oficial.".into());
        }

        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Não foi possível localizar o Aster.exe: {e}"))?;
        let app_dir: PathBuf = current_exe
            .parent()
            .ok_or_else(|| "Pasta do executável não encontrada.".to_string())?
            .to_path_buf();

        let tmp = std::env::temp_dir();
        let zip_path = tmp.join("aster_update_pkg.zip");
        let stage_dir = tmp.join("aster_update_stage");
        let script_path = tmp.join("aster_apply_update.cmd");

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| format!("Erro HTTP: {e}"))?;

        let res = client
            .get(&download_url)
            .header("User-Agent", "Aster-Companion-Updater")
            .send()
            .await
            .map_err(|e| format!("Erro baixando atualização: {e}"))?;

        if !res.status().is_success() {
            return Err(format!("Falha no download (HTTP {}).", res.status()));
        }

        let bytes = res
            .bytes()
            .await
            .map_err(|e| format!("Erro lendo pacote da atualização: {e}"))?;
        if bytes.len() < 100_000 {
            return Err("Arquivo baixado parece incompleto.".into());
        }
        fs::write(&zip_path, &bytes).map_err(|e| format!("Erro salvando zip temporário: {e}"))?;

        if stage_dir.exists() {
            let _ = fs::remove_dir_all(&stage_dir);
        }

        let ps_extract = format!(
            "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            zip_path.display().to_string().replace('\'', "''"),
            stage_dir.display().to_string().replace('\'', "''"),
        );
        let mut ex_cmd = Command::new("powershell");
        ex_cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_extract]);
        ex_cmd.creation_flags(CREATE_NO_WINDOW);
        let ex_status = ex_cmd
            .status()
            .map_err(|e| format!("Erro ao extrair atualização: {e}"))?;
        if !ex_status.success() {
            return Err("Falha ao extrair o pacote .zip da atualização.".into());
        }

        // Suporta tanto zip com pasta raiz (Aster-Companion\Aster.exe) quanto direto na raiz (Aster.exe)
        let src_dir = if stage_dir.join("Aster.exe").is_file() {
            stage_dir.clone()
        } else {
            let nested = stage_dir.join("Aster-Companion");
            if nested.join("Aster.exe").is_file() {
                nested
            } else {
                return Err("Pacote de atualização não contém Aster.exe.".into());
            }
        };

        let pid = std::process::id();
        let exe_name = current_exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Aster.exe");

        // Robocopy sem /MIR nem /PURGE: copia por cima os arquivos novos e NUNCA apaga .env, memory.json, docs ou skins próprias!
        let cmd_content = format!(
            "@echo off\r\n\
            timeout /t 2 /nobreak >nul\r\n\
            taskkill /PID {pid} /F >nul 2>&1\r\n\
            timeout /t 1 /nobreak >nul\r\n\
            robocopy \"{src}\" \"{dst}\" /E /R:3 /W:1 /NFL /NDL /NJH /NJS /NP >nul\r\n\
            del /f /q \"{zip}\" >nul 2>&1\r\n\
            rmdir /s /q \"{stage}\" >nul 2>&1\r\n\
            start \"\" \"{dst}\\{exe}\"\r\n\
            del /f /q \"%~f0\" >nul 2>&1\r\n",
            pid = pid,
            src = src_dir.display(),
            dst = app_dir.display(),
            zip = zip_path.display(),
            stage = stage_dir.display(),
            exe = exe_name,
        );

        fs::write(&script_path, cmd_content)
            .map_err(|e| format!("Erro criando script de atualização: {e}"))?;

        let mut launch = Command::new("cmd");
        launch.args(["/C", &script_path.to_string_lossy()]);
        launch.creation_flags(CREATE_NO_WINDOW);
        launch
            .spawn()
            .map_err(|e| format!("Erro iniciando atualizador: {e}"))?;

        app.exit(0);
        Ok("Atualizando e reiniciando a Aster...".into())
    }
}
