use super::required_str;
use serde_json::Value;

pub async fn validate(_config: &Value, secret: &Value) -> Result<(), String> {
    let access_key = required_str(secret, "access_key")?;
    let secret_key = required_str(secret, "secret_key")?;

    // Fetching the key's own IAM record is a cheap way to confirm both halves
    // of the pair are valid and still active.
    let client = reqwest::Client::new();
    let response = client
        .get(format!("https://api.scaleway.com/iam/v1alpha1/api-keys/{access_key}"))
        .header("X-Auth-Token", secret_key)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Scaleway returned {}", response.status()))
    }
}
