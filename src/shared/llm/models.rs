use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: Role,
    pub content: String,
}

impl LlmMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}

#[derive(Debug, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Enum,
}

#[derive(Debug, Clone, Default)]
pub struct ResponseSchema {
    pub format: OutputFormat,
    pub json_schema: Option<String>,
    pub allowed_choices: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub messages: Vec<LlmMessage>,
    /// Optional model override.
    pub model: Option<String>,
    /// Optional temperature override.
    pub temperature: Option<f32>,
    pub expected_output: ResponseSchema,
}

impl GenerateRequest {
    pub fn new(messages: Vec<LlmMessage>) -> Self {
        Self {
            messages,
            model: None,
            temperature: None,
            expected_output: ResponseSchema::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerateResponse {
    pub text: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct EmbedRequest {
    pub input: Vec<String>,
    pub model: Option<String>,
}

impl EmbedRequest {
    pub fn new(input: Vec<String>) -> Self {
        Self { input, model: None }
    }
}

#[derive(Debug, Clone)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
}
