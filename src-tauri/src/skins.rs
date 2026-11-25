use crate::config::{find_project_root, AppConfig};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{Emitter, Manager};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize)]
pub struct SkinInfo {
    pub id: String,
    pub name: String,
    pub sprite_count: usize,
    pub active: bool,
    pub present_keys: Vec<String>,
    pub preview: Option<String>,
}

pub fn skins_dir_path() -> Result<PathBuf, String> {
    let base = if let Some(root) = find_project_root() {
        root
    } else if let Ok(exe) = std::env::current_exe() {
        exe.parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Pasta do executável não encontrada.".to_string())?
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?
    };

    let dir = base.join("skins");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Erro criando pasta skins/: {e}"))?;
    }
    Ok(dir)
}

fn sprite_mime_and_priority(filename: &str) -> Option<(&'static str, u8)> {
    let low = filename.to_lowercase();
    if low.ends_with(".gif") {
        Some(("image/gif", 3))
    } else if low.ends_with(".webp") {
        Some(("image/webp", 2))
    } else if low.ends_with(".png") {
        Some(("image/png", 1))
    } else {
        None
    }
}

fn map_filename_to_key(filename: &str) -> Option<&'static str> {
    let low = filename.to_lowercase();
    let stem = low
        .strip_suffix(".png")
        .or_else(|| low.strip_suffix(".webp"))
        .or_else(|| low.strip_suffix(".gif"))?;
    match stem {
        "aster_idle" | "idle" => Some("idle"),
        "aster_blink" | "blink" => Some("blink"),
        "aster_talk" | "talk" => Some("talk"),
        "aster_talk_calm" | "talk_calm" => Some("talk_calm"),
        "aster_smug" | "smug" => Some("smug"),
        "aster_angry" | "angry" => Some("angry"),
        "aster_sleepy" | "sleepy" => Some("sleepy"),
        "aster_think" | "think" => Some("think"),
        "aster_drag" | "drag" => Some("drag"),
        "aster_sit" | "sit" => Some("sit"),
        "aster_listen" | "listen" => Some("listen"),
        "aster_music" | "music" => Some("music"),
        "aster_walk_1" | "walk_1" => Some("walk1"),
        "aster_walk_2" | "walk_2" => Some("walk2"),
        "aster_walk_3" | "walk_3" => Some("walk3"),
        "aster_walk_4" | "walk_4" => Some("walk4"),
        "aster_walk_5" | "walk_5" => Some("walk5"),
        "aster_walk_6" | "walk_6" => Some("walk6"),
        "aster_walk_7" | "walk_7" => Some("walk7"),
        "aster_walk_8" | "walk_8" => Some("walk8"),
        _ => None,
    }
}

fn prettify_name(id: &str) -> String {
    let mut chars = id.replace(['_', '-'], " ");
    if let Some(first) = chars.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    chars
}

#[tauri::command]
pub fn skins_list() -> Result<Vec<SkinInfo>, String> {
    let cfg = AppConfig::load();
    let active_id = if cfg.active_skin.is_empty() {
        "padrao".to_string()
    } else {
        cfg.active_skin
    };

    let all_default_keys: Vec<String> = [
        "idle", "blink", "talk", "talk_calm", "smug", "angry", "sleepy", "think",
        "drag", "sit", "listen", "music",
        "walk1", "walk2", "walk3", "walk4", "walk5", "walk6", "walk7", "walk8",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut list = vec![SkinInfo {
        id: "padrao".to_string(),
        name: "Aster Padrão".to_string(),
        sprite_count: 20,
        active: active_id == "padrao",
        present_keys: all_default_keys,
        preview: None,
    }];

    let dir = skins_dir_path()?;
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut custom = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if id.starts_with('.') || id.starts_with('_') || id.eq_ignore_ascii_case("padrao") {
                continue;
            }
            let mut keys: Vec<String> = Vec::new();
            let mut preview_path: Option<(PathBuf, &'static str, u8)> = None;
            if let Ok(rd) = fs::read_dir(&p) {
                for e in rd.flatten() {
                    let ep = e.path();
                    if !ep.is_file() {
                        continue;
                    }
                    let Some(fname) = e.file_name().to_str().map(|s| s.to_string()) else {
                        continue;
                    };
                    if let (Some(key), Some((mime, prio))) =
                        (map_filename_to_key(&fname), sprite_mime_and_priority(&fname))
                    {
                        keys.push(key.to_string());
                        if key == "idle" {
                            let replace = preview_path.as_ref().map(|(_, _, p0)| prio > *p0).unwrap_or(true);
                            if replace {
                                preview_path = Some((ep, mime, prio));
                            }
                        }
                    }
                }
            }
            keys.sort();
            keys.dedup();
            let count = keys.len();
            let preview = preview_path.and_then(|(pp, mime, _)| {
                fs::read(&pp)
                    .ok()
                    .map(|bytes| format!("data:{mime};base64,{}", BASE64.encode(&bytes)))
            });

            custom.push(SkinInfo {
                name: prettify_name(&id),
                active: active_id.eq_ignore_ascii_case(&id),
                id,
                sprite_count: count,
                present_keys: keys,
                preview,
            });
        }
        custom.sort_by(|a, b| a.id.to_lowercase().cmp(&b.id.to_lowercase()));
        list.extend(custom);
    }

    Ok(list)
}

#[tauri::command]
pub fn skins_get_overrides(skin_id: Option<String>) -> Result<HashMap<String, String>, String> {
    let id = match skin_id {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => AppConfig::load().active_skin,
    };

    let mut map = HashMap::new();
    if id.is_empty() || id.eq_ignore_ascii_case("padrao") {
        return Ok(map);
    }

    // Bloqueia path traversal
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err("Nome de skin inválido.".into());
    }

    let skin_dir = skins_dir_path()?.join(&id);
    if !skin_dir.is_dir() {
        return Ok(map);
    }

    let mut priorities: HashMap<&'static str, u8> = HashMap::new();
    if let Ok(entries) = fs::read_dir(&skin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(fname) = entry.file_name().to_str().map(|s| s.to_string()) else {
                continue;
            };
            let (Some(key), Some((mime, prio))) =
                (map_filename_to_key(&fname), sprite_mime_and_priority(&fname))
            else {
                continue;
            };
            if priorities.get(key).copied().unwrap_or(0) > prio {
                continue;
            }
            if let Ok(bytes) = fs::read(&path) {
                let data_uri = format!("data:{mime};base64,{}", BASE64.encode(&bytes));
                map.insert(key.to_string(), data_uri);
                priorities.insert(key, prio);
            }
        }
    }

    Ok(map)
}

#[tauri::command]
pub fn skins_set_active(app: tauri::AppHandle, skin_id: String) -> Result<(), String> {
    let clean = skin_id.trim().to_string();
    if clean.contains("..") || clean.contains('/') || clean.contains('\\') {
        return Err("Nome de skin inválido.".into());
    }
    let mut cfg = AppConfig::load();
    cfg.active_skin = if clean.is_empty() {
        "padrao".to_string()
    } else {
        clean.clone()
    };
    crate::config::save_settings(app.clone(), cfg)?;

    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("aster-skin-changed", clean);
    }
    Ok(())
}

#[tauri::command]
pub fn skins_create(name: String) -> Result<String, String> {
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect();
    let slug = slug.trim_matches('_').to_string();
    if slug.is_empty() || slug == "padrao" {
        return Err("Informe um nome válido para a nova skin.".into());
    }
    let dir = skins_dir_path()?.join(&slug);
    fs::create_dir_all(&dir).map_err(|e| format!("Erro criando pasta da skin: {e}"))?;

    let readme = dir.join("README.txt");
    if !readme.exists() {
        let _ = fs::write(
            &readme,
            "=== GUIA DE SPRITES DA SKIN ===\n\
             Coloque aqui os PNGs com fundo transparente.\n\
             Com apenas os 6 ESSENCIAIS a skin já funciona 100% sem voltar pro padrão!\n\n\
             [ESSENCIAIS - 6]\n\
             - aster_idle.png    (parada)\n\
             - aster_talk.png    (falando)\n\
             - aster_think.png   (pensando)\n\
             - aster_sleepy.png  (com sono)\n\
             - aster_drag.png    (sendo segurada)\n\
             - aster_sit.png     (sentada na janela)\n\n\
             [EXPRESSOES RECOMENDADAS - 3]\n\
             - aster_smug.png    (convencida/zueira)\n\
             - aster_angry.png   (irritada)\n\
             - aster_blink.png   (piscando)\n\n\
             [EXTRAS OPCIONAIS]\n\
             - aster_talk_calm.png (fala calma)\n\
             - aster_listen.png    (ouvindo microfone)\n\
             - aster_music.png     (curtindo musica)\n\
             - aster_walk_1.png .. aster_walk_8.png (caminhada)\n",
        );
    }

    #[cfg(windows)]
    {
        let mut cmd = Command::new("explorer");
        cmd.arg(&dir);
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.spawn();
    }

    Ok(slug)
}

#[tauri::command]
pub async fn skins_import_folder(skin_id: String, add_outline: bool) -> Result<String, String> {
    let clean = skin_id.trim().to_string();
    if clean.is_empty()
        || clean.eq_ignore_ascii_case("padrao")
        || clean.contains("..")
        || clean.contains('/')
        || clean.contains('\\')
    {
        return Err("Skin inválida para importação.".into());
    }

    let root = find_project_root().ok_or_else(|| "Pasta raiz do projeto não encontrada.".to_string())?;
    let py_exe = root.join(".venv").join("Scripts").join("python.exe");
    let script = root.join("processar_skin.py");
    if !py_exe.is_file() {
        return Err("Ambiente .venv não encontrado. Crie o .venv e instale rembg + onnxruntime.".into());
    }
    if !script.is_file() {
        return Err("Script processar_skin.py não encontrado na raiz do projeto.".into());
    }

    let skin_dir = skins_dir_path()?.join(&clean);
    fs::create_dir_all(&skin_dir).map_err(|e| format!("Erro criando pasta da skin: {e}"))?;

    // Abre seletor nativo de pasta do Windows
    let selected_dir = tauri::async_runtime::spawn_blocking(move || -> Result<Option<String>, String> {
        #[cfg(windows)]
        {
            let ps = r#"
Add-Type -AssemblyName System.Windows.Forms
$dlg = New-Object System.Windows.Forms.FolderBrowserDialog
$dlg.Description = 'Selecione a pasta com os sprites crus (.webp, .png, .jpg) e/ou .mp4 do walk'
$dlg.ShowNewFolderButton = $false
$res = $dlg.ShowDialog()
if ($res -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dlg.SelectedPath
}
"#;
            let mut cmd = Command::new("powershell");
            cmd.args(["-NoProfile", "-STA", "-Command", ps]);
            cmd.creation_flags(CREATE_NO_WINDOW);
            let out = cmd.output().map_err(|e| format!("Erro abrindo seletor de pasta: {e}"))?;
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if path.is_empty() {
                Ok(None)
            } else {
                Ok(Some(path))
            }
        }
        #[cfg(not(windows))]
        {
            Err("Importador disponível apenas no Windows.".into())
        }
    })
    .await
    .map_err(|e| format!("Erro no seletor: {e}"))??;

    let Some(input_dir) = selected_dir else {
        return Ok("Importação cancelada.".into());
    };

    let outline_arg = if add_outline { "2" } else { "0" };
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let mut cmd = Command::new(&py_exe);
        cmd.arg(&script)
            .arg("--input-dir")
            .arg(&input_dir)
            .arg("--skin-dir")
            .arg(&skin_dir)
            .arg("--outline")
            .arg(outline_arg);
        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let out = cmd
            .output()
            .map_err(|e| format!("Erro executando processar_skin.py: {e}"))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Falha no processamento: {}", err.chars().take(240).collect::<String>()));
        }
        let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(if msg.is_empty() {
            "Importação concluída.".into()
        } else {
            msg
        })
    })
    .await
    .map_err(|e| format!("Erro na thread de importação: {e}"))?
}

#[tauri::command]
pub fn skins_open_folder() -> Result<(), String> {
    let dir = skins_dir_path()?;
    #[cfg(windows)]
    {
        let mut cmd = Command::new("explorer");
        cmd.arg(&dir);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn()
            .map_err(|e| format!("Erro abrindo pasta skins/: {e}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = dir;
        Err("Disponível apenas no Windows.".into())
    }
}
