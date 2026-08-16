mod anthropic;
mod aws;
mod azure;
mod gcp;
mod kubeconfig;
mod oidc;
mod openai;
mod scaleway;

use serde_json::Value;
use tauri::AppHandle;

/// Runs the OIDC authorization-code + PKCE browser login flow. Returns a
/// secret-store patch ready for `secrets::merge`. Only OIDC has this - the
/// other providers are all machine-to-machine (no interactive login needed).
pub async fn oidc_login(app: &AppHandle, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    oidc::login(app, config, existing_secret).await
}

/// Refreshes an OIDC session's access token using its stored refresh token.
pub async fn oidc_refresh(config: &Value, existing_secret: &Value) -> Result<Value, String> {
    oidc::refresh(config, existing_secret).await
}

/// Runs a read-only "does this account actually work" check against the
/// provider's API. `secret` is the keychain payload for this account (may be
/// an empty object if none was ever stored).
pub async fn validate(provider: &str, config: &Value, secret: &Value) -> Result<(), String> {
    match provider {
        "openai" => openai::validate(config, secret).await,
        "anthropic" => anthropic::validate(config, secret).await,
        "aws" => aws::validate(config, secret).await,
        "azure" => azure::validate(config, secret).await,
        "gcp" => gcp::validate(config, secret).await,
        "scaleway" => scaleway::validate(config, secret).await,
        "kubeconfig" => kubeconfig::validate(config, secret).await,
        "oidc" => oidc::validate(config, secret).await,
        other => Err(format!("no validator registered for provider '{other}'")),
    }
}

/// Shared helper: pull a required non-empty string field out of a JSON object.
pub(super) fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing '{key}'"))
}

/// Same, but returns `None` instead of erroring when absent - for optional fields.
pub(super) fn optional_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}
