use super::{optional_str, required_str};
use serde_json::Value;

pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let api_key = required_str(secret, "api_key")?;
    let base_url = optional_str(config, "base_url").unwrap_or("https://api.anthropic.com");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v1/models", base_url.trim_end_matches('/')))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Anthropic returned {}", response.status()))
    }
}
