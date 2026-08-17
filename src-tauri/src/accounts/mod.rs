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
/// Exposed crate-wide (not just within `accounts`) so `diagnostics` can report
/// the account data file's size without reaching into `store`'s internals.
pub(crate) use store::accounts_file;
/// Exposed crate-wide so `chat` can read an AI account's API key without
/// reaching into `secrets`'s internals. Never exposed to the frontend.
pub(crate) use secrets::get as get_account_secret;

pub fn init_state(app: &AppHandle) -> AccountsState {
    AccountsState(Mutex::new(store::load(app)))
}
