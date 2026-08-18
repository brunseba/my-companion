use super::model::Conversation;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct ChatState(pub Mutex<Vec<Conversation>>);

/// Exposed (not just used internally) so Diagnostics can report the
/// conversation data file's size - same reasoning as `accounts::accounts_file`.
pub fn conversations_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    Ok(dir.join("conversations.json"))
}

pub fn load(app: &AppHandle) -> Vec<Conversation> {
    let Ok(path) = conversations_file(app) else {
        return Vec::new();
    };
    let Ok(bytes) = fs::read(&path) else {
        return Vec::new();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save(app: &AppHandle, conversations: &[Conversation]) -> Result<(), String> {
    let path = conversations_file(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create app data dir: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(conversations)
        .map_err(|e| format!("failed to serialize conversations: {e}"))?;
    fs::write(&path, bytes).map_err(|e| format!("failed to write conversations file: {e}"))
}
