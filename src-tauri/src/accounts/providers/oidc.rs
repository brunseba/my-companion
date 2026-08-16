use super::oauth::{self, Endpoints};
use super::{optional_str, required_str};
use serde_json::Value;
use tauri::AppHandle;

async fn endpoints(issuer: &str) -> Result<Endpoints, String> {
    oauth::discover(issuer).await
}

/// Reachability check, plus - if a session is already stored - a live check
/// that its access token is still accepted by the provider.
pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let issuer = required_str(config, "issuer_url")?;
    let endpoints = endpoints(issuer).await?;
    oauth::validate_session(&endpoints, secret).await
}

pub async fn login(app: &AppHandle, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let issuer = required_str(config, "issuer_url")?;
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    let scopes = optional_str(config, "scopes").unwrap_or("openid profile email");
    let endpoints = endpoints(issuer).await?;
    oauth::login(app, &endpoints, client_id, client_secret, scopes, &[]).await
}

pub async fn refresh(config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let issuer = required_str(config, "issuer_url")?;
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    let endpoints = endpoints(issuer).await?;
    oauth::refresh(&endpoints, client_id, client_secret, existing_secret).await
}
