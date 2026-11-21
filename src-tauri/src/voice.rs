use crate::config::{find_project_root, AppConfig};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
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
}

impl VoiceManager {
    pub fn new() -> Self {
        Self {
            whisper: Mutex::new(None),
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

#[tauri::command]
pub async fn init_voice(manager: State<'_, Arc<VoiceManager>>) -> Result<VoiceInitStatus, String> {
    let mgr_w = Arc::clone(manager.inner());
    let cfg = AppConfig::load();

    let w_res = tauri::async_runtime::spawn_blocking(move || mgr_w.ensure_whisper())
        .await
        .map_err(|e| format!("Erro interno Whisper: {e}"))?;
    w_res?;

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

#[tauri::command]
pub fn stop_voice(manager: State<'_, Arc<VoiceManager>>) {
    manager.shutdown();
}
