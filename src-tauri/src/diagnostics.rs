//! App-level resource monitoring for the Settings/Diagnostics page - how much
//! RAM and CPU *this app's own process* is using, plus how much disk space
//! its own data (not the whole system's) takes up. Not a system-wide monitor.
use crate::accounts::accounts_file;
use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{get_current_pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, State};

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
