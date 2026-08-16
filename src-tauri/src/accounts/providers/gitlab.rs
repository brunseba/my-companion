use super::oauth;
use super::{optional_str, required_str};
use serde_json::Value;
use tauri::AppHandle;

// GitLab is a genuine OIDC provider (self-managed instances too), so unlike
// GitHub/Atlassian this module discovers its endpoints rather than
// hardcoding them.
const DEFAULT_SCOPES: &str = "read_api read_user";

fn base_url(config: &Value) -> String {
    optional_str(config, "base_url")
        .unwrap_or("https://gitlab.com")
        .trim_end_matches('/')
        .to_string()
}

async fn whoami_with_token(base: &str, token: &str) -> Result<(), String> {
    let response = reqwest::Client::new()
        .get(format!("{base}/api/v4/user"))
        .header("PRIVATE-TOKEN", token)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("GitLab returned {}", response.status()))
    }
}

pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let base = base_url(config);
    match optional_str(config, "auth_method").unwrap_or("token") {
        "oauth" => {
            let endpoints = oauth::discover(&base).await?;
            oauth::validate_session(&endpoints, secret).await
        }
        _ => {
            let token = required_str(secret, "token")?;
            whoami_with_token(&base, token).await
        }
    }
}

pub async fn login(app: &AppHandle, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let base = base_url(config);
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    let endpoints = oauth::discover(&base).await?;
    oauth::login(app, &endpoints, client_id, client_secret, DEFAULT_SCOPES, &[]).await
}

pub async fn refresh(config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let base = base_url(config);
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    let endpoints = oauth::discover(&base).await?;
    oauth::refresh(&endpoints, client_id, client_secret, existing_secret).await
}
