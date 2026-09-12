use super::{EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse, LlmService};
use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;
use tracing::{info_span, Instrument};

/// Wraps an [`LlmService`] and emits a span for every call.
///
/// Spans carry the GenAI semantic-convention attributes so OTLP backends
/// (Tempo, Langfuse) can attribute a trace to the provider and model that
/// actually served it.
pub struct InstrumentedLlmService {
    system: String,
    /// The wrapped client's configured model. Recorded when a request does not
    /// override `model`, so traces are never attributed to a placeholder.
    default_model: String,
    inner: Arc<dyn LlmService>,
}

impl InstrumentedLlmService {
    pub fn new(
        system: impl Into<String>,
        default_model: impl Into<String>,
        inner: Arc<dyn LlmService>,
    ) -> Self {
        Self {
            system: system.into(),
            default_model: default_model.into(),
            inner,
        }
    }

    /// Resolve the model name to record for a request: the caller's override if
    /// present, otherwise the provider client's configured default.
    fn resolve_model(&self, requested: Option<&str>) -> String {
        requested.unwrap_or(&self.default_model).to_string()
    }
}

// `dyn LlmService` is not `Debug`, so a derive is not possible here. AGENTS.md
// requires public structs to be debuggable; this manual impl keeps the
// trait-object field out of the output while exposing the identifying fields.
impl fmt::Debug for InstrumentedLlmService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedLlmService")
            .field("system", &self.system)
            .field("default_model", &self.default_model)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl LlmService for InstrumentedLlmService {
    async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
        let model = self.resolve_model(req.model.as_deref());

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
                let span = tracing::Span::current();
                span.record("gen_ai.usage.input_tokens", resp.prompt_tokens);
                span.record("gen_ai.usage.output_tokens", resp.completion_tokens);
            }
            res
        }
        .instrument(span)
        .await
    }

    async fn embed(&self, req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
        let model = self.resolve_model(req.model.as_deref());

        let span = info_span!(
            "llm.embed",
            "gen_ai.system" = %self.system,
            "gen_ai.request.model" = %model,
            "langfuse.observation.type" = "embedding"
        );

        async move { self.inner.embed(req).await }
            .instrument(span)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::LlmMessage;
    use std::sync::Mutex;
    use tracing::Level;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::fmt::MakeWriter;

    /// In-memory sink for formatted span output.
    #[derive(Clone, Default)]
    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl CaptureWriter {
        fn contents(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().expect("capture lock")).into_owned()
        }
    }

    impl std::io::Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("capture lock").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for CaptureWriter {
        type Writer = CaptureWriter;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[derive(Default)]
    struct FakeLlm {
        generate_calls: Mutex<u32>,
        embed_calls: Mutex<u32>,
    }

    #[async_trait]
    impl LlmService for FakeLlm {
        async fn generate(&self, _req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
            *self.generate_calls.lock().expect("call counter") += 1;
            Ok(GenerateResponse {
                text: "ok".to_string(),
                prompt_tokens: 11,
                completion_tokens: 22,
            })
        }

        async fn embed(&self, req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
            *self.embed_calls.lock().expect("call counter") += 1;
            Ok(EmbedResponse {
                embeddings: vec![vec![0.0; req.input.len()]],
            })
        }
    }

    /// Install a subscriber that emits span-creation events into `sink`.
    fn capture_spans(sink: CaptureWriter) -> tracing::subscriber::DefaultGuard {
        let subscriber = tracing_subscriber::fmt()
            .with_writer(sink)
            .with_ansi(false)
            .with_span_events(FmtSpan::NEW)
            .with_max_level(Level::TRACE)
            .finish();
        tracing::subscriber::set_default(subscriber)
    }

    #[test]
    fn falls_back_to_configured_default_model() {
        let svc = InstrumentedLlmService::new("anthropic", "claude-3-5-sonnet-latest", {
            Arc::new(FakeLlm::default())
        });

        assert_eq!(svc.resolve_model(None), "claude-3-5-sonnet-latest");
        assert_eq!(svc.resolve_model(Some("override-model")), "override-model");
    }

    #[tokio::test]
    async fn generate_records_resolved_model_and_observation_type() {
        let sink = CaptureWriter::default();
        let _guard = capture_spans(sink.clone());

        let inner = Arc::new(FakeLlm::default());
        let svc =
            InstrumentedLlmService::new("anthropic", "claude-3-5-sonnet-latest", inner.clone());

        let resp = svc
            .generate(GenerateRequest::new(vec![LlmMessage::user("hi")]))
            .await
            .expect("generate should succeed");

        assert_eq!(resp.text, "ok");
        assert_eq!(*inner.generate_calls.lock().expect("call counter"), 1);

        let out = sink.contents();
        assert!(
            out.contains("gen_ai.request.model=claude-3-5-sonnet-latest"),
            "span did not record the configured default model: {out}"
        );
        assert!(
            out.contains("langfuse.observation.type=\"generation\""),
            "generate span was not labelled a generation: {out}"
        );
    }

    #[tokio::test]
    async fn embed_records_resolved_model_and_embedding_observation_type() {
        let sink = CaptureWriter::default();
        let _guard = capture_spans(sink.clone());

        let inner = Arc::new(FakeLlm::default());
        let svc = InstrumentedLlmService::new("openai", "text-embedding-3-small", inner.clone());

        let resp = svc
            .embed(EmbedRequest::new(vec!["hello".to_string()]))
            .await
            .expect("embed should succeed");

        assert_eq!(resp.embeddings.len(), 1);
        assert_eq!(*inner.embed_calls.lock().expect("call counter"), 1);

        let out = sink.contents();
        assert!(
            out.contains("gen_ai.request.model=text-embedding-3-small"),
            "embed span did not record the configured default model: {out}"
        );
        assert!(
            out.contains("langfuse.observation.type=\"embedding\""),
            "embed span was not labelled an embedding: {out}"
        );
    }
}
