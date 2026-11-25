use crate::config::find_project_root;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const ALLOWED_EXTS: &[&str] = &["txt", "md", "markdown", "json", "csv", "log"];

const STOPWORDS: &[&str] = &[
    "a", "o", "as", "os", "um", "uma", "uns", "umas", "de", "do", "da", "dos", "das", "em", "no",
    "na", "nos", "nas", "por", "para", "pra", "pro", "com", "sem", "que", "qual", "quais", "como",
    "onde", "quando", "porque", "porquê", "sobre", "isso", "isto", "esse", "essa", "este", "esta",
    "meu", "minha", "meus", "minhas", "seu", "sua", "ter", "tem", "foi", "ser", "esta", "está",
    "estou", "sao", "são", "era", "mais", "menos", "muito", "muita", "tambem", "também", "voce",
    "você", "aster", "me", "te", "se", "eu", "ele", "ela", "nos", "nós", "eles", "elas", "aqui",
    "ali", "la", "lá", "fala", "diz", "mostra", "olha", "ve", "vê",
];

#[derive(Debug, Clone, Serialize)]
pub struct DocInfo {
    pub name: String,
    pub size_kb: u64,
}

fn docs_dir_path() -> Result<PathBuf, String> {
    let base = if let Some(root) = find_project_root() {
        root
    } else if let Ok(exe) = std::env::current_exe() {
        exe.parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Pasta do executável não encontrada.".to_string())?
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?
    };

    let dir = base.join("docs");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Erro criando pasta docs/: {e}"))?;
    }
    Ok(dir)
}

fn is_supported_doc(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| ALLOWED_EXTS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
}

fn normalize_text(s: &str) -> String {
    s.to_lowercase()
        .replace('ç', "c")
        .replace(['á', 'à', 'ã', 'â'], "a")
        .replace(['é', 'ê', 'è'], "e")
        .replace(['í', 'ì', 'î'], "i")
        .replace(['ó', 'õ', 'ô', 'ò'], "o")
        .replace(['ú', 'ù', 'û'], "u")
}

fn stem_pt(word: &str) -> String {
    let suffixes = [
        "amentos", "imentos", "amento", "imento", "acoes", "icoes", "acao", "icao",
        "ando", "endo", "indo", "mente", "idades", "idade", "istas", "ista",
        "ores", "oras", "oes", "ais", "eis", "ois", "es", "s",
    ];
    for suf in suffixes {
        if word.len() > suf.len() + 3 && word.ends_with(suf) {
            return word[..word.len() - suf.len()].to_string();
        }
    }
    word.to_string()
}

fn extract_keywords(text: &str) -> Vec<String> {
    let norm = normalize_text(text);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for tok in norm.split(|c: char| !c.is_alphanumeric()) {
        if tok.len() >= 3 && !STOPWORDS.contains(&tok) {
            if seen.insert(tok.to_string()) {
                out.push(tok.to_string());
            }
            let st = stem_pt(tok);
            if st.len() >= 4 && st != tok && seen.insert(st.clone()) {
                out.push(st);
            }
        }
    }
    out
}

fn split_chunks(content: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in content.split("\n\n") {
        let p = para.trim();
        if p.is_empty() {
            continue;
        }
        if !current.is_empty() && current.len() + p.len() > 480 {
            chunks.push(current.trim().to_string());
            current.clear();
        }
        if p.len() > 520 {
            for line in p.lines() {
                let l = line.trim();
                if l.is_empty() {
                    continue;
                }
                if !current.is_empty() && current.len() + l.len() > 480 {
                    chunks.push(current.trim().to_string());
                    current.clear();
                }
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(l);
            }
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(p);
        }
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks
}

#[tauri::command]
pub fn docs_list() -> Result<Vec<DocInfo>, String> {
    let dir = docs_dir_path()?;
    let mut list = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("Erro lendo docs/: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_supported_doc(&path) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let size_kb = entry.metadata().map(|m| (m.len() + 1023) / 1024).unwrap_or(0);
        list.push(DocInfo { name, size_kb });
    }
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
pub fn docs_open_folder() -> Result<(), String> {
    let dir = docs_dir_path()?;
    #[cfg(windows)]
    {
        let mut cmd = Command::new("explorer");
        cmd.arg(&dir);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn()
            .map_err(|e| format!("Erro abrindo pasta docs/: {e}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = dir;
        Err("Disponível apenas no Windows.".into())
    }
}

#[tauri::command]
pub fn docs_search(query: String, max_chars: Option<usize>) -> Result<String, String> {
    let keywords = extract_keywords(&query);
    if keywords.is_empty() {
        return Ok(String::new());
    }

    let dir = docs_dir_path()?;
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(String::new()),
    };

    let mut scored: Vec<(usize, String, String)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_supported_doc(&path) {
            continue;
        }
        // Ignora arquivos enormes (>2 MB) para manter busca instantânea
        if entry.metadata().map(|m| m.len() > 2 * 1024 * 1024).unwrap_or(true) {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let fname = entry.file_name().to_string_lossy().to_string();
        let fname_norm = normalize_text(&fname);

        for chunk in split_chunks(&raw) {
            let chunk_norm = normalize_text(&chunk);
            let mut score = 0usize;
            for kw in &keywords {
                if chunk_norm.contains(kw.as_str()) {
                    score += 2;
                } else if fname_norm.contains(kw.as_str()) {
                    score += 1;
                }
            }
            if score >= 2 {
                scored.push((score, fname.clone(), chunk));
            }
        }
    }

    if scored.is_empty() {
        return Ok(String::new());
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let budget = max_chars.unwrap_or(1200);
    let mut result = String::new();

    for (_score, fname, chunk) in scored.into_iter().take(3) {
        let snippet = format!("[{fname}]: {chunk}\n");
        if result.len() + snippet.len() > budget && !result.is_empty() {
            break;
        }
        result.push_str(&snippet);
    }

    Ok(result.trim().to_string())
}
