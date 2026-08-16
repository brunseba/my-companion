use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountCategory {
    Ai,
    Csp,
    K8s,
    Oidc,
    Scm,
    Tracker,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Unknown,
    Valid,
    Expired,
    Error,
}

impl Default for AccountStatus {
    fn default() -> Self {
        AccountStatus::Unknown
    }
}

/// A stored account. Never carries secret values - those live in the OS keychain,
/// keyed by `id`, and are only ever read or written from Rust (see `secrets.rs`).
/// `config` holds provider-specific, non-secret settings (region, issuer URL,
/// kubeconfig path, ...); the frontend decides its shape per provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub category: AccountCategory,
    pub provider: String,
    pub name: String,
    pub config: serde_json::Value,
    pub status: AccountStatus,
    pub created_at: String,
    pub updated_at: String,
    pub last_validated_at: Option<String>,
    /// Set when `status` is `Error`, cleared otherwise. Safe to show in the UI -
    /// provider validators are careful not to echo secrets into error text.
    pub last_error: Option<String>,
    /// RFC3339 expiry of the current session's access token, if this account
    /// has an active interactive login (currently: OIDC only). `None` means
    /// there's no session - either it was never signed in, or it's a
    /// non-interactive provider.
    pub session_expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub category: AccountCategory,
    pub provider: String,
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
    /// Secret fields (API keys, client secrets, ...). Stored in the keychain, never persisted to disk.
    pub secret: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountInput {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    /// If present, replaces the stored secret. Omit to leave the existing secret untouched.
    pub secret: Option<serde_json::Value>,
}
