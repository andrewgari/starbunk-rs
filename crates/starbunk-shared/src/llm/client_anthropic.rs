use super::models::*;
use super::service::LlmService;
use anyhow::Context as _;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct AnthropicClient {
    base_url: String,
    api_key: String,
    default_model: String,
    client: Client,
}

impl AnthropicClient {
    pub fn new(base_url: Option<String>, api_key: String, model: String) -> Self {
        Self {
            base_url: base_url
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            api_key,
            default_model: model,
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
        }
    }
}

#[derive(Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LlmService for AnthropicClient {
    async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
        let model = req
            .model
            .as_deref()
            .unwrap_or(&self.default_model);

        let mut system: Option<String> = None;
        let mut messages = Vec::new();

        for m in req.messages {
            if m.role == Role::System {
                system = Some(m.content);
            } else {
                messages.push(ApiMessage {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content,
                });
            }
        }

        let body = ApiRequest {
            model,
            messages,
            system,
            max_tokens: 4096,
            temperature: req.temperature,
        };

        let url = format!("{}/messages", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("anthropic: request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "anthropic: unexpected status {}: {}",
                status,
                text
            ));
        }

        let api_resp: ApiResponse = resp
            .json()
            .await
            .context("anthropic: failed to decode response")?;

        let block = api_resp
            .content
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("anthropic: no content returned"))?;

        Ok(GenerateResponse {
            text: block.text,
            prompt_tokens: api_resp.usage.input_tokens,
            completion_tokens: api_resp.usage.output_tokens,
        })
    }

    async fn embed(&self, _req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
        Err(anyhow::anyhow!("anthropic: embed not supported"))
    }
}
