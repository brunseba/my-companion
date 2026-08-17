use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    /// "user" or "assistant" - matches both OpenAI's and Anthropic's role
    /// strings directly, so no translation is needed when building a request.
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// A conversation is pinned to one AI account (and therefore one provider)
/// for its whole lifetime - switching providers mid-conversation would mean
/// switching API semantics, so that's a new conversation instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub account_id: String,
    pub model: String,
    /// Empty until the first message is sent, then set from its content.
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationInput {
    pub account_id: String,
    pub model: String,
}
