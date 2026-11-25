use crate::config::{find_project_root, AppConfig};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

static TMP_SEQ: AtomicU64 = AtomicU64::new(1);

struct PyWorker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl PyWorker {
    fn spawn(script_name: &str, ready_token: &str, cfg: &AppConfig) -> Result<Self, String> {
        let root = find_project_root()
            .ok_or_else(|| "Pasta do projeto (transcrever.py / .venv) não encontrada.".to_string())?;
        let script_path = root.join(script_name);
        if !script_path.is_file() {
            return Err(format!("Script não encontrado: {}", script_path.display()));
        }

        let venv_python = root.join(".venv").join("Scripts").join("python.exe");
        let python_exe: PathBuf = if venv_python.is_file() {
            venv_python
        } else {
            PathBuf::from("python")
        };

        let mut cmd = Command::new(&python_exe);
        cmd.arg("-u")
            .arg(&script_path)
            .current_dir(&root)
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONUTF8", "1")
            .env("ASTER_WHISPER_MODEL", &cfg.whisper_model)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Falha ao iniciar {script_name} com {}: {e}", python_exe.display()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| format!("Sem stdin no worker {script_name}"))?;
        let stdout_raw = child
            .stdout
            .take()
            .ok_or_else(|| format!("Sem stdout no worker {script_name}"))?;
        let mut stdout = BufReader::new(stdout_raw);

        let mut line = String::new();
        loop {
            line.clear();
            let n = stdout
                .read_line(&mut line)
                .map_err(|e| format!("Erro lendo {script_name}: {e}"))?;
            if n == 0 {
                let _ = child.kill();
                return Err(format!(
                    "Worker {script_name} encerrou antes de ficar pronto (verifique .venv / dependências)."
                ));
            }
            if line.trim() == ready_token {
                break;
            }
        }

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn request(&mut self, payload: &str) -> Result<String, String> {
        writeln!(self.stdin, "{payload}")
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("Erro enviando dados ao worker: {e}"))?;

        let mut line = String::new();
        let n = self
            .stdout
            .read_line(&mut line)
            .map_err(|e| format!("Erro lendo resposta do worker: {e}"))?;
        if n == 0 {
            return Err("Worker Python encerrou inesperadamente.".into());
        }
        Ok(line.trim().to_string())
    }
}

impl Drop for PyWorker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct VoiceManager {
    whisper: Mutex<Option<PyWorker>>,
    http: reqwest::Client,
}

impl VoiceManager {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            whisper: Mutex::new(None),
            http,
        }
    }

    fn ensure_whisper(&self) -> Result<(), String> {
        let mut guard = self.whisper.lock().map_err(|_| "Lock Whisper falhou".to_string())?;
        if let Some(w) = guard.as_mut() {
            if w.is_alive() {
                return Ok(());
            }
        }
        *guard = None;
        let cfg = AppConfig::load();
        let worker = PyWorker::spawn("transcrever.py", "READY", &cfg)?;
        *guard = Some(worker);
        Ok(())
    }

    pub fn shutdown(&self) {
        if let Ok(mut w) = self.whisper.lock() {
            *w = None;
        }
    }
}

#[derive(Serialize)]
pub struct VoiceInitStatus {
    pub whisper_model: String,
    pub whisper_ready: bool,
}

fn next_temp_path(prefix: &str, ext: &str) -> PathBuf {
    let id = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("{prefix}_{pid}_{id}.{ext}"))
}

fn has_groq_stt(cfg: &AppConfig) -> bool {
    !cfg.cloud_api_key.is_empty()
        && (cfg.cloud_url.contains("api.groq.com") || cfg.cloud_api_key.starts_with("gsk_"))
}

#[tauri::command]
pub async fn init_voice(manager: State<'_, Arc<VoiceManager>>) -> Result<VoiceInitStatus, String> {
    let cfg = AppConfig::load();
    if has_groq_stt(&cfg) {
        return Ok(VoiceInitStatus {
            whisper_model: "groq-whisper-v3-turbo".to_string(),
            whisper_ready: true,
        });
    }

    let mgr_w = Arc::clone(manager.inner());
    let w_res = tauri::async_runtime::spawn_blocking(move || mgr_w.ensure_whisper())
        .await
        .map_err(|e| format!("Erro interno Whisper: {e}"))?;
    w_res.map_err(|e| {
        format!("{e} — Dica: configure uma chave grátis do Groq em Cérebro (IA) para usar o microfone sem precisar de Python!")
    })?;

    Ok(VoiceInitStatus {
        whisper_model: cfg.whisper_model,
        whisper_ready: true,
    })
}

#[tauri::command]
pub async fn transcribe_audio(
    manager: State<'_, Arc<VoiceManager>>,
    wav_base64: String,
) -> Result<String, String> {
    let cfg = AppConfig::load();
    if has_groq_stt(&cfg) {
        let bytes = BASE64
            .decode(wav_base64.trim())
            .map_err(|e| format!("WAV inválido: {e}"))?;
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| format!("Erro MIME WAV: {e}"))?;
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", "whisper-large-v3-turbo")
            .text("language", "pt")
            .text("response_format", "json");

        let res = manager
            .http
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", cfg.cloud_api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Erro Groq Whisper: {e}"))?;

        if !res.status().is_success() {
            let st = res.status();
            let err_txt = res.text().await.unwrap_or_default();
            return Err(format!(
                "Groq Whisper HTTP {st}: {}",
                err_txt.chars().take(120).collect::<String>()
            ));
        }

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Resposta inválida Groq Whisper: {e}"))?;
        return Ok(body["text"].as_str().unwrap_or("").trim().to_string());
    }

    let mgr = Arc::clone(manager.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = BASE64
            .decode(wav_base64.trim())
            .map_err(|e| format!("WAV inválido: {e}"))?;

        let tmp_wav = next_temp_path("aster_stt", "wav");
        fs::write(&tmp_wav, &bytes).map_err(|e| format!("Erro gravando WAV temp: {e}"))?;

        let result = (|| -> Result<String, String> {
            mgr.ensure_whisper()?;
            let mut guard = mgr
                .whisper
                .lock()
                .map_err(|_| "Lock Whisper falhou".to_string())?;
            let worker = guard
                .as_mut()
                .ok_or_else(|| "Worker Whisper indisponível".to_string())?;

            let resp = match worker.request(&tmp_wav.to_string_lossy()) {
                Ok(r) => r,
                Err(e) => {
                    *guard = None;
                    return Err(e);
                }
            };

            if let Some(text) = resp.strip_prefix("RESULT:") {
                Ok(text.trim().to_string())
            } else if let Some(err) = resp.strip_prefix("ERROR:") {
                Err(err.trim().to_string())
            } else {
                Ok(String::new())
            }
        })();

        let _ = fs::remove_file(&tmp_wav);
        result
    })
    .await
    .map_err(|e| format!("Erro na thread STT: {e}"))?
}

#[derive(Serialize)]
pub struct AudioPayload {
    pub mime: String,
    pub audio_base64: String,
}

#[tauri::command]
pub async fn synthesize_fish_speech(
    manager: State<'_, Arc<VoiceManager>>,
    text: String,
) -> Result<Option<AudioPayload>, String> {
    let cfg = AppConfig::load();
    if cfg.tts_enabled != "1" || cfg.fish_api_key.is_empty() {
        return Ok(None);
    }

    let clean: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        return Ok(None);
    }

    let mut body = json!({
        "text": clean,
        "format": "mp3",
        "mp3_bitrate": 64,
        "chunk_length": 150,
        "normalize": false,
        "latency": "low"
    });
    if !cfg.fish_voice_id.is_empty() {
        body["reference_id"] = json!(cfg.fish_voice_id);
    }

    let model_header = if cfg.fish_model.is_empty() {
        "s1"
    } else {
        cfg.fish_model.as_str()
    };

    let res = manager
        .http
        .post("https://api.fish.audio/v1/tts")
        .header("Authorization", format!("Bearer {}", cfg.fish_api_key))
        .header("model", model_header)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Fish Audio inacessível: {e}"))?;

    let status = res.status();
    if !status.is_success() {
        let err_txt = res.text().await.unwrap_or_default();
        return Err(format!(
            "Fish Audio HTTP {status}: {}",
            err_txt.chars().take(120).collect::<String>()
        ));
    }

    let bytes = res
        .bytes()
        .await
        .map_err(|e| format!("Erro lendo áudio do Fish Audio: {e}"))?;
    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(AudioPayload {
        mime: "audio/mpeg".to_string(),
        audio_base64: BASE64.encode(&bytes),
    }))
}

#[tauri::command]
pub fn stop_voice(manager: State<'_, Arc<VoiceManager>>) {
    manager.shutdown();
}
