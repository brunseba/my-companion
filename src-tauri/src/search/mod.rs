// `commands` must stay a visible submodule, not re-exported functions - same
// reasoning as accounts::commands and chat::commands.
pub mod commands;
mod embed;
mod store;

use tauri::{AppHandle, Manager};

pub use embed::Embedder;
pub use store::SearchIndex;

/// Embeds and indexes one chat message. Best-effort by design: callers
/// (`chat::commands::send_message`) run this in a spawned background task
/// and are expected to log-and-ignore failures rather than let a search
/// indexing problem affect the chat reply itself.
pub async fn index_message(
    app: &AppHandle,
    message_id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
    created_at: &str,
) -> Result<(), String> {
    let app_for_embed = app.clone();
    let content_owned = content.to_string();
    let vector = tokio::task::spawn_blocking(move || {
        let embedder = app_for_embed.state::<Embedder>();
        embed::embed_blocking(&app_for_embed, &embedder, &content_owned)
    })
    .await
    .map_err(|e| format!("embedding task panicked: {e}"))??;

    let index = app.state::<SearchIndex>();
    store::insert(app, &index, message_id, conversation_id, role, content, created_at, vector).await
}

/// Removes every indexed message for a deleted conversation.
pub async fn delete_conversation(app: &AppHandle, conversation_id: &str) -> Result<(), String> {
    let index = app.state::<SearchIndex>();
    store::delete_conversation(app, &index, conversation_id).await
}
