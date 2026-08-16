use super::atlassian;
use super::optional_str;
use serde_json::Value;
use tauri::AppHandle;

const SCOPES: &str = "read:confluence-user read:confluence-content.all offline_access";
const WHOAMI_PATH: &str = "/wiki/rest/api/user/current";

pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    match optional_str(config, "auth_method").unwrap_or("token") {
        "oauth" => atlassian::validate_session(secret).await,
        _ => atlassian::validate_token(config, secret, WHOAMI_PATH).await,
    }
}

pub async fn login(app: &AppHandle, config: &Value, existing_secret: &Value) -> Result<Value, String> {
    atlassian::login(app, config, existing_secret, SCOPES).await
}

pub async fn refresh(config: &Value, existing_secret: &Value) -> Result<Value, String> {
    atlassian::refresh(config, existing_secret).await
}
