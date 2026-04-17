//! ONE Telethon Admin Node - Generic proxy to telethon-service user API
//!
//! Base node for all flows needing Telegram user-API access beyond the Bot API.
//! Requires telethon-service running at :13100 (see weft-one/modules/telegram-admin/).

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node};

const TELETHON_TIMEOUT_SECS: u64 = 30;

#[derive(Default)]
pub struct OneTelethonAdminNode;

#[async_trait]
impl Node for OneTelethonAdminNode {
    fn node_type(&self) -> &'static str {
        "OneTelethonAdmin"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "ONE Telethon Admin",
            inputs: vec![
                PortDef::new("operation", "String", true),
                PortDef::new("payload", "JsonDict", false),
            ],
            outputs: vec![
                PortDef::new("result", "JsonDict", false),
                PortDef::new("ok", "Boolean", false),
            ],
            features: NodeFeatures { ..Default::default() },
            fields: vec![
                FieldDef::text("telethon_url").with_default(serde_json::json!("http://127.0.0.1:13100")),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let telethon_url = ctx.config.get("telethon_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://127.0.0.1:13100");

        let operation = ctx.input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if operation.is_empty() {
            return NodeResult::failed("operation is required");
        }

        let payload = ctx.input.get("payload")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let url = format!("{}/tg/{}", telethon_url, operation);

        tracing::info!("OneTelethonAdmin: POST /tg/{}", operation);

        let result = ctx.http_client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(TELETHON_TIMEOUT_SECS))
            .send()
            .await;

        match result {
            Ok(response) => {
                let status = response.status();
                match response.json::<serde_json::Value>().await {
                    Ok(resp_json) => {
                        let ok = status.is_success()
                            && resp_json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

                        tracing::info!("OneTelethonAdmin: op={} ok={}", operation, ok);

                        NodeResult::completed(serde_json::json!({
                            "result": resp_json,
                            "ok": ok,
                        }))
                    }
                    Err(e) => {
                        tracing::error!("OneTelethonAdmin: parse error: {}", e);
                        NodeResult::failed(&format!("telethon-service response parse failed: {}", e))
                    }
                }
            }
            Err(e) => {
                tracing::error!("OneTelethonAdmin: request failed: {}", e);
                NodeResult::failed(&format!("telethon-service request failed: {}", e))
            }
        }
    }
}

register_node!(OneTelethonAdminNode);
