//! Shared authorization-code + PKCE machinery, used by every provider that
//! supports interactive sign-in (generic OIDC, GitLab, GitHub, Jira,
//! Confluence). Each provider module supplies its own `Endpoints` (either via
//! `discover()`, for providers with a real OIDC discovery document, or
//! hardcoded, for providers with fixed endpoints like GitHub and Atlassian)
//! and calls `login`/`refresh` here.
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use url::Url;

pub struct Endpoints {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    /// When present, `validate_session` uses this as a live "is the stored
    /// access token still accepted" check.
    pub userinfo_endpoint: Option<String>,
}

#[derive(Deserialize)]
struct Discovery {
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    userinfo_endpoint: Option<String>,
}

/// Fetches an OIDC discovery document. Only providers that actually publish
/// one (generic OIDC, GitLab) call this - GitHub and Atlassian have fixed,
/// hardcoded endpoints instead.
pub async fn discover(issuer: &str) -> Result<Endpoints, String> {
    let discovery_url = format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/'));
    let response = reqwest::get(&discovery_url)
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("discovery document returned {}", response.status()));
    }
    let discovery: Discovery = response
        .json()
        .await
        .map_err(|e| format!("invalid discovery document: {e}"))?;
    Ok(Endpoints {
        authorization_endpoint: discovery.authorization_endpoint,
        token_endpoint: discovery.token_endpoint,
        userinfo_endpoint: discovery.userinfo_endpoint,
    })
}

/// If a session is stored, confirms its access token is still accepted.
/// A no-op (`Ok`) when there's no session yet, or the provider has no
/// userinfo endpoint to check against (e.g. GitHub, Atlassian - they validate
/// their sessions differently, see their own modules).
pub async fn validate_session(endpoints: &Endpoints, secret: &Value) -> Result<(), String> {
    let access_token = secret
        .get("session")
        .and_then(|s| s.get("access_token"))
        .and_then(Value::as_str);
    let (Some(token), Some(userinfo_endpoint)) = (access_token, &endpoints.userinfo_endpoint) else {
        return Ok(());
    };
    let response = reqwest::Client::new()
        .get(userinfo_endpoint)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("userinfo request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "session token rejected by userinfo endpoint ({}) - sign in again",
            response.status()
        ))
    }
}

struct Pkce {
    verifier: String,
    challenge: String,
}

fn generate_pkce() -> Pkce {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    Pkce { verifier, challenge }
}

fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

struct Callback {
    code: String,
}

/// Blocks the calling (blocking-pool) thread until the browser redirects back
/// with our authorization code, or five minutes pass. Runs on a loopback port
/// the OS assigns us, so it never collides with anything else on the machine.
fn await_callback(port_tx: std::sync::mpsc::Sender<u16>, expected_state: String) -> Result<Callback, String> {
    let server = tiny_http::Server::http("127.0.0.1:0").map_err(|e| format!("failed to start local listener: {e}"))?;
    let port = server
        .server_addr()
        .to_ip()
        .ok_or("failed to determine local listener port")?
        .port();
    port_tx
        .send(port)
        .map_err(|_| "failed to hand off local listener port".to_string())?;

    let deadline = std::time::Instant::now() + Duration::from_secs(300);
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return Err("timed out waiting for the browser sign-in to complete".to_string());
        }
        let request = match server.recv_timeout(remaining) {
            Ok(Some(request)) => request,
            Ok(None) => continue,
            Err(e) => return Err(format!("local listener error: {e}")),
        };

        let full_url = format!("http://127.0.0.1{}", request.url());
        let params: std::collections::HashMap<_, _> = match Url::parse(&full_url) {
            Ok(url) => url.query_pairs().into_owned().collect(),
            Err(_) => Default::default(),
        };

        let body = "<html><body>Signed in - you can close this window and return to my-companion.</body></html>";
        let response = tiny_http::Response::from_string(body)
            .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
        let _ = request.respond(response);

        // Ignore stray requests (favicon, etc.) that aren't our callback.
        if !params.contains_key("code") && !params.contains_key("error") {
            continue;
        }
        if params.get("state").map(String::as_str) != Some(expected_state.as_str()) {
            return Err("state mismatch on callback - possible CSRF, aborting sign-in".to_string());
        }
        if let Some(error) = params.get("error") {
            return Err(format!("provider returned an error: {error}"));
        }
        if let Some(code) = params.get("code") {
            return Ok(Callback { code: code.clone() });
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
}

fn session_patch(token: TokenResponse) -> Value {
    let expires_at = token
        .expires_in
        .map(|secs| (chrono::Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339());
    serde_json::json!({
        "session": {
            "access_token": token.access_token,
            "refresh_token": token.refresh_token,
            "id_token": token.id_token,
            "token_type": token.token_type,
            "expires_at": expires_at,
        }
    })
}

async fn exchange_code(
    token_endpoint: &str,
    client_id: &str,
    client_secret: Option<&str>,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<Value, String> {
    let mut form = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }

    let response = reqwest::Client::new()
        .post(token_endpoint)
        // GitHub's token endpoint returns form-encoded unless explicitly asked
        // for JSON; harmless (and correct either way) for standard OIDC/OAuth
        // token endpoints, which always return JSON regardless of this header.
        .header("Accept", "application/json")
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("token endpoint returned {}", response.status()));
    }
    let token: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("invalid token response: {e}"))?;
    Ok(session_patch(token))
}

/// Runs the full authorization-code + PKCE flow: opens the system browser,
/// waits for the redirect on a local loopback listener, and exchanges the
/// code for tokens. Returns a secret-store patch (`{"session": {...}}`) ready
/// to hand to `secrets::merge`. `extra_auth_params` lets a provider add
/// query params to the authorization URL beyond the standard OAuth2 ones
/// (e.g. Atlassian's `audience`/`prompt`).
pub async fn login(
    app: &AppHandle,
    endpoints: &Endpoints,
    client_id: &str,
    client_secret: Option<&str>,
    scopes: &str,
    extra_auth_params: &[(&str, &str)],
) -> Result<Value, String> {
    let pkce = generate_pkce();
    let state = generate_state();

    let (port_tx, port_rx) = std::sync::mpsc::channel::<u16>();
    let expected_state = state.clone();
    let listener = tokio::task::spawn_blocking(move || await_callback(port_tx, expected_state));

    let port = tokio::task::spawn_blocking(move || port_rx.recv())
        .await
        .map_err(|e| format!("internal error waiting for the local listener: {e}"))?
        .map_err(|_| "local listener failed to start".to_string())?;
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let mut auth_url =
        Url::parse(&endpoints.authorization_endpoint).map_err(|e| format!("invalid authorization endpoint: {e}"))?;
    {
        let mut query = auth_url.query_pairs_mut();
        query
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("scope", scopes)
            .append_pair("state", &state)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256");
        for (key, value) in extra_auth_params {
            query.append_pair(key, value);
        }
    }

    app.opener()
        .open_url(auth_url.to_string(), None::<&str>)
        .map_err(|e| format!("failed to open system browser: {e}"))?;

    let callback = listener
        .await
        .map_err(|e| format!("internal error during sign-in: {e}"))??;

    exchange_code(
        &endpoints.token_endpoint,
        client_id,
        client_secret,
        &callback.code,
        &redirect_uri,
        &pkce.verifier,
    )
    .await
}

/// Uses a previously stored refresh token to get a fresh access token,
/// without involving the browser. Fails with a clear "sign in again" message
/// if there's no refresh token or the provider rejects it.
pub async fn refresh(
    endpoints: &Endpoints,
    client_id: &str,
    client_secret: Option<&str>,
    existing_secret: &Value,
) -> Result<Value, String> {
    let refresh_token = existing_secret
        .get("session")
        .and_then(|s| s.get("refresh_token"))
        .and_then(Value::as_str)
        .ok_or("no refresh token stored - sign in again")?
        .to_string();

    let mut form = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
        ("client_id", client_id),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }

    let response = reqwest::Client::new()
        .post(&endpoints.token_endpoint)
        .header("Accept", "application/json")
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("token refresh failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("token endpoint returned {} - sign in again", response.status()));
    }
    let mut token: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("invalid token response: {e}"))?;
    // Providers commonly omit refresh_token on a refresh response, meaning "keep using the old one".
    if token.refresh_token.is_none() {
        token.refresh_token = Some(refresh_token);
    }
    Ok(session_patch(token))
}
