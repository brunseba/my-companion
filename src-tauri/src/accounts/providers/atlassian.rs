//! Shared logic for Jira and Confluence - both are Atlassian Cloud products
//! with the same OAuth 2.0 (3LO) app registration and the same Basic-auth API
//! token scheme, differing only in scopes and REST paths (see jira.rs /
//! confluence.rs).
use super::oauth::{self, Endpoints};
use super::{optional_str, required_str};
use serde_json::Value;
use tauri::AppHandle;

const AUTHORIZATION_ENDPOINT: &str = "https://auth.atlassian.com/authorize";
const TOKEN_ENDPOINT: &str = "https://auth.atlassian.com/oauth/token";
const ACCESSIBLE_RESOURCES_ENDPOINT: &str = "https://api.atlassian.com/oauth/token/accessible-resources";

fn endpoints() -> Endpoints {
    Endpoints {
        authorization_endpoint: AUTHORIZATION_ENDPOINT.to_string(),
        token_endpoint: TOKEN_ENDPOINT.to_string(),
        userinfo_endpoint: None,
    }
}

pub async fn login(app: &AppHandle, config: &Value, existing_secret: &Value, scopes: &str) -> Result<Value, String> {
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    // Atlassian's 3LO flow requires `audience` to say which API you're
    // targeting, and `prompt=consent` so the scope grant screen always shows
    // (otherwise a returning user can silently get a token with stale scopes).
    oauth::login(
        app,
        &endpoints(),
        client_id,
        client_secret,
        scopes,
        &[("audience", "api.atlassian.com"), ("prompt", "consent")],
    )
    .await
}

pub async fn refresh(config: &Value, existing_secret: &Value) -> Result<Value, String> {
    let client_id = required_str(config, "client_id")?;
    let client_secret = optional_str(existing_secret, "client_secret");
    oauth::refresh(&endpoints(), client_id, client_secret, existing_secret).await
}

/// Atlassian's 3LO flow has no plain userinfo endpoint - the standard "is
/// this token still good" check is asking which sites it can reach.
pub async fn validate_session(secret: &Value) -> Result<(), String> {
    let token = secret
        .get("session")
        .and_then(|s| s.get("access_token"))
        .and_then(Value::as_str)
        .ok_or("no session stored - sign in")?;
    let response = reqwest::Client::new()
        .get(ACCESSIBLE_RESOURCES_ENDPOINT)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("session token rejected ({}) - sign in again", response.status()))
    }
}

/// Email + API token (Atlassian Cloud's Basic auth scheme), checked against a
/// product-specific "who am I" endpoint.
pub async fn validate_token(config: &Value, secret: &Value, whoami_path: &str) -> Result<(), String> {
    let base_url = required_str(config, "base_url")?;
    let email = required_str(secret, "email")?;
    let api_token = required_str(secret, "api_token")?;
    let response = reqwest::Client::new()
        .get(format!("{}{}", base_url.trim_end_matches('/'), whoami_path))
        .basic_auth(email, Some(api_token))
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("{} returned {}", whoami_path, response.status()))
    }
}
