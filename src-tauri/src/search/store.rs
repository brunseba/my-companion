//! The vector index itself: one LanceDB table (`messages`) at
//! `<app data dir>/search_index/`, holding one row per indexed chat message.
use super::embed::EMBEDDING_DIM;
use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures_util::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{Connection, Table};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

const TABLE_NAME: &str = "messages";

/// Holds the LanceDB connection, opened lazily on first use (connecting is
/// async, so it can't happen synchronously in Tauri's `.manage()` setup).
pub struct SearchIndex(Mutex<Option<Connection>>);

impl SearchIndex {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

pub struct SearchHit {
    pub message_id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    /// Vector distance - lower is more similar. Not normalized to a 0-1
    /// score; callers should treat it as a ranking signal, not a percentage.
    pub distance: f32,
}

fn message_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("message_id", DataType::Utf8, false),
        Field::new("conversation_id", DataType::Utf8, false),
        Field::new("role", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), EMBEDDING_DIM),
            false,
        ),
    ]))
}

async fn table(app: &AppHandle, index: &SearchIndex) -> Result<Table, String> {
    let mut guard = index.0.lock().await;
    if guard.is_none() {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("failed to resolve app data dir: {e}"))?
            .join("search_index");
        let path = dir.to_str().ok_or("app data path is not valid UTF-8")?;
        let connection = lancedb::connect(path)
            .execute()
            .await
            .map_err(|e| format!("failed to open search index: {e}"))?;
        *guard = Some(connection);
    }
    let connection = guard.as_ref().unwrap();

    let existing = connection
        .table_names()
        .execute()
        .await
        .map_err(|e| format!("failed to list search index tables: {e}"))?;
    if existing.iter().any(|name| name == TABLE_NAME) {
        connection
            .open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| format!("failed to open search index table: {e}"))
    } else {
        connection
            .create_empty_table(TABLE_NAME, message_schema())
            .execute()
            .await
            .map_err(|e| format!("failed to create search index table: {e}"))
    }
}

fn record_batch(
    message_id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
    created_at: &str,
    vector: Vec<f32>,
) -> Result<RecordBatch, String> {
    let embedding = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vec![Some(vector.into_iter().map(Some).collect::<Vec<_>>())],
        EMBEDDING_DIM,
    );
    RecordBatch::try_new(
        message_schema(),
        vec![
            Arc::new(StringArray::from(vec![message_id])),
            Arc::new(StringArray::from(vec![conversation_id])),
            Arc::new(StringArray::from(vec![role])),
            Arc::new(StringArray::from(vec![content])),
            Arc::new(StringArray::from(vec![created_at])),
            Arc::new(embedding),
        ],
    )
    .map_err(|e| format!("failed to build search index row: {e}"))
}

pub async fn insert(
    app: &AppHandle,
    index: &SearchIndex,
    message_id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
    created_at: &str,
    vector: Vec<f32>,
) -> Result<(), String> {
    let tbl = table(app, index).await?;
    let batch = record_batch(message_id, conversation_id, role, content, created_at, vector)?;
    tbl.add(batch)
        .execute()
        .await
        .map(|_| ())
        .map_err(|e| format!("failed to index message: {e}"))
}

/// Removes every indexed message belonging to `conversation_id` - called when
/// the conversation itself is deleted, so search results never point at a
/// conversation that no longer exists.
pub async fn delete_conversation(app: &AppHandle, index: &SearchIndex, conversation_id: &str) -> Result<(), String> {
    let tbl = table(app, index).await?;
    // conversation_id is always a UUID we generated ourselves (see
    // chat::commands), never user-supplied text, so this is safe from SQL
    // injection without needing to escape it.
    let predicate = format!("conversation_id = '{conversation_id}'");
    tbl.delete(&predicate)
        .await
        .map(|_| ())
        .map_err(|e| format!("failed to remove indexed messages: {e}"))
}

pub async fn search(
    app: &AppHandle,
    index: &SearchIndex,
    query_vector: Vec<f32>,
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    let tbl = table(app, index).await?;
    let batches: Vec<RecordBatch> = tbl
        .query()
        .nearest_to(query_vector.as_slice())
        .map_err(|e| format!("invalid search query: {e}"))?
        .limit(limit)
        .execute()
        .await
        .map_err(|e| format!("search failed: {e}"))?
        .try_collect()
        .await
        .map_err(|e| format!("search failed: {e}"))?;

    let mut hits = Vec::new();
    for batch in &batches {
        let column = |name: &str| -> Result<&StringArray, String> {
            batch
                .column_by_name(name)
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| format!("search index result is missing '{name}'"))
        };
        let message_ids = column("message_id")?;
        let conversation_ids = column("conversation_id")?;
        let roles = column("role")?;
        let contents = column("content")?;
        let distances = batch
            .column_by_name("_distance")
            .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

        for i in 0..batch.num_rows() {
            hits.push(SearchHit {
                message_id: message_ids.value(i).to_string(),
                conversation_id: conversation_ids.value(i).to_string(),
                role: roles.value(i).to_string(),
                content: contents.value(i).to_string(),
                distance: distances.map(|d| d.value(i)).unwrap_or(0.0),
            });
        }
    }
    Ok(hits)
}
