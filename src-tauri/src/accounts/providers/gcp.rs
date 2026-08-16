use super::required_str;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: i64,
    exp: i64,
}

/// Signs a short-lived JWT with the service account's private key and
/// exchanges it for an access token - proves the key is valid and active
/// without needing to call any GCP resource API.
pub async fn validate(_config: &Value, secret: &Value) -> Result<(), String> {
    let raw = required_str(secret, "service_account_json")?;
    let key: Value = serde_json::from_str(raw).map_err(|e| format!("invalid service account JSON: {e}"))?;
    let client_email = required_str(&key, "client_email")?;
    let private_key = required_str(&key, "private_key")?;

    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        iss: client_email,
        scope: "https://www.googleapis.com/auth/cloud-platform.read-only",
        aud: "https://oauth2.googleapis.com/token",
        iat: now,
        exp: now + 3600,
    };

    let encoding_key =
        EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|e| format!("invalid private key: {e}"))?;
    let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
        .map_err(|e| format!("failed to sign JWT: {e}"))?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", jwt.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Google token exchange returned {}", response.status()))
    }
}
