use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,
    pub ollama_model: String,
    pub hermes_url: String,
    pub hermes_model: String,
    pub hermes_key: String,
    pub discord_webhook_url: String,
    pub whisper_model: String,
    pub tts_voice: String,
    pub tts_rate: String,
    pub use_chatterbox: String,
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
    pub fn load() -> Self {
        let vars = parse_env_file();
        Self {
            ollama_url: get_var(&vars, "OLLAMA_URL", "http://127.0.0.1:11434/api/chat"),
            ollama_model: get_var(&vars, "OLLAMA_MODEL", "qwen3:4b-instruct"),
            hermes_url: get_var(&vars, "HERMES_URL", "http://127.0.0.1:8642/v1/chat/completions"),
            hermes_model: get_var(&vars, "HERMES_MODEL", "hermes-agent"),
            hermes_key: get_var(&vars, "HERMES_KEY", ""),
            discord_webhook_url: get_var(&vars, "DISCORD_WEBHOOK_URL", ""),
            whisper_model: get_var(&vars, "WHISPER_MODEL", "base"),
            tts_voice: {
                let v = get_var(&vars, "TTS_VOICE", "pt-BR-ThalitaMultilingualNeural");
                if matches!(
                    v.as_str(),
                    "pt-BR-GiovannaNeural" | "pt-BR-BrendaNeural" | "pt-BR-ThalitaNeural"
                ) {
                    "pt-BR-ThalitaMultilingualNeural".to_string()
                } else {
                    v
                }
            },
            tts_rate: get_var(&vars, "TTS_RATE", "+8%"),
            use_chatterbox: get_var(&vars, "USE_CHATTERBOX", "0"),
        }
    }
}

#[tauri::command]
pub fn get_settings() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
pub fn save_settings(cfg: AppConfig) -> Result<(), String> {
    let path = target_env_path()?;
    let content = format!(
        "# Configuração local da Aster Companion (NÃO subir para o Git!)\n\
         OLLAMA_URL={}\n\
         OLLAMA_MODEL={}\n\
         \n\
         HERMES_URL={}\n\
         HERMES_MODEL={}\n\
         HERMES_KEY={}\n\
         \n\
         DISCORD_WEBHOOK_URL={}\n\
         \n\
         # Voz\n\
         WHISPER_MODEL={}\n\
         TTS_VOICE={}\n\
         TTS_RATE={}\n\
         USE_CHATTERBOX={}\n",
        clean_env_val(&cfg.ollama_url),
        clean_env_val(&cfg.ollama_model),
        clean_env_val(&cfg.hermes_url),
        clean_env_val(&cfg.hermes_model),
        clean_env_val(&cfg.hermes_key),
        clean_env_val(&cfg.discord_webhook_url),
        clean_env_val(&cfg.whisper_model),
        clean_env_val(&cfg.tts_voice),
        clean_env_val(&cfg.tts_rate),
        clean_env_val(&cfg.use_chatterbox),
    );
    fs::write(&path, content).map_err(|e| format!("Erro salvando .env: {e}"))
}
