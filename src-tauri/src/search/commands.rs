use super::embed::Embedder;
use super::store::{SearchHit, SearchIndex};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct SearchResult {
    pub message_id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub distance: f32,
}

impl From<SearchHit> for SearchResult {
    fn from(hit: SearchHit) -> Self {
        Self {
            message_id: hit.message_id,
            conversation_id: hit.conversation_id,
            role: hit.role,
            content: hit.content,
            distance: hit.distance,
        }
    }
}

/// Embeds `query` and returns the most similar indexed messages, most
/// similar first. Only messages sent since this feature shipped are indexed
/// - there's no backfill of older conversations yet.
#[tauri::command]
pub async fn search_conversations(app: AppHandle, index: State<'_, SearchIndex>, query: String) -> Result<Vec<SearchResult>, String> {
    let app_for_embed = app.clone();
    let query_vector = tokio::task::spawn_blocking(move || {
        let embedder = app_for_embed.state::<Embedder>();
        super::embed::embed_blocking(&app_for_embed, &embedder, &query)
    })
    .await
    .map_err(|e| format!("embedding task panicked: {e}"))??;

    let hits = super::store::search(&app, &index, query_vector, 10).await?;
    Ok(hits.into_iter().map(SearchResult::from).collect())
}
