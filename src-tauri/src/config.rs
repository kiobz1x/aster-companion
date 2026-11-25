use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use tauri::{Emitter, Manager};

static CONFIG_CACHE: OnceLock<RwLock<AppConfig>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_ai_provider")]
    pub ai_provider: String,
    pub ollama_url: String,
    pub ollama_model: String,
    #[serde(default = "default_vision_model")]
    pub ollama_vision_model: String,
    #[serde(default = "default_cloud_url")]
    pub cloud_url: String,
    #[serde(default = "default_cloud_model")]
    pub cloud_model: String,
    #[serde(default)]
    pub cloud_api_key: String,
    pub hermes_url: String,
    pub hermes_model: String,
    pub hermes_key: String,
    pub discord_webhook_url: String,
    pub whisper_model: String,
    pub fish_api_key: String,
    pub fish_voice_id: String,
    pub fish_model: String,
    pub tts_enabled: String,
    #[serde(default = "default_tts_volume")]
    pub tts_volume: String,
    #[serde(default = "default_skin")]
    pub active_skin: String,
    #[serde(default = "default_ui_theme")]
    pub ui_theme: String,
    #[serde(default = "default_particles_enabled")]
    pub particles_enabled: String,
    #[serde(default = "default_sprite_scale")]
    pub sprite_scale: String,
    #[serde(default = "default_lol_enabled")]
    pub lol_enabled: String,
    #[serde(default)]
    pub user_name: String,
}

fn default_ai_provider() -> String {
    "ollama".to_string()
}

fn default_cloud_url() -> String {
    "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".to_string()
}

fn default_cloud_model() -> String {
    "gemini-2.5-flash".to_string()
}

fn default_sprite_scale() -> String {
    "100".to_string()
}

fn default_lol_enabled() -> String {
    "1".to_string()
}

fn default_particles_enabled() -> String {
    "1".to_string()
}

fn default_vision_model() -> String {
    "qwen2.5vl:3b".to_string()
}

fn default_tts_volume() -> String {
    "70".to_string()
}

fn default_skin() -> String {
    "padrao".to_string()
}

fn default_ui_theme() -> String {
    "gothic".to_string()
}

pub fn find_project_root() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.to_path_buf());
            if let Some(p1) = dir.parent() {
                if let Some(p2) = p1.parent() {
                    if let Some(p3) = p2.parent() {
                        candidates.push(p3.to_path_buf());
                    }
                }
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.clone());
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    candidates
        .into_iter()
        .find(|dir| dir.join("transcrever.py").is_file() || dir.join(".env").is_file())
}

fn find_env_file() -> Option<PathBuf> {
    if let Some(root) = find_project_root() {
        let p = root.join(".env");
        if p.is_file() {
            return Some(p);
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(".env"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join(".env"));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(".env"));
        }
    }

    candidates.into_iter().find(|p| p.is_file())
}

fn target_env_path() -> Result<PathBuf, String> {
    if let Some(p) = find_env_file() {
        return Ok(p);
    }
    if let Some(root) = find_project_root() {
        return Ok(root.join(".env"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return Ok(dir.join(".env"));
        }
    }
    Err("Não foi possível localizar o caminho do .env.".into())
}

fn parse_env_file() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(path) = find_env_file() else {
        return map;
    };

    let Ok(content) = fs::read_to_string(path) else {
        return map;
    };

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            map.insert(key, val);
        }
    }

    map
}

fn get_var(file_vars: &HashMap<String, String>, key: &str, default: &str) -> String {
    if let Some(v) = file_vars.get(key) {
        if !v.is_empty() {
            return v.clone();
        }
    }
    if let Ok(v) = std::env::var(key) {
        if !v.is_empty() {
            return v;
        }
    }
    default.to_string()
}

fn clean_env_val(s: &str) -> String {
    s.replace(['\r', '\n'], "").trim().to_string()
}

impl AppConfig {
    fn load_from_disk() -> Self {
        let vars = parse_env_file();
        Self {
            ai_provider: get_var(&vars, "AI_PROVIDER", "ollama"),
            ollama_url: get_var(&vars, "OLLAMA_URL", "http://127.0.0.1:11434/api/chat"),
            ollama_model: get_var(&vars, "OLLAMA_MODEL", "qwen3:4b-instruct"),
            ollama_vision_model: get_var(&vars, "OLLAMA_VISION_MODEL", "qwen2.5vl:3b"),
            cloud_url: get_var(
                &vars,
                "CLOUD_URL",
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            ),
            cloud_model: get_var(&vars, "CLOUD_MODEL", "gemini-2.5-flash"),
            cloud_api_key: get_var(&vars, "CLOUD_API_KEY", ""),
            hermes_url: get_var(&vars, "HERMES_URL", "http://127.0.0.1:8642/v1/chat/completions"),
            hermes_model: get_var(&vars, "HERMES_MODEL", "hermes-agent"),
            hermes_key: get_var(&vars, "HERMES_KEY", ""),
            discord_webhook_url: get_var(&vars, "DISCORD_WEBHOOK_URL", ""),
            whisper_model: get_var(&vars, "WHISPER_MODEL", "base"),
            fish_api_key: get_var(&vars, "FISH_API_KEY", ""),
            fish_voice_id: get_var(&vars, "FISH_VOICE_ID", ""),
            fish_model: get_var(&vars, "FISH_MODEL", "s1"),
            tts_enabled: get_var(&vars, "TTS_ENABLED", "1"),
            tts_volume: get_var(&vars, "TTS_VOLUME", "70"),
            active_skin: get_var(&vars, "ACTIVE_SKIN", "padrao"),
            ui_theme: get_var(&vars, "UI_THEME", "gothic"),
            particles_enabled: get_var(&vars, "PARTICLES_ENABLED", "1"),
            sprite_scale: get_var(&vars, "SPRITE_SCALE", "100"),
            lol_enabled: get_var(&vars, "LOL_ENABLED", "1"),
            user_name: get_var(&vars, "USER_NAME", ""),
        }
    }

    pub fn load() -> Self {
        let lock = CONFIG_CACHE.get_or_init(|| RwLock::new(Self::load_from_disk()));
        lock.read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| Self::load_from_disk())
    }
}

#[tauri::command]
pub fn get_settings() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, cfg: AppConfig) -> Result<(), String> {
    let path = target_env_path()?;
    let vol_clean = clean_env_val(&cfg.tts_volume)
        .parse::<u32>()
        .unwrap_or(70)
        .min(100);
    let theme_clean = {
        let t = clean_env_val(&cfg.ui_theme);
        if t.is_empty() { "gothic".to_string() } else { t }
    };
    let particles_clean = if clean_env_val(&cfg.particles_enabled) == "0" { "0" } else { "1" };
    let lol_clean = if clean_env_val(&cfg.lol_enabled) == "0" { "0" } else { "1" };
    let scale_clean = clean_env_val(&cfg.sprite_scale)
        .parse::<u32>()
        .unwrap_or(100)
        .clamp(75, 130);
    let provider_clean = if clean_env_val(&cfg.ai_provider).eq_ignore_ascii_case("cloud") {
        "cloud"
    } else {
        "ollama"
    };
    let content = format!(
        "# Configuração local da Aster Companion (NÃO subir para o Git!)\n\
         AI_PROVIDER={}\n\
         OLLAMA_URL={}\n\
         OLLAMA_MODEL={}\n\
         OLLAMA_VISION_MODEL={}\n\
         \n\
         # API na Nuvem (OpenAI-compatible: Gemini, Groq, OpenRouter, OpenAI)\n\
         CLOUD_URL={}\n\
         CLOUD_MODEL={}\n\
         CLOUD_API_KEY={}\n\
         \n\
         HERMES_URL={}\n\
         HERMES_MODEL={}\n\
         HERMES_KEY={}\n\
         \n\
         DISCORD_WEBHOOK_URL={}\n\
         \n\
         # Voz (STT Whisper + TTS Fish Audio)\n\
         WHISPER_MODEL={}\n\
         TTS_ENABLED={}\n\
         TTS_VOLUME={}\n\
         FISH_API_KEY={}\n\
         FISH_VOICE_ID={}\n\
         FISH_MODEL={}\n\
         \n\
         # Aparência, perfil, partículas e jogos\n\
         USER_NAME={}\n\
         ACTIVE_SKIN={}\n\
         UI_THEME={}\n\
         PARTICLES_ENABLED={}\n\
         SPRITE_SCALE={}\n\
         LOL_ENABLED={}\n",
        provider_clean,
        clean_env_val(&cfg.ollama_url),
        clean_env_val(&cfg.ollama_model),
        clean_env_val(&cfg.ollama_vision_model),
        clean_env_val(&cfg.cloud_url),
        clean_env_val(&cfg.cloud_model),
        clean_env_val(&cfg.cloud_api_key),
        clean_env_val(&cfg.hermes_url),
        clean_env_val(&cfg.hermes_model),
        clean_env_val(&cfg.hermes_key),
        clean_env_val(&cfg.discord_webhook_url),
        clean_env_val(&cfg.whisper_model),
        clean_env_val(&cfg.tts_enabled),
        vol_clean,
        clean_env_val(&cfg.fish_api_key),
        clean_env_val(&cfg.fish_voice_id),
        clean_env_val(&cfg.fish_model),
        clean_env_val(&cfg.user_name),
        clean_env_val(&cfg.active_skin),
        theme_clean,
        particles_clean,
        scale_clean,
        lol_clean,
    );
    fs::write(&path, content).map_err(|e| format!("Erro salvando .env: {e}"))?;
    if let Some(lock) = CONFIG_CACHE.get() {
        if let Ok(mut guard) = lock.write() {
            *guard = AppConfig::load_from_disk();
        }
    }
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.emit(
            "aster-settings-updated",
            serde_json::json!({
                "tts_volume": vol_clean,
                "ui_theme": theme_clean,
                "particles_enabled": particles_clean == "1",
                "sprite_scale": scale_clean,
                "lol_enabled": lol_clean == "1",
                "user_name": clean_env_val(&cfg.user_name)
            }),
        );
    }
    Ok(())
}
