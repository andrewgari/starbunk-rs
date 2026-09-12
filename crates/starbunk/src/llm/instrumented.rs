use super::{EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse, LlmService};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info_span, Instrument};

pub struct InstrumentedLlmService {
    system: String,
    inner: Arc<dyn LlmService>,
}

impl InstrumentedLlmService {
    pub fn new(system: impl Into<String>, inner: Arc<dyn LlmService>) -> Self {
        Self {
            system: system.into(),
            inner,
        }
    }
}

#[async_trait]
impl LlmService for InstrumentedLlmService {
    async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
        let model = req.model.as_deref().unwrap_or("default").to_string();
        
        let span = info_span!(
            "llm.generate",
            "gen_ai.system" = %self.system,
            "gen_ai.request.model" = %model,
            "gen_ai.usage.input_tokens" = tracing::field::Empty,
            "gen_ai.usage.output_tokens" = tracing::field::Empty,
            "langfuse.observation.type" = "generation"
        );
        
        async move {
            let res = self.inner.generate(req).await;
            if let Ok(resp) = &res {
                tracing::Span::current().record("gen_ai.usage.input_tokens", resp.prompt_tokens);
                tracing::Span::current().record("gen_ai.usage.output_tokens", resp.completion_tokens);
            }
            res
        }
        .instrument(span)
        .await
    }

    async fn embed(&self, req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
        let model = req.model.as_deref().unwrap_or("default").to_string();
        
        let span = info_span!(
            "llm.embed",
            "gen_ai.system" = %self.system,
            "gen_ai.request.model" = %model,
            "langfuse.observation.type" = "generation"
        );
        
        async move {
            self.inner.embed(req).await
        }
        .instrument(span)
        .await
    }
}
