// Same reasoning as accounts::commands: must stay a visible submodule, not
// re-exported functions, so `tauri::generate_handler!` can find the hidden
// items the `#[tauri::command]` macro emits alongside each function.
pub mod commands;
mod model;
mod store;
mod stream;

use std::sync::Mutex;
use tauri::AppHandle;

pub use store::{conversations_file, ChatState};

pub fn init_state(app: &AppHandle) -> ChatState {
    ChatState(Mutex::new(store::load(app)))
}
