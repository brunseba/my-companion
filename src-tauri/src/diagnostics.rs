//! App-level resource monitoring for the Diagnostics page - how much RAM and
//! CPU *this app's own process* is using, how much disk space its own data
//! takes up, and (see `activity_stats`) how much of that data the newer
//! features (Chat, search) have actually accumulated. Not a system-wide
//! monitor.
use crate::accounts::accounts_file;
use crate::chat::{conversations_file, ChatState};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use sysinfo::{get_current_pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Manager, State};

/// A persistent `System` handle, so repeated refreshes can diff against the
/// previous sample - required for `cpu_usage()` to report anything other
/// than 0 (see sysinfo's docs: CPU usage is computed from the delta between
/// two refreshes, so the very first poll after launch always reads ~0%).
pub struct ResourceMonitor(Mutex<System>);

impl ResourceMonitor {
    pub fn new() -> Self {
        Self(Mutex::new(System::new()))
    }
}

#[derive(Serialize)]
pub struct ResourceUsage {
    memory_bytes: u64,
    cpu_percent: f32,
    accounts_file_bytes: u64,
    binary_bytes: u64,
}

fn file_size(path: Option<std::path::PathBuf>) -> u64 {
    path.and_then(|p| std::fs::metadata(p).ok()).map(|m| m.len()).unwrap_or(0)
}

/// Total size of every file under `path`, recursively. Used for the search
/// index and embedding model cache, which are directories (LanceDB datasets,
/// ONNX model files) rather than single files. `0` if `path` doesn't exist
/// yet - e.g. before anything has ever been indexed.
fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                dir_size(&path)
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            }
        })
        .sum()
}

#[tauri::command]
pub fn resource_usage(app: AppHandle, monitor: State<ResourceMonitor>) -> Result<ResourceUsage, String> {
    let pid = get_current_pid().map_err(|e| e.to_string())?;

    let mut sys = monitor.0.lock().unwrap();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
    let process = sys.process(pid).ok_or("process info unavailable")?;

    Ok(ResourceUsage {
        memory_bytes: process.memory(),
        cpu_percent: process.cpu_usage(),
        accounts_file_bytes: file_size(accounts_file(&app).ok()),
        binary_bytes: file_size(std::env::current_exe().ok()),
    })
}

#[derive(Serialize)]
pub struct ActivityStats {
    conversation_count: usize,
    message_count: usize,
    conversations_file_bytes: u64,
    indexed_message_count: u64,
    search_index_bytes: u64,
    embedding_model_bytes: u64,
}

/// Usage/consumption for the newer, data-heavier features (Chat, search) -
/// separate from `resource_usage` since these are about what's accumulated
/// over time, not a live process sample.
#[tauri::command]
pub async fn activity_stats(app: AppHandle, chat_state: State<'_, ChatState>) -> Result<ActivityStats, String> {
    let (conversation_count, message_count) = {
        let conversations = chat_state.0.lock().unwrap();
        let message_count = conversations.iter().map(|c| c.messages.len()).sum();
        (conversations.len(), message_count)
    };

    let indexed_message_count = crate::search::count_indexed(&app).await?;

    let data_dir = app.path().app_data_dir().map_err(|e| format!("failed to resolve app data dir: {e}"))?;

    Ok(ActivityStats {
        conversation_count,
        message_count,
        conversations_file_bytes: file_size(conversations_file(&app).ok()),
        indexed_message_count,
        search_index_bytes: dir_size(&data_dir.join("search_index")),
        embedding_model_bytes: dir_size(&data_dir.join("models")),
    })
}
