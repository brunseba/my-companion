use super::model::Account;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// In-memory account list, synced to disk on every mutation. Fine at the sizes
/// this app deals with (dozens of accounts, not thousands).
pub struct AccountsState(pub Mutex<Vec<Account>>);

/// Exposed (not just used internally) so the Settings page can show the user
/// exactly where their account metadata lives on disk.
pub fn accounts_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    Ok(dir.join("accounts.json"))
}

pub fn load(app: &AppHandle) -> Vec<Account> {
    let Ok(path) = accounts_file(app) else {
        return Vec::new();
    };
    let Ok(bytes) = fs::read(&path) else {
        return Vec::new();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save(app: &AppHandle, accounts: &[Account]) -> Result<(), String> {
    let path = accounts_file(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create app data dir: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(accounts)
        .map_err(|e| format!("failed to serialize accounts: {e}"))?;
    fs::write(&path, bytes).map_err(|e| format!("failed to write accounts file: {e}"))
}
