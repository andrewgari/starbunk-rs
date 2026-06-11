use super::models::*;
use async_trait::async_trait;
use std::sync::Arc;

/// Shared abstraction for interacting with an LLM provider.
#[async_trait]
pub trait LlmService: Send + Sync {
    async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse>;
    async fn embed(&self, req: EmbedRequest) -> anyhow::Result<EmbedResponse>;
}

/// Tiered registry of LLM services. High tier is most capable (and expensive),
/// low tier is fastest / cheapest. Each accessor falls back to an available tier.
pub trait Registry: Send + Sync {
    fn high(&self) -> Option<Arc<dyn LlmService>>;
    fn medium(&self) -> Option<Arc<dyn LlmService>>;
    fn low(&self) -> Option<Arc<dyn LlmService>>;
}

pub(super) struct TieredRegistry {
    pub high: Option<Arc<dyn LlmService>>,
    pub medium: Option<Arc<dyn LlmService>>,
    pub low: Option<Arc<dyn LlmService>>,
}

impl Registry for TieredRegistry {
    fn high(&self) -> Option<Arc<dyn LlmService>> {
        self.high
            .clone()
            .or_else(|| self.medium.clone())
            .or_else(|| self.low.clone())
    }

    fn medium(&self) -> Option<Arc<dyn LlmService>> {
        self.medium
            .clone()
            .or_else(|| self.high.clone())
            .or_else(|| self.low.clone())
    }

    fn low(&self) -> Option<Arc<dyn LlmService>> {
        self.low
            .clone()
            .or_else(|| self.medium.clone())
            .or_else(|| self.high.clone())
    }
}
