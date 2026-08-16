use super::{optional_str, required_str};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use serde_json::Value;

pub async fn validate(config: &Value, secret: &Value) -> Result<(), String> {
    let access_key_id = required_str(secret, "access_key_id")?;
    let secret_access_key = required_str(secret, "secret_access_key")?;
    let session_token = optional_str(secret, "session_token").map(str::to_owned);
    let region = optional_str(config, "region").unwrap_or("us-east-1");

    let credentials = Credentials::new(
        access_key_id,
        secret_access_key,
        session_token,
        None,
        "my-companion",
    );
    let sdk_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region.to_string()))
        .credentials_provider(credentials)
        .load()
        .await;

    aws_sdk_sts::Client::new(&sdk_config)
        .get_caller_identity()
        .send()
        .await
        .map_err(|e| format!("AWS STS GetCallerIdentity failed: {e}"))?;

    Ok(())
}
