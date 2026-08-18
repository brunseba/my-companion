//! Local, offline text embeddings via `fastembed` (BGESmallENV15, 384 dims -
//! fastembed's own docs describe it as "the fast and default English model").
//! No network call, no API key, no account needed - the ONNX model downloads
//! once (cached under the app data dir) and every embedding after that runs
//! entirely on-device.
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub const EMBEDDING_DIM: i32 = 384;

/// Lazily-loaded model, kept across calls so repeated embedding doesn't
/// reload (or re-download) it each time. Both loading and inference are
/// CPU-bound and synchronous - callers must run `embed_blocking` inside
/// `tokio::task::spawn_blocking`, never directly on the async runtime.
pub struct Embedder(Mutex<Option<TextEmbedding>>);

impl Embedder {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

fn load_model(app: &AppHandle) -> Result<TextEmbedding, String> {
    let cache_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?
        .join("models");
    let options = TextInitOptions::new(EmbeddingModel::BGESmallENV15).with_cache_dir(cache_dir);
    TextEmbedding::try_new(options).map_err(|e| format!("failed to load embedding model: {e}"))
}

/// Embeds one piece of text, loading the model first if this is the first
/// call in this app session. Blocking - always call from within
/// `spawn_blocking`.
pub fn embed_blocking(app: &AppHandle, embedder: &Embedder, text: &str) -> Result<Vec<f32>, String> {
    let mut guard = embedder.0.lock().unwrap();
    if guard.is_none() {
        *guard = Some(load_model(app)?);
    }
    let model = guard.as_mut().unwrap();
    let mut embeddings = model
        .embed(vec![text], None)
        .map_err(|e| format!("failed to embed text: {e}"))?;
    embeddings
        .pop()
        .ok_or_else(|| "embedding model returned no output".to_string())
}
