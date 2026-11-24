use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;
use tauri::{Emitter, Manager};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn local_http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

fn external_http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

/// Garante que o Ollama local esteja rodando em segundo plano (sem janela de terminal).
pub async fn ensure_ollama_running() -> bool {
    let cfg = AppConfig::load();
    let is_local = cfg.ollama_url.contains("127.0.0.1") || cfg.ollama_url.contains("localhost");
    if !is_local {
        return true;
    }

    let base = cfg
        .ollama_url
        .split("/api/")
        .next()
        .unwrap_or("http://127.0.0.1:11434");
    let tags_url = format!("{}/api/tags", base.trim_end_matches('/'));

    let client = local_http();

    if client
        .get(&tags_url)
        .timeout(Duration::from_millis(1200))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
    {
        return true;
    }

    // Ollama desligado -> sobe `ollama serve` escondido em segundo plano
    let mut cmd = Command::new("ollama");
    cmd.arg("serve");
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    if cmd.spawn().is_err() {
        return false;
    }

    // Aguarda brevemente o servidor subir (até ~3s)
    for _ in 0..6 {
        let _ = tauri::async_runtime::spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(500));
        })
        .await;
        if client
            .get(&tags_url)
            .timeout(Duration::from_millis(1200))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

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
    let client = local_http();

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
        .timeout(Duration::from_secs(60))
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
pub async fn chat_ollama_stream(
    app: tauri::AppHandle,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    num_ctx: Option<u32>,
) -> Result<String, String> {
    let cfg = AppConfig::load();
    let client = local_http();

    let payload = json!({
        "model": cfg.ollama_model,
        "messages": messages,
        "stream": true,
        "think": false,
        "options": {
            "temperature": temperature.unwrap_or(0.7),
            "num_ctx": num_ctx.unwrap_or(2048)
        }
    });

    let mut res = match client
        .post(&cfg.ollama_url)
        .timeout(Duration::from_secs(90))
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) if e.is_connect() => {
            // Tenta subir o Ollama automaticamente em segundo plano e reconectar 1x
            if ensure_ollama_running().await {
                client
                    .post(&cfg.ollama_url)
                    .timeout(Duration::from_secs(90))
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e2| format!("Ollama inacessível após auto-start: {e2}"))?
            } else {
                return Err(format!("Ollama desligado ou inacessível em {}.", cfg.ollama_url));
            }
        }
        Err(e) => {
            return Err(if e.is_timeout() {
                "Ollama demorou demais para responder (timeout).".to_string()
            } else {
                format!("Erro ao falar com Ollama: {e}")
            });
        }
    };

    if !res.status().is_success() {
        return Err(format!("Ollama HTTP {}", res.status()));
    }

    let main_win = app.get_webview_window("main");
    let mut full = String::new();
    let mut line_buf = String::new();

    while let Some(chunk) = res
        .chunk()
        .await
        .map_err(|e| format!("Erro lendo stream do Ollama: {e}"))?
    {
        let text = String::from_utf8_lossy(&chunk);
        line_buf.push_str(&text);

        while let Some(nl) = line_buf.find('\n') {
            let line = line_buf[..nl].trim().to_string();
            line_buf.drain(..=nl);
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(piece) = v["message"]["content"].as_str() {
                    if !piece.is_empty() {
                        full.push_str(piece);
                        if let Some(win) = &main_win {
                            let _ = win.emit("ollama-chunk", piece);
                        }
                    }
                }
            }
        }
    }

    // Resto eventual sem \n no final
    let tail = line_buf.trim();
    if !tail.is_empty() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(tail) {
            if let Some(piece) = v["message"]["content"].as_str() {
                if !piece.is_empty() {
                    full.push_str(piece);
                    if let Some(win) = &main_win {
                        let _ = win.emit("ollama-chunk", piece);
                    }
                }
            }
        }
    }

    Ok(strip_think_tags(&full))
}

#[cfg(windows)]
fn capture_screen_base64() -> Result<String, String> {
    let ps_script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$g.Dispose()

$maxW = 1280
if ($bmp.Width -gt $maxW) {
    $newH = [int]([double]$bmp.Height * ($maxW / [double]$bmp.Width))
    $resized = New-Object System.Drawing.Bitmap($maxW, $newH)
    $g2 = [System.Drawing.Graphics]::FromImage($resized)
    $g2.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g2.DrawImage($bmp, 0, 0, $maxW, $newH)
    $g2.Dispose()
    $bmp.Dispose()
    $bmp = $resized
}

$ms = New-Object System.IO.MemoryStream
$codec = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() | Where-Object { $_.MimeType -eq 'image/jpeg' } | Select-Object -First 1
$ep = New-Object System.Drawing.Imaging.EncoderParameters(1)
$ep.Param[0] = New-Object System.Drawing.Imaging.EncoderParameter([System.Drawing.Imaging.Encoder]::Quality, [long]78)
$bmp.Save($ms, $codec, $ep)
$bmp.Dispose()
[Convert]::ToBase64String($ms.ToArray())
$ms.Dispose()
"#;

    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", ps_script]);
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd
        .output()
        .map_err(|e| format!("Falha ao capturar tela: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("Erro capturando tela: {}", err.trim()));
    }
    let b64 = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if b64.is_empty() {
        return Err("Screenshot retornou vazio.".into());
    }
    Ok(b64)
}

#[tauri::command]
pub async fn ask_ollama_vision(prompt: String, system_prompt: String) -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let _ = (prompt, system_prompt);
        return Err("Visão de tela disponível apenas no Windows.".into());
    }

    #[cfg(windows)]
    {
        let cfg = AppConfig::load();
        let vision_model = if cfg.ollama_vision_model.trim().is_empty() {
            "qwen2.5vl:3b".to_string()
        } else {
            cfg.ollama_vision_model.trim().to_string()
        };

        let img_b64 = tauri::async_runtime::spawn_blocking(capture_screen_base64)
            .await
            .map_err(|e| format!("Erro na thread de screenshot: {e}"))??;

        let client = local_http();

        let payload = json!({
            "model": vision_model,
            "stream": false,
            "think": false,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": prompt, "images": [img_b64] }
            ],
            "options": {
                "temperature": 0.6,
                "num_ctx": 2048
            }
        });

        let res = match client
            .post(&cfg.ollama_url)
            .timeout(Duration::from_secs(120))
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) if e.is_connect() => {
                if ensure_ollama_running().await {
                    client
                        .post(&cfg.ollama_url)
                        .timeout(Duration::from_secs(120))
                        .json(&payload)
                        .send()
                        .await
                        .map_err(|e2| format!("Ollama inacessível após auto-start: {e2}"))?
                } else {
                    return Err(format!("Ollama desligado em {}.", cfg.ollama_url));
                }
            }
            Err(e) => return Err(format!("Erro ao enviar print pro Ollama: {e}")),
        };

        if !res.status().is_success() {
            let status = res.status();
            let err_txt = res.text().await.unwrap_or_default();
            if status.as_u16() == 404 || err_txt.contains("not found") {
                return Err(format!(
                    "Modelo de visão '{vision_model}' não instalado no Ollama. Rode no terminal: ollama pull {vision_model} (ou troque em Configurações -> IA)."
                ));
            }
            return Err(format!("Ollama Visão HTTP {status}: {}", err_txt.chars().take(120).collect::<String>()));
        }

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Resposta inválida do Ollama Visão: {e}"))?;

        let content = body["message"]["content"].as_str().unwrap_or("");
        Ok(strip_think_tags(content))
    }
}

#[tauri::command]
pub async fn ask_hermes(task: String) -> Result<String, String> {
    let cfg = AppConfig::load();
    let client = local_http();

    let payload = json!({
        "model": cfg.hermes_model,
        "stream": false,
        "messages": [
            {
                "role": "system",
                "content": "Você é Hermes executando uma tarefa delegada pela Aster. Responda de forma objetiva e concisa em PT-BR."
            },
            {
                "role": "user",
                "content": task
            }
        ]
    });

    let mut req = client
        .post(&cfg.hermes_url)
        .timeout(Duration::from_secs(180))
        .json(&payload);
    if !cfg.hermes_key.is_empty() && cfg.hermes_key != "cole_sua_hermes_key_aqui" {
        req = req.header("Authorization", format!("Bearer {}", cfg.hermes_key));
    }

    let res = req
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                format!("Hermes desligado ou porta fechada ({}). Verifique se o api_server do Hermes está rodando.", cfg.hermes_url)
            } else if e.is_timeout() {
                "Hermes conectou, mas demorou mais de 3 min sem responder (timeout).".to_string()
            } else {
                format!("Erro de conexão com Hermes ({}): {e}", cfg.hermes_url)
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

    let client = external_http();

    let res = client
        .post(&cfg.discord_webhook_url)
        .timeout(Duration::from_secs(15))
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
pub async fn list_ollama_models() -> Result<Vec<String>, String> {
    let cfg = AppConfig::load();
    let base = cfg
        .ollama_url
        .split("/api/")
        .next()
        .unwrap_or("http://127.0.0.1:11434");
    let tags_url = format!("{}/api/tags", base.trim_end_matches('/'));

    let client = local_http();

    let res = client
        .get(&tags_url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|e| format!("Ollama inacessível: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("Ollama HTTP {}", res.status()));
    }

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Resposta inválida: {e}"))?;

    let mut models: Vec<String> = body["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();
    Ok(models)
}

#[tauri::command]
pub async fn check_services_health() -> ServiceHealth {
    let cfg = AppConfig::load();
    let client = local_http();

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

    let o_res = client
        .get(format!("{ollama_base}/api/tags"))
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    let h_res = client
        .get(format!("{hermes_base}/health"))
        .timeout(Duration::from_secs(2))
        .send()
        .await;

    ServiceHealth {
        ollama_ok: o_res.map(|r| r.status().is_success()).unwrap_or(false),
        hermes_ok: h_res.map(|r| r.status().is_success()).unwrap_or(false),
        fish_configured: cfg.tts_enabled == "1" && !cfg.fish_api_key.is_empty(),
    }
}
