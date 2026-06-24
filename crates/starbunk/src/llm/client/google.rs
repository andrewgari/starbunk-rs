use super::super::models::*;
use super::super::service::LlmService;
use anyhow::Context as _;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct GoogleClient {
    base_url: String,
    api_key: String,
    default_model: String,
    client: Client,
}

impl GoogleClient {
    pub fn new(base_url: Option<String>, api_key: String, model: String) -> Self {
        Self {
            base_url: base_url
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
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
struct ApiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(rename = "responseMimeType", skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
    #[serde(rename = "responseSchema", skip_serializing_if = "Option::is_none")]
    response_schema: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct ApiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: UsageMetadata,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
}

#[async_trait]
impl LlmService for GoogleClient {
    async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
        let model = req.model.as_deref().unwrap_or(&self.default_model);

        let mut system_instruction: Option<Content> = None;
        let mut contents = Vec::new();

        for m in req.messages {
            if m.role == Role::System {
                system_instruction = Some(Content {
                    role: None,
                    parts: vec![Part { text: m.content }],
                });
            } else {
                let role = if m.role == Role::Assistant {
                    "model".to_string()
                } else {
                    "user".to_string()
                };
                contents.push(Content {
                    role: Some(role),
                    parts: vec![Part { text: m.content }],
                });
            }
        }

        let response_mime_type = match req.expected_output.format {
            OutputFormat::Json => Some("application/json".to_string()),
            _ => None,
        };

        let response_schema = req
            .expected_output
            .json_schema
            .and_then(|s| serde_json::from_str(&s).ok())
            .map(clean_schema_for_gemini);

        let generation_config = if req.temperature.is_some()
            || response_mime_type.is_some()
            || response_schema.is_some()
        {
            Some(GenerationConfig {
                temperature: req.temperature,
                response_mime_type,
                response_schema,
            })
        } else {
            None
        };

        let body = ApiRequest {
            system_instruction,
            contents,
            generation_config,
        };

        let url = format!("{}/models/{}:generateContent", self.base_url, model);
        let resp = self
            .client
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&body)
            .send()
            .await
            .context("google: request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "google: unexpected status {}: {}",
                status,
                text
            ));
        }

        let api_resp: ApiResponse = resp
            .json()
            .await
            .context("google: failed to decode response")?;

        let candidate = api_resp
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("google: no candidates returned"))?;

        let text = candidate
            .content
            .parts
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("google: no parts in response"))?
            .text;

        Ok(GenerateResponse {
            text,
            prompt_tokens: api_resp.usage_metadata.prompt_token_count,
            completion_tokens: api_resp.usage_metadata.candidates_token_count,
        })
    }

    async fn embed(&self, req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
        let model = req.model.as_deref().unwrap_or("text-embedding-004");

        let mut embeddings = Vec::new();

        for text in req.input {
            let model_path = format!("models/{}", model);
            let body = EmbedApiRequest {
                model: model_path.as_str(),
                content: Content {
                    role: None,
                    parts: vec![Part { text }],
                },
            };

            let url = format!("{}/models/{}:embedContent", self.base_url, model);
            let resp = self
                .client
                .post(&url)
                .query(&[("key", &self.api_key)])
                .json(&body)
                .send()
                .await
                .context("google: embed request failed")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "google: unexpected embed status {}: {}",
                    status,
                    text
                ));
            }

            let api_resp: EmbedApiResponse = resp
                .json()
                .await
                .context("google: failed to decode embed response")?;

            embeddings.push(api_resp.embedding.values);
        }

        Ok(EmbedResponse { embeddings })
    }
}

#[derive(Serialize)]
struct EmbedApiRequest<'a> {
    model: &'a str,
    content: Content,
}

#[derive(Deserialize)]
struct EmbedApiResponse {
    embedding: EmbedValues,
}

#[derive(Deserialize)]
struct EmbedValues {
    values: Vec<f32>,
}

fn clean_schema_for_gemini(mut value: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(ref mut map) = value {
        map.remove("$schema");
        map.remove("$defs");
        map.remove("definitions");
        map.remove("title");

        for (_, v) in map.iter_mut() {
            let old_v = std::mem::replace(v, serde_json::Value::Null);
            *v = clean_schema_for_gemini(old_v);
        }
    } else if let serde_json::Value::Array(ref mut arr) = value {
        for v in arr.iter_mut() {
            let old_v = std::mem::replace(v, serde_json::Value::Null);
            *v = clean_schema_for_gemini(old_v);
        }
    }
    value
}
