use super::oauth::{self, Endpoints};
use super::{optional_str, required_str};
use serde_json::Value;
use tauri::AppHandle;

const AUTHORIZATION_ENDPOINT: &str = "https://github.com/login/oauth/authorize";
const TOKEN_ENDPOINT: &str = "https://github.com/login/oauth/access_token";
const DEFAULT_SCOPES: &str = "read:user repo";

/// GitHub Enterprise Server exposes its REST API under `/api/v3` on the same
/// host; github.com uses a separate api.github.com host instead.
fn api_base(config: &Value) -> String {
    match optional_str(config, "base_url") {
        Some(base) => format!("{}/api/v3", base.trim_end_matches('/')),
        None => "https://api.github.com".to_string(),
    }
}

async fn whoami(api_base: &str, token: &str) -> Result<(), String> {
    let response = reqwest::Client::new()
        .get(format!("{api_base}/user"))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        // GitHub's API rejects requests with no User-Agent.
        .header("User-Agent", "my-companion")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("GitHub returned {}", response.status()))
    }
}

pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let api_base = api_base(config);
    let token = match optional_str(config, "auth_method").unwrap_or("token") {
        "oauth" => secret
            .get("session")
            .and_then(|s| s.get("access_token"))
            .and_then(Value::as_str)
            .ok_or("no session stored - sign in")?,
        _ => required_str(secret, "token")?,
    };
    whoami(&api_base, token).await
}

fn endpoints() -> Endpoints {
    Endpoints {
        authorization_endpoint: AUTHORIZATION_ENDPOINT.to_string(),
        token_endpoint: TOKEN_ENDPOINT.to_string(),
        // No userinfo endpoint in the OIDC sense - `validate` above checks the
        // session directly against the GitHub REST API instead.
        userinfo_endpoint: None,
    }
}

pub async fn login(app: &AppHandle, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    oauth::login(app, &endpoints(), client_id, client_secret, DEFAULT_SCOPES, &[]).await
}

pub async fn refresh(config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    oauth::refresh(&endpoints(), client_id, client_secret, existing_secret).await
}
