use super::{optional_str, required_str};
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Api, ListParams};
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client, Config};
use serde_json::Value;
use std::path::PathBuf;

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

/// Lists namespaces (capped at 1 result) as a cheap "can we actually talk to
/// this cluster with this context" check.
pub async fn validate(config: &Value, _secret: &Value) -> Result<(), String> {
    let path = required_str(config, "path")?;
    let context = optional_str(config, "context").map(str::to_owned);

    let kubeconfig = Kubeconfig::read_from(expand_tilde(path))
        .map_err(|e| format!("failed to read kubeconfig at '{path}': {e}"))?;

    let options = KubeConfigOptions {
        context,
        ..Default::default()
    };
    let client_config = Config::from_custom_kubeconfig(kubeconfig, &options)
        .await
        .map_err(|e| format!("failed to build cluster config: {e}"))?;
    let client = Client::try_from(client_config).map_err(|e| format!("failed to build cluster client: {e}"))?;

    let namespaces: Api<Namespace> = Api::all(client);
    namespaces
        .list(&ListParams::default().limit(1))
        .await
        .map_err(|e| format!("failed to list namespaces: {e}"))?;
    Ok(())
}
