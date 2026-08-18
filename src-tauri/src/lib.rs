mod accounts;
mod chat;
mod diagnostics;
mod search;

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
            app.manage(diagnostics::ResourceMonitor::new());
            app.manage(chat::init_state(app.handle()));
            app.manage(search::Embedder::new());
            app.manage(search::SearchIndex::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            accounts::commands::list_accounts,
            accounts::commands::create_account,
            accounts::commands::update_account,
            accounts::commands::test_account,
            accounts::commands::oauth_login,
            accounts::commands::refresh_oauth_session,
            accounts::commands::delete_account,
            accounts::commands::app_data_info,
            accounts::commands::reset_all_data,
            diagnostics::resource_usage,
            chat::commands::list_conversations,
            chat::commands::create_conversation,
            chat::commands::delete_conversation,
            chat::commands::send_message,
            search::commands::search_conversations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
