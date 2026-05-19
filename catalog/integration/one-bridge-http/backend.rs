//! ONE Bridge HTTP - Shared HTTP helpers for all nodes that call cc-bridge.
//!
//! Not a runnable node. Exported as `crate::one_bridge_http` and used by:
//! one-bridge-llm, one-bridge-usage, bridge-rag-query, one-telethon-admin.
//!
//! Centralizes: base URL + path building, 120s timeout, error body extraction,
//! and JSON parse error normalization.

use reqwest::Client;
use serde_json::Value;

const BRIDGE_TIMEOUT_SECS: u64 = 120;

/// GET `{base_url}{path}?{params}` - returns parsed JSON or a descriptive error string.
pub async fn bridge_get(
    client: &Client,
    base_url: &str,
    path: &str,
    params: &[(&str, &str)],
) -> Result<Value, String> {
    let url = format!("{}{}", base_url, path);
    let result = client
        .get(&url)
        .query(params)
        .timeout(std::time::Duration::from_secs(BRIDGE_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("Bridge GET {} failed: {}", path, e))?;

    let status = result.status();
    if !status.is_success() {
        let body = result.text().await.unwrap_or_default();
        return Err(format!("Bridge GET {} returned HTTP {}: {}", path, status, body));
    }

    result
        .json::<Value>()
        .await
        .map_err(|e| format!("Bridge GET {} response parse failed: {}", path, e))
}

/// POST JSON `body` to `{base_url}{path}` - returns parsed JSON or a descriptive error string.
pub async fn bridge_post(
    client: &Client,
    base_url: &str,
    path: &str,
    body: Value,
) -> Result<Value, String> {
    let url = format!("{}{}", base_url, path);
    let result = client
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(BRIDGE_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("Bridge POST {} failed: {}", path, e))?;

    let status = result.status();
    if !status.is_success() {
        let body_text = result.text().await.unwrap_or_default();
        return Err(format!("Bridge POST {} returned HTTP {}: {}", path, status, body_text));
    }

    result
        .json::<Value>()
        .await
        .map_err(|e| format!("Bridge POST {} response parse failed: {}", path, e))
}
