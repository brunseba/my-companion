use super::{optional_str, required_str};
use serde_json::Value;

pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let api_key = required_str(secret, "api_key")?;
    let base_url = optional_str(config, "base_url").unwrap_or("https://api.openai.com/v1");

    let client = reqwest::Client::new();
    let mut request = client
        .get(format!("{}/models", base_url.trim_end_matches('/')))
        .bearer_auth(api_key);
    if let Some(org) = optional_str(config, "organization") {
        request = request.header("OpenAI-Organization", org);
    }

    let response = request.send().await.map_err(|e| format!("request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("OpenAI returned {}", response.status()))
    }
}
