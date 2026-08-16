mod accounts;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(accounts::init_state(app.handle()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            accounts::commands::list_accounts,
            accounts::commands::create_account,
            accounts::commands::update_account,
            accounts::commands::test_account,
            accounts::commands::oidc_login,
            accounts::commands::refresh_oidc_session,
            accounts::commands::delete_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
