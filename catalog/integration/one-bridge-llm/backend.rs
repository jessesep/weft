//! ONE Bridge LLM Node - Call Claude through the ONE cc-bridge infrastructure

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node, one_bridge_http};

#[derive(Default)]
pub struct OneBridgeLlmNode;

#[async_trait]
impl Node for OneBridgeLlmNode {
    fn node_type(&self) -> &'static str {
        "OneBridgeLlm"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "ONE Bridge LLM",
            inputs: vec![
                PortDef::new("prompt", "String", true),
                PortDef::new("system_prompt", "String", false),
                PortDef::new("model", "String", false),
            ],
            outputs: vec![
                PortDef::new("response", "String", false),
                PortDef::new("model", "String", false),
                PortDef::new("input_tokens", "Number", false),
                PortDef::new("output_tokens", "Number", false),
                PortDef::new("cost_usd", "Number", false),
            ],
            features: NodeFeatures {
                ..Default::default()
            },
            fields: vec![
                FieldDef::text("bridge_url").with_default(serde_json::json!("http://localhost:8070")),
                FieldDef::number("max_tokens").with_default(serde_json::json!(4096)),
                FieldDef::number("temperature"),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let bridge_url = ctx.config.get("bridge_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:8070");

        let max_tokens = ctx.config_u64("max_tokens", 4096);

        let temperature = ctx.config.get("temperature")
            .and_then(|v| v.as_f64());

        let prompt = ctx.input.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if prompt.is_empty() {
            return NodeResult::failed("Prompt is required and cannot be empty");
        }

        let system_prompt = ctx.input.get("system_prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let model = ctx.input.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "ONE Bridge LLM: prompt_len={}, model={}",
            prompt.len(),
            if model.is_empty() { "(default)" } else { model }
        );

        let mut body = serde_json::json!({
            "prompt": prompt,
            "max_tokens": max_tokens,
            "execution_id": ctx.executionId,
            "node_id": ctx.nodeId,
            "project_id": ctx.projectId.clone().unwrap_or_default(),
        });

        if !system_prompt.is_empty() {
            body["system_prompt"] = serde_json::json!(system_prompt);
        }
        if !model.is_empty() {
            body["model"] = serde_json::json!(model);
        }
        if let Some(temp) = temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        match one_bridge_http::bridge_post(&ctx.http_client, bridge_url, "/api/weft/llm", body).await {
            Ok(resp_json) => {
                let text = resp_json.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let resp_model = resp_json.get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let input_tokens = resp_json.get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let output_tokens = resp_json.get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_read_tokens = resp_json.get("cache_read_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_creation_tokens = resp_json.get("cache_creation_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cost_usd = resp_json.get("cost_usd")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                tracing::info!(
                    "ONE Bridge LLM: model={}, tokens={}/{}, cost=${:.6}",
                    resp_model, input_tokens, output_tokens, cost_usd
                );

                if cost_usd > 0.0 {
                    ctx.report_usage_cost(
                        &format!("one-bridge-llm:{}", resp_model),
                        "llm",
                        cost_usd,
                        false,
                        Some(serde_json::json!({
                            "inputTokens": input_tokens,
                            "outputTokens": output_tokens,
                            "cacheReadTokens": cache_read_tokens,
                            "cacheCreationTokens": cache_creation_tokens,
                        })),
                    ).await;
                }

                NodeResult::completed(serde_json::json!({
                    "response": text,
                    "model": resp_model,
                    "input_tokens": input_tokens,
                    "output_tokens": output_tokens,
                    "cost_usd": cost_usd,
                }))
            }
            Err(e) => {
                tracing::error!("ONE Bridge LLM: {}", e);
                NodeResult::failed(&e)
            }
        }
    }
}

register_node!(OneBridgeLlmNode);
