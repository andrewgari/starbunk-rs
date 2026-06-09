use super::super::models::*;
use super::super::service::LlmService;
use anyhow::Context as _;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OpenAiClient {
    base_url: String,
    api_key: String,
    default_model: String,
    client: Client,
}

impl OpenAiClient {
    pub fn new(base_url: Option<String>, api_key: String, model: String) -> Self {
        Self {
            base_url: base_url
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
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
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: ApiMessage,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Serialize)]
struct EmbedApiRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Deserialize)]
struct EmbedApiResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

#[async_trait]
impl LlmService for OpenAiClient {
    async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
        let model = req
            .model
            .as_deref()
            .unwrap_or(&self.default_model);

        let messages: Vec<ApiMessage> = req
            .messages
            .into_iter()
            .map(|m| ApiMessage {
                role: format!("{:?}", m.role).to_lowercase(),
                content: m.content,
            })
            .collect();

        let body = ApiRequest {
            model,
            messages,
            temperature: req.temperature,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("openai: request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("openai: unexpected status {}: {}", status, text));
        }

        let api_resp: ApiResponse = resp
            .json()
            .await
            .context("openai: failed to decode response")?;

        let choice = api_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("openai: no choices returned"))?;

        Ok(GenerateResponse {
            text: choice.message.content,
            prompt_tokens: api_resp.usage.prompt_tokens,
            completion_tokens: api_resp.usage.completion_tokens,
        })
    }

    async fn embed(&self, req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
        let model = req
            .model
            .as_deref()
            .unwrap_or("text-embedding-3-small");

        let body = EmbedApiRequest {
            model,
            input: &req.input,
        };

        let url = format!("{}/embeddings", self.base_url);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("openai: embed request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "openai: unexpected embed status {}: {}",
                status,
                text
            ));
        }

        let api_resp: EmbedApiResponse = resp
            .json()
            .await
            .context("openai: failed to decode embed response")?;

        Ok(EmbedResponse {
            embeddings: api_resp.data.into_iter().map(|d| d.embedding).collect(),
        })
    }
}
