//! Streams a chat completion from OpenAI or Anthropic, emitting each text
//! delta as a `chat:delta` event so the frontend can render tokens as they
//! arrive, and returning the fully assembled reply once the stream ends.
use super::model::ChatMessage;
use futures_util::StreamExt;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

pub async fn stream_reply(
    app: &AppHandle,
    conversation_id: &str,
    provider: &str,
    base_url: Option<&str>,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    match provider {
        "openai" => stream_openai(app, conversation_id, base_url, api_key, model, messages).await,
        "anthropic" => stream_anthropic(app, conversation_id, base_url, api_key, model, messages).await,
        other => Err(format!("chat isn't supported for the '{other}' provider")),
    }
}

fn emit_delta(app: &AppHandle, conversation_id: &str, text: &str) {
    // Best-effort: a delta the frontend misses just means a less smooth
    // stream, not a broken one - the full reply is still returned at the end.
    let _ = app.emit(
        "chat:delta",
        serde_json::json!({ "conversation_id": conversation_id, "text": text }),
    );
}

fn messages_json(messages: &[ChatMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect()
}

/// Splits a growing SSE buffer into complete `\n\n`-delimited events, calling
/// `on_data` with each event's `data: ...` line content (skipping the
/// `[DONE]` sentinel). Shared between providers - only the payload shape
/// inside each event differs.
async fn consume_sse<F: FnMut(&str)>(
    response: reqwest::Response,
    mut on_data: F,
) -> Result<(), String> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("stream error: {e}"))?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        while let Some(pos) = buffer.find("\n\n") {
            let event: String = buffer.drain(..pos + 2).collect();
            for line in event.lines() {
                let Some(data) = line.strip_prefix("data: ") else { continue };
                if data == "[DONE]" {
                    continue;
                }
                on_data(data);
            }
        }
    }
    Ok(())
}

async fn stream_openai(
    app: &AppHandle,
    conversation_id: &str,
    base_url: Option<&str>,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let url = format!(
        "{}/chat/completions",
        base_url.unwrap_or("https://api.openai.com/v1").trim_end_matches('/')
    );
    let body = serde_json::json!({
        "model": model,
        "stream": true,
        "messages": messages_json(messages),
    });

    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("OpenAI returned {}", response.status()));
    }

    let mut full_text = String::new();
    consume_sse(response, |data| {
        let Ok(value) = serde_json::from_str::<Value>(data) else { return };
        if let Some(delta) = value["choices"][0]["delta"]["content"].as_str() {
            full_text.push_str(delta);
            emit_delta(app, conversation_id, delta);
        }
    })
    .await?;

    Ok(full_text)
}

async fn stream_anthropic(
    app: &AppHandle,
    conversation_id: &str,
    base_url: Option<&str>,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let url = format!(
        "{}/v1/messages",
        base_url.unwrap_or("https://api.anthropic.com").trim_end_matches('/')
    );
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "stream": true,
        "messages": messages_json(messages),
    });

    let response = reqwest::Client::new()
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("Anthropic returned {}", response.status()));
    }

    let mut full_text = String::new();
    consume_sse(response, |data| {
        let Ok(value) = serde_json::from_str::<Value>(data) else { return };
        if value.get("type").and_then(Value::as_str) != Some("content_block_delta") {
            return;
        }
        if let Some(delta) = value["delta"]["text"].as_str() {
            full_text.push_str(delta);
            emit_delta(app, conversation_id, delta);
        }
    })
    .await?;

    Ok(full_text)
}
