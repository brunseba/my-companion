use super::model::{Account, AccountStatus, CreateAccountInput, UpdateAccountInput};
use super::providers;
use super::secrets;
use super::store::{self, AccountsState};
use chrono::Utc;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command]
pub fn list_accounts(state: State<AccountsState>) -> Vec<Account> {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn create_account(
    app: AppHandle,
    state: State<AccountsState>,
    input: CreateAccountInput,
) -> Result<Account, String> {
    let now = Utc::now().to_rfc3339();
    let account = Account {
        id: Uuid::new_v4().to_string(),
        category: input.category,
        provider: input.provider,
        name: input.name,
        config: input.config,
        status: AccountStatus::Unknown,
        created_at: now.clone(),
        updated_at: now,
        last_validated_at: None,
        last_error: None,
        session_expires_at: None,
    };

    if let Some(secret) = &input.secret {
        secrets::set(&account.id, secret)?;
    }

    let mut accounts = state.0.lock().unwrap();
    accounts.push(account.clone());
    store::save(&app, &accounts)?;
    Ok(account)
}

#[tauri::command]
pub fn update_account(
    app: AppHandle,
    state: State<AccountsState>,
    id: String,
    input: UpdateAccountInput,
) -> Result<Account, String> {
    let mut accounts = state.0.lock().unwrap();
    let account = accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("no account with id {id}"))?;

    if let Some(name) = input.name {
        account.name = name;
    }
    if let Some(config) = input.config {
        account.config = config;
    }
    account.updated_at = Utc::now().to_rfc3339();
    // Any config/secret change invalidates the last known status until re-validated.
    account.status = AccountStatus::Unknown;
    account.last_validated_at = None;
    account.last_error = None;
    account.session_expires_at = None;

    if let Some(secret) = &input.secret {
        secrets::set(&id, secret)?;
    }

    let updated = account.clone();
    store::save(&app, &accounts)?;
    Ok(updated)
}

#[tauri::command]
pub async fn test_account(app: AppHandle, state: State<'_, AccountsState>, id: String) -> Result<Account, String> {
    // Snapshot what we need before the `.await` - the mutex guard can't cross it.
    let (provider, config) = {
        let accounts = state.0.lock().unwrap();
        let account = accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("no account with id {id}"))?;
        (account.provider.clone(), account.config.clone())
    };
    let secret = secrets::get(&id)?.unwrap_or_else(|| serde_json::json!({}));

    let result = providers::validate(&provider, &config, &secret).await;

    let mut accounts = state.0.lock().unwrap();
    let account = accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("no account with id {id}"))?;
    account.last_validated_at = Some(Utc::now().to_rfc3339());
    match result {
        Ok(()) => {
            account.status = AccountStatus::Valid;
            account.last_error = None;
        }
        Err(e) => {
            account.status = AccountStatus::Error;
            account.last_error = Some(e);
        }
    }
    let updated = account.clone();
    store::save(&app, &accounts)?;
    Ok(updated)
}

#[tauri::command]
pub async fn oauth_login(app: AppHandle, state: State<'_, AccountsState>, id: String) -> Result<Account, String> {
    let (provider, config) = oauth_account(&state, &id)?;
    let existing_secret = secrets::get(&id)?.unwrap_or_else(|| serde_json::json!({}));
    let result = providers::oauth_login(&app, &provider, &config, &existing_secret).await;
    apply_session_result(&app, &state, &id, result)
}

#[tauri::command]
pub async fn refresh_oauth_session(
    app: AppHandle,
    state: State<'_, AccountsState>,
    id: String,
) -> Result<Account, String> {
    let (provider, config) = oauth_account(&state, &id)?;
    let existing_secret = secrets::get(&id)?.unwrap_or_else(|| serde_json::json!({}));
    let result = providers::oauth_refresh(&provider, &config, &existing_secret).await;
    apply_session_result(&app, &state, &id, result)
}

/// Looks up an account's provider + config, checking it's actually set up for
/// OAuth sign-in - a GitHub/GitLab/Jira/Confluence account using a plain
/// token instead (or any of the machine-to-machine providers like AWS) has no
/// interactive session to sign into or refresh.
fn oauth_account(state: &AccountsState, id: &str) -> Result<(String, serde_json::Value), String> {
    let accounts = state.0.lock().unwrap();
    let account = accounts
        .iter()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("no account with id {id}"))?;
    if !providers::is_oauth_account(&account.provider, &account.config) {
        return Err(format!(
            "'{}' accounts don't support interactive sign-in with their current auth method",
            account.provider
        ));
    }
    Ok((account.provider.clone(), account.config.clone()))
}

/// Shared tail end of the login/refresh commands: persist the new session (or
/// record the failure) and return the updated account.
fn apply_session_result(
    app: &AppHandle,
    state: &AccountsState,
    id: &str,
    result: Result<serde_json::Value, String>,
) -> Result<Account, String> {
    let mut accounts = state.0.lock().unwrap();
    let account = accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("no account with id {id}"))?;
    account.last_validated_at = Some(Utc::now().to_rfc3339());

    match result {
        Ok(patch) => {
            secrets::merge(id, &patch)?;
            account.session_expires_at = patch
                .get("session")
                .and_then(|s| s.get("expires_at"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned);
            account.status = AccountStatus::Valid;
            account.last_error = None;
        }
        Err(e) => {
            account.status = AccountStatus::Error;
            account.last_error = Some(e);
        }
    }

    let updated = account.clone();
    store::save(app, &accounts)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_account(app: AppHandle, state: State<AccountsState>, id: String) -> Result<(), String> {
    let mut accounts = state.0.lock().unwrap();
    let before = accounts.len();
    accounts.retain(|a| a.id != id);
    if accounts.len() == before {
        return Err(format!("no account with id {id}"));
    }
    secrets::delete(&id)?;
    store::save(&app, &accounts)
}
