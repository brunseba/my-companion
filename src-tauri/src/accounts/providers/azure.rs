use super::required_str;
use serde_json::Value;

/// Confirms the service principal can actually acquire a token (client
/// credentials flow) - doesn't call any Azure resource API beyond that.
pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let tenant_id = required_str(config, "tenant_id")?;
    let client_id = required_str(secret, "client_id")?;
    let client_secret = required_str(secret, "client_secret")?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token"))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("scope", "https://management.azure.com/.default"),
        ])
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Azure token request returned {}", response.status()))
    }
}
