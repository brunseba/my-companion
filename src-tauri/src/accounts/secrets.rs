use keyring::Entry;

/// Keychain "service" name accounts are stored under; each account's secret
/// payload is keyed by its own id within this service.
const SERVICE: &str = "com.brun_s.my-companion.accounts";

pub fn set(id: &str, value: &serde_json::Value) -> Result<(), String> {
    let entry = Entry::new(SERVICE, id).map_err(|e| e.to_string())?;
    entry
        .set_password(&value.to_string())
        .map_err(|e| e.to_string())
}

pub fn delete(id: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, id).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Never sent back to the frontend - only read internally, by validation
/// (Phase 2) and session refresh (Phase 3) logic.
pub fn get(id: &str) -> Result<Option<serde_json::Value>, String> {
    let entry = Entry::new(SERVICE, id).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(password) => serde_json::from_str(&password).map(Some).map_err(|e| e.to_string()),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Shallow-merges `patch`'s top-level keys into the existing secret payload
/// (creating it if absent) and writes the result back. Used by the OIDC login
//// refresh flow to add or update a `session` sub-object without disturbing
/// unrelated fields like `client_secret`.
pub fn merge(id: &str, patch: &serde_json::Value) -> Result<(), String> {
    let mut current = get(id)?.unwrap_or_else(|| serde_json::json!({}));
    let (Some(current_obj), Some(patch_obj)) = (current.as_object_mut(), patch.as_object()) else {
        return Err("secret payloads must be JSON objects".to_string());
    };
    for (key, value) in patch_obj {
        current_obj.insert(key.clone(), value.clone());
    }
    set(id, &current)
}
