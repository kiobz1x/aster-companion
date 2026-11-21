use crate::config::find_project_root;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_FACTS: usize = 50;

#[derive(Debug, Default, Serialize, Deserialize)]
struct MemoryStore {
    facts: Vec<String>,
}

fn memory_file_path() -> Result<PathBuf, String> {
    if let Some(root) = find_project_root() {
        return Ok(root.join("memory.json"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return Ok(dir.join("memory.json"));
        }
    }
    std::env::current_dir()
        .map(|d| d.join("memory.json"))
        .map_err(|e| format!("Não foi possível localizar pasta para memory.json: {e}"))
}

fn load_store() -> Result<MemoryStore, String> {
    let path = memory_file_path()?;
    if !path.is_file() {
        return Ok(MemoryStore::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Erro lendo memory.json: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("memory.json inválido: {e}"))
}

fn save_store(store: &MemoryStore) -> Result<(), String> {
    let path = memory_file_path()?;
    let json =
        serde_json::to_string_pretty(store).map_err(|e| format!("Erro serializando memória: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Erro salvando memory.json: {e}"))
}

#[tauri::command]
pub fn memory_list() -> Result<Vec<String>, String> {
    Ok(load_store()?.facts)
}

#[tauri::command]
pub fn memory_add(fact: String) -> Result<Vec<String>, String> {
    let clean = fact.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        return Err("Fato vazio.".into());
    }

    let mut store = load_store()?;
    let lower = clean.to_lowercase();
    if !store.facts.iter().any(|f| f.to_lowercase() == lower) {
        store.facts.push(clean);
        if store.facts.len() > MAX_FACTS {
            let excess = store.facts.len() - MAX_FACTS;
            store.facts.drain(0..excess);
        }
        save_store(&store)?;
    }
    Ok(store.facts)
}

#[tauri::command]
pub fn memory_remove(query: String) -> Result<usize, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    let mut store = load_store()?;
    let before = store.facts.len();

    // Permite remover pelo número (1-based) ou por texto contido no fato
    if let Ok(idx) = trimmed.parse::<usize>() {
        if idx >= 1 && idx <= store.facts.len() {
            store.facts.remove(idx - 1);
            save_store(&store)?;
            return Ok(1);
        }
        return Ok(0);
    }

    let q_low = trimmed.to_lowercase();
    store.facts.retain(|f| !f.to_lowercase().contains(&q_low));
    let removed = before - store.facts.len();
    if removed > 0 {
        save_store(&store)?;
    }
    Ok(removed)
}

#[tauri::command]
pub fn memory_clear() -> Result<(), String> {
    save_store(&MemoryStore::default())
}
