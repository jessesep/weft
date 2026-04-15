//! ONE Bridge LLM Node - Call Claude through the ONE cc-bridge infrastructure

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node};

/// ONE Bridge LLM node for calling Claude through cc-bridge.
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

        let url = format!("{}/api/weft/llm", bridge_url);

        tracing::info!(
            "ONE Bridge LLM request: url={}, prompt_len={}, model={}",
            url,
            prompt.len(),
            if model.is_empty() { "(default)" } else { model }
        );

        // Build request body
        let mut body = serde_json::json!({
            "prompt": prompt,
            "max_tokens": max_tokens,
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

        let client = ctx.http_client.clone();

        let result = client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await;

        match result {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    let error_body = response.text().await.unwrap_or_default();
                    tracing::error!("ONE Bridge LLM error (HTTP {}): {}", status, error_body);
                    return NodeResult::failed(
                        &format!("Bridge returned HTTP {}: {}", status, error_body)
                    );
                }

                match response.json::<serde_json::Value>().await {
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

                        tracing::info!(
                            "ONE Bridge LLM response: model={}, tokens={}/{}",
                            resp_model, input_tokens, output_tokens
                        );

                        NodeResult::completed(serde_json::json!({
                            "response": text,
                            "model": resp_model,
                            "input_tokens": input_tokens,
                            "output_tokens": output_tokens,
                        }))
                    }
                    Err(e) => {
                        tracing::error!("ONE Bridge LLM: failed to parse response JSON: {}", e);
                        NodeResult::failed(&format!("Failed to parse bridge response: {}", e))
                    }
                }
            }
            Err(e) => {
                tracing::error!("ONE Bridge LLM: request failed: {}", e);
                NodeResult::failed(&format!("Bridge request failed: {}", e))
            }
        }
    }
}

register_node!(OneBridgeLlmNode);
