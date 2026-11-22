use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

fn strip_think_tags(input: &str) -> String {
    let mut out = input.to_string();
    while let Some(start) = out.find("<think>") {
        if let Some(end_rel) = out[start..].find("</think>") {
            let end = start + end_rel + "</think>".len();
            out.replace_range(start..end, "");
        } else {
            out.truncate(start);
            break;
        }
    }
    out.trim().to_string()
}

#[tauri::command]
pub async fn chat_ollama(
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    num_ctx: Option<u32>,
) -> Result<String, String> {
    let cfg = AppConfig::load();
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Falha ao iniciar cliente HTTP: {e}"))?;

    let payload = json!({
        "model": cfg.ollama_model,
        "messages": messages,
        "stream": false,
        "think": false,
        "options": {
            "temperature": temperature.unwrap_or(0.7),
            "num_ctx": num_ctx.unwrap_or(2048)
        }
    });

    let res = client
        .post(&cfg.ollama_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                format!("Ollama desligado ou inacessível em {}.", cfg.ollama_url)
            } else if e.is_timeout() {
                "Ollama demorou demais para responder (timeout).".to_string()
            } else {
                format!("Erro ao falar com Ollama: {e}")
            }
        })?;

    if !res.status().is_success() {
        return Err(format!("Ollama HTTP {}", res.status()));
    }

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Resposta inválida do Ollama: {e}"))?;

    let content = body["message"]["content"].as_str().unwrap_or("");
    Ok(strip_think_tags(content))
}

#[tauri::command]
pub async fn ask_hermes(task: String) -> Result<String, String> {
    let cfg = AppConfig::load();
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Falha ao iniciar cliente HTTP: {e}"))?;

    let payload = json!({
        "model": cfg.hermes_model,
        "stream": false,
        "messages": [
            {
                "role": "system",
                "content": "Você é Kaida/Hermes executando uma tarefa delegada pela Aster. Responda de forma objetiva e concisa em PT-BR."
            },
            {
                "role": "user",
                "content": task
            }
        ]
    });

    let mut req = client.post(&cfg.hermes_url).json(&payload);
    if !cfg.hermes_key.is_empty() && cfg.hermes_key != "cole_sua_hermes_key_aqui" {
        req = req.header("Authorization", format!("Bearer {}", cfg.hermes_key));
    }

    let res = req
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                format!("Kaida/Hermes desligada ou porta fechada ({}). Verifique se o api_server do Hermes está rodando.", cfg.hermes_url)
            } else if e.is_timeout() {
                "Kaida/Hermes conectou, mas demorou mais de 3 min sem responder (timeout).".to_string()
            } else {
                format!("Erro de conexão com Kaida/Hermes ({}): {e}", cfg.hermes_url)
            }
        })?;

    let status = res.status();
    if !status.is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("Hermes HTTP {status}: {}", err_body.chars().take(120).collect::<String>()));
    }

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Resposta inválida do Hermes: {e}"))?;

    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Concluído.");

    Ok(strip_think_tags(content))
}

#[tauri::command]
pub async fn post_discord_webhook(content: String) -> Result<(), String> {
    let cfg = AppConfig::load();
    if cfg.discord_webhook_url.is_empty() {
        return Err("Configure DISCORD_WEBHOOK_URL no arquivo .env primeiro.".into());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Falha ao iniciar cliente HTTP: {e}"))?;

    let res = client
        .post(&cfg.discord_webhook_url)
        .json(&json!({
            "username": "Aster",
            "content": content
        }))
        .send()
        .await
        .map_err(|e| format!("Erro ao enviar pro Discord: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("Discord Webhook HTTP {}", res.status()));
    }

    Ok(())
}

#[tauri::command]
pub fn read_clipboard() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    match clipboard.get_text() {
        Ok(text) => Ok(text),
        Err(arboard::Error::ContentNotAvailable) => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceHealth {
    pub ollama_ok: bool,
    pub hermes_ok: bool,
    pub fish_configured: bool,
}

#[tauri::command]
pub async fn check_services_health() -> ServiceHealth {
    let cfg = AppConfig::load();
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let ollama_base = cfg
        .ollama_url
        .split("/api/")
        .next()
        .unwrap_or("http://127.0.0.1:11434");
    let hermes_base = cfg
        .hermes_url
        .split("/v1/")
        .next()
        .unwrap_or("http://127.0.0.1:8642");

    let o_res = client.get(format!("{ollama_base}/api/tags")).send().await;
    let h_res = client.get(format!("{hermes_base}/health")).send().await;

    ServiceHealth {
        ollama_ok: o_res.map(|r| r.status().is_success()).unwrap_or(false),
        hermes_ok: h_res.map(|r| r.status().is_success()).unwrap_or(false),
        fish_configured: cfg.tts_enabled == "1" && !cfg.fish_api_key.is_empty(),
    }
}
