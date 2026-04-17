//! Bridge RAG Query Node - Query the ONE bridge RAG/CNS endpoint
//!
//! Requires /api/weft/rag on the bridge provisioner (see ARCHITECTURE.md).

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node, one_bridge_http};

#[derive(Default)]
pub struct BridgeRagQueryNode;

#[async_trait]
impl Node for BridgeRagQueryNode {
    fn node_type(&self) -> &'static str {
        "BridgeRagQuery"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "Bridge RAG Query",
            inputs: vec![
                PortDef::new("query", "String", true),
                PortDef::new("top_k", "Number", false),
            ],
            outputs: vec![
                PortDef::new("chunks", "List", false),
                PortDef::new("sources", "List", false),
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

        let query = ctx.input.get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if query.is_empty() {
            return NodeResult::failed("query is required");
        }

        let top_k = ctx.input.get("top_k")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        tracing::info!("BridgeRagQuery: query_len={} top_k={}", query.len(), top_k);

        let body = serde_json::json!({
            "query": query,
            "top_k": top_k,
        });

        match one_bridge_http::bridge_post(&ctx.http_client, bridge_url, "/api/weft/rag", body).await {
            Ok(resp) => {
                let chunks = resp.get("chunks").cloned().unwrap_or(serde_json::json!([]));
                let sources = resp.get("sources").cloned().unwrap_or(serde_json::json!([]));

                NodeResult::completed(serde_json::json!({
                    "chunks": chunks,
                    "sources": sources,
                }))
            }
            Err(e) => {
                tracing::error!("BridgeRagQuery: {}", e);
                NodeResult::failed(&e)
            }
        }
    }
}

register_node!(BridgeRagQueryNode);
