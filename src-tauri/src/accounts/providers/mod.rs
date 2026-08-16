mod anthropic;
mod atlassian;
mod aws;
mod azure;
mod confluence;
mod gcp;
mod github;
mod gitlab;
mod jira;
mod kubeconfig;
mod oauth;
mod oidc;
mod openai;
mod scaleway;

use serde_json::Value;
use tauri::AppHandle;

/// Providers with an interactive sign-in flow. OIDC is oauth-only; the other
/// four support either a stored token or OAuth, chosen per-account via
/// `config.auth_method` (see `is_oauth_account`).
const OAUTH_CAPABLE_PROVIDERS: &[&str] = &["oidc", "gitlab", "github", "jira", "confluence"];

pub fn is_oauth_capable(provider: &str) -> bool {
    OAUTH_CAPABLE_PROVIDERS.contains(&provider)
}

/// Whether this specific account is configured to use OAuth sign-in - true
/// for every OIDC account (it has no other mode), and for GitLab/GitHub/Jira/
/// Confluence accounts whose `config.auth_method` is explicitly `"oauth"`.
pub fn is_oauth_account(provider: &str, config: &Value) -> bool {
    provider == "oidc" || (is_oauth_capable(provider) && optional_str(config, "auth_method") == Some("oauth"))
}

/// Runs the authorization-code + PKCE browser login flow for whichever
/// OAuth-capable provider this account is. Returns a secret-store patch
/// ready for `secrets::merge`.
pub async fn oauth_login(app: &AppHandle, provider: &str, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    match provider {
        "oidc" => oidc::login(app, config, existing_secret).await,
        "gitlab" => gitlab::login(app, config, existing_secret).await,
        "github" => github::login(app, config, existing_secret).await,
        "jira" => jira::login(app, config, existing_secret).await,
        "confluence" => confluence::login(app, config, existing_secret).await,
        other => Err(format!("'{other}' accounts don't support interactive sign-in")),
    }
}

/// Refreshes a stored OAuth session's access token using its refresh token.
pub async fn oauth_refresh(provider: &str, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    match provider {
        "oidc" => oidc::refresh(config, existing_secret).await,
        "gitlab" => gitlab::refresh(config, existing_secret).await,
        "github" => github::refresh(config, existing_secret).await,
        "jira" => jira::refresh(config, existing_secret).await,
        "confluence" => confluence::refresh(config, existing_secret).await,
        other => Err(format!("'{other}' accounts don't have sessions to refresh")),
    }
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
        "github" => github::validate(config, secret).await,
        "gitlab" => gitlab::validate(config, secret).await,
        "jira" => jira::validate(config, secret).await,
        "confluence" => confluence::validate(config, secret).await,
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
