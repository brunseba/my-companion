use super::model::{ChatMessage, Conversation, CreateConversationInput};
use super::store::{self, ChatState};
use super::stream::stream_reply;
use crate::accounts::{self, AccountsState};
use chrono::Utc;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command]
pub fn list_conversations(state: State<ChatState>) -> Vec<Conversation> {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn create_conversation(
    app: AppHandle,
    state: State<ChatState>,
    input: CreateConversationInput,
) -> Result<Conversation, String> {
    let now = Utc::now().to_rfc3339();
    let conversation = Conversation {
        id: Uuid::new_v4().to_string(),
        account_id: input.account_id,
        model: input.model,
        title: String::new(),
        messages: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut conversations = state.0.lock().unwrap();
    conversations.push(conversation.clone());
    store::save(&app, &conversations)?;
    Ok(conversation)
}

#[tauri::command]
pub fn delete_conversation(app: AppHandle, state: State<ChatState>, id: String) -> Result<(), String> {
    let mut conversations = state.0.lock().unwrap();
    let before = conversations.len();
    conversations.retain(|c| c.id != id);
    if conversations.len() == before {
        return Err(format!("no conversation with id {id}"));
    }
    store::save(&app, &conversations)
}

/// First ~48 characters of the opening message, used as the conversation's
/// title once it has one.
fn title_from(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.chars().count() <= 48 {
        trimmed.to_string()
    } else {
        let truncated: String = trimmed.chars().take(48).collect();
        format!("{truncated}…")
    }
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    chat_state: State<'_, ChatState>,
    accounts_state: State<'_, AccountsState>,
    conversation_id: String,
    content: String,
) -> Result<ChatMessage, String> {
    let user_message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content,
        created_at: Utc::now().to_rfc3339(),
    };

    // Snapshot everything the API call needs up front - both mutex guards
    // have to be released before the `.await` below, and locking the
    // account/secret here (rather than during the request) avoids holding
    // them for as long as a slow network call takes.
    let (provider, base_url, model, api_key, history) = {
        let mut conversations = chat_state.0.lock().unwrap();
        let conversation = conversations
            .iter_mut()
            .find(|c| c.id == conversation_id)
            .ok_or_else(|| format!("no conversation with id {conversation_id}"))?;
        conversation.messages.push(user_message);
        conversation.updated_at = Utc::now().to_rfc3339();
        if conversation.title.is_empty() {
            conversation.title = title_from(&conversation.messages[0].content);
        }
        let account_id = conversation.account_id.clone();
        let model = conversation.model.clone();
        let history = conversation.messages.clone();

        let accounts = accounts_state.0.lock().unwrap();
        let account = accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or("this conversation's account no longer exists")?;
        let provider = account.provider.clone();
        let base_url = account.config.get("base_url").and_then(|v| v.as_str()).map(str::to_owned);
        drop(accounts);

        let api_key = accounts::get_account_secret(&account_id)?
            .and_then(|s| s.get("api_key").and_then(|v| v.as_str()).map(str::to_owned))
            .ok_or_else(|| "no API key stored for this account".to_string())?;

        store::save(&app, &conversations)?;
        (provider, base_url, model, api_key, history)
    };

    let text = stream_reply(&app, &conversation_id, &provider, base_url.as_deref(), &api_key, &model, &history).await?;

    let mut conversations = chat_state.0.lock().unwrap();
    let conversation = conversations
        .iter_mut()
        .find(|c| c.id == conversation_id)
        .ok_or_else(|| format!("no conversation with id {conversation_id}"))?;
    let assistant_message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: text,
        created_at: Utc::now().to_rfc3339(),
    };
    conversation.messages.push(assistant_message.clone());
    conversation.updated_at = Utc::now().to_rfc3339();
    store::save(&app, &conversations)?;
    Ok(assistant_message)
}
