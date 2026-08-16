// `commands` must stay a visible submodule (not re-exported functions): the
// `#[tauri::command]` macro emits hidden sibling items next to each function
// that `tauri::generate_handler!` looks up by path, and `pub use`-ing just the
// function name would leave those behind.
pub mod commands;
mod model;
mod providers;
mod secrets;
mod store;

use std::sync::Mutex;
use tauri::AppHandle;

#[allow(unused_imports)] // re-exported for later phases (events, session refresh) to reference
pub use model::Account;
pub use store::AccountsState;

pub fn init_state(app: &AppHandle) -> AccountsState {
    AccountsState(Mutex::new(store::load(app)))
}
