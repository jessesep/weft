//! ONE Bridge Usage Node - Query cc-bridge cost/usage data

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node, one_bridge_http};

#[derive(Default)]
pub struct OneBridgeUsageNode;

#[async_trait]
impl Node for OneBridgeUsageNode {
    fn node_type(&self) -> &'static str {
        "OneBridgeUsage"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "ONE Bridge Usage",
            inputs: vec![
                PortDef::new("agent", "String", false),
                PortDef::new("days", "Number", false),
            ],
            outputs: vec![
                PortDef::new("total_cost", "Number", false),
                PortDef::new("by_day", "List[JsonDict]", false),
                PortDef::new("by_agent", "List[JsonDict]", false),
            ],
            features: NodeFeatures { ..Default::default() },
            fields: vec![
                FieldDef::text("bridge_url").with_default(serde_json::json!("http://localhost:8070")),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let bridge_url = ctx.config.get("bridge_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:8070");

        let days = ctx.input.get("days")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let agent = ctx.input.get("agent")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

    let days_str = days.to_string();
    let mut params: Vec<(&str, &str)> = vec![("days", &days_str)];
    if !agent.is_empty() {
        params.push(("agent", &agent));
    }

        tracing::info!("OneBridgeUsage: GET /api/usage days={} agent={:?}", days, agent);

        match one_bridge_http::bridge_get(&ctx.http_client, bridge_url, "/api/usage", &params).await {
            Ok(resp) => {
                let total_cost = resp.get("total_cost")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let by_day = resp.get("by_day").cloned().unwrap_or(serde_json::json!([]));
                let by_agent = resp.get("by_agent").cloned().unwrap_or(serde_json::json!([]));

                NodeResult::completed(serde_json::json!({
                    "total_cost": total_cost,
                    "by_day": by_day,
                    "by_agent": by_agent,
                }))
            }
            Err(e) => {
                tracing::error!("OneBridgeUsage: {}", e);
                NodeResult::failed(&e)
            }
        }
    }
}

register_node!(OneBridgeUsageNode);
