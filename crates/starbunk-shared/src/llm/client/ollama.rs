use super::super::models;
use super::super::models::*;
use super::super::service::LlmService;
use anyhow::Context as _;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaClient {
    base_url: String,
    default_model: String,
    client: Client,
}

impl OllamaClient {
    pub fn new(base_url: Option<String>, model: String) -> Self {
        Self {
            base_url: base_url
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            default_model: model,
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ApiMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Options>,
}

#[derive(Serialize)]
struct Options {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ApiMessage,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Serialize)]
struct EmbedApiRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Deserialize)]
struct EmbedApiResponse {
    embeddings: Vec<Vec<f32>>,
}

#[async_trait]
impl LlmService for OllamaClient {
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

        let options = req.temperature.map(|t| Options { temperature: Some(t) });

        let body = ChatRequest {
            model,
            messages,
            stream: false,
            options,
        };

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("ollama: request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("ollama: unexpected status {}: {}", status, text));
        }

        let api_resp: ChatResponse = resp
            .json()
            .await
            .context("ollama: failed to decode response")?;

        Ok(GenerateResponse {
            text: api_resp.message.content,
            prompt_tokens: api_resp.prompt_eval_count.unwrap_or(0),
            completion_tokens: api_resp.eval_count.unwrap_or(0),
        })
    }

    async fn embed(&self, req: models::EmbedRequest) -> anyhow::Result<models::EmbedResponse> {
        let model = req.model.as_deref().unwrap_or(&self.default_model);

        let body = EmbedApiRequest {
            model,
            input: &req.input,
        };

        let url = format!("{}/api/embed", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("ollama: embed request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "ollama: unexpected embed status {}: {}",
                status,
                text
            ));
        }

        let api_resp: EmbedApiResponse = resp
            .json()
            .await
            .context("ollama: failed to decode embed response")?;

        Ok(models::EmbedResponse {
            embeddings: api_resp.embeddings,
        })
    }
}
