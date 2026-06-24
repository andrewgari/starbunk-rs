use anyhow::Context as _;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starbunk::llm::{GenerateRequest, LlmMessage, LlmService, OutputFormat, ResponseSchema};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Addressee {
    #[serde(rename = "self")]
    SelfAddr,
    OtherUser,
    Room,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intent {
    Question,
    Statement,
    #[serde(rename = "low-effort")]
    LowEffort,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuralTags {
    pub addressee: Option<Addressee>,
    pub intent: Option<Intent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TagResult {
    pub topical_tags: Vec<String>,
    pub structural: StructuralTags,
}

#[derive(Debug, Default)]
pub struct TaggingContext {
    pub thread_context: String,
    pub active_conversations: Vec<String>,
    pub recent_conversations: Vec<String>,
    pub currently_used_tags: Vec<String>,
}

#[async_trait]
pub trait TaggerService: Send + Sync {
    async fn tag_message(&self, content: &str, ctx: TaggingContext) -> anyhow::Result<TagResult>;
}

pub struct LlmTagger {
    llm: Arc<dyn LlmService>,
}

impl LlmTagger {
    pub fn new(llm: Arc<dyn LlmService>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl TaggerService for LlmTagger {
    async fn tag_message(&self, content: &str, ctx: TaggingContext) -> anyhow::Result<TagResult> {
        let mut system_prompt = String::from(
            "You are an analytical conversation tagger. Extract topical tags and structural \
             tags from the message.\n\n\
             Guidelines for topical tags:\n\
             - Topical tags should be broad concepts or specific entities (e.g., 'programming', \
               'binary search').\n\
             - Combine very generic and very specific tags. Example: For \"I can't believe they \
               killed off the main character in the first chapter!\", tags might be: \"books\", \
               \"reading\", \"plot twist\", \"talking about shocking story events\", \
               \"game of thrones\".\n\
             - If an existing tag perfectly matches the topic, reuse it instead of creating a duplicate.\n\
             - Figure out the best tag or tags given the current conversations context.\n\
             - If the message is purely conversational ('lol', 'yeah'), topical_tags should be empty.\n\n\
             Addressee must be one of: 'self', 'other-user', 'room'.\n\
             Intent must be one of: 'question', 'statement', 'low-effort'.",
        );

        if !ctx.thread_context.is_empty() {
            system_prompt.push_str("\n\nThread Context:\n");
            system_prompt.push_str(&ctx.thread_context);
        }
        if !ctx.active_conversations.is_empty() {
            system_prompt.push_str("\n\nActive Conversations:");
            for c in &ctx.active_conversations {
                system_prompt.push_str("\n- ");
                system_prompt.push_str(c);
            }
        }
        if !ctx.recent_conversations.is_empty() {
            system_prompt.push_str("\n\nRecent Conversations:");
            for c in &ctx.recent_conversations {
                system_prompt.push_str("\n- ");
                system_prompt.push_str(c);
            }
        }
        if !ctx.currently_used_tags.is_empty() {
            system_prompt.push_str("\n\nCurrently Used Tags:");
            for t in &ctx.currently_used_tags {
                system_prompt.push_str("\n- ");
                system_prompt.push_str(t);
            }
        }

        let mut req = GenerateRequest::new(vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(content),
        ]);
        req.expected_output = ResponseSchema {
            format: OutputFormat::Json,
            ..Default::default()
        };

        let resp = self
            .llm
            .generate(req)
            .await
            .context("tagger: generate failed")?;

        tracing::warn!("raw tagger response: {}", resp.text);

        let val: serde_json::Value =
            serde_json::from_str(&resp.text).context("tagger: failed to parse JSON response")?;

        let mut topical_tags = vec![];
        if let Some(arr) = val.get("topical_tags").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    topical_tags.push(s.to_string());
                }
            }
        }

        let mut structural = StructuralTags::default();
        let struct_obj = val.get("structural").or_else(|| val.get("structural_tags"));
        if let Some(obj) = struct_obj {
            if let Some(a) = obj.get("addressee").and_then(|v| v.as_str()) {
                structural.addressee =
                    serde_json::from_value(serde_json::Value::String(a.to_string())).ok();
            }
            if let Some(i) = obj.get("intent").and_then(|v| v.as_str()) {
                structural.intent =
                    serde_json::from_value(serde_json::Value::String(i.to_string())).ok();
            }
        } else {
            if let Some(a) = val.get("addressee").and_then(|v| v.as_str()) {
                structural.addressee =
                    serde_json::from_value(serde_json::Value::String(a.to_string())).ok();
            }
            if let Some(i) = val.get("intent").and_then(|v| v.as_str()) {
                structural.intent =
                    serde_json::from_value(serde_json::Value::String(i.to_string())).ok();
            }
        }

        Ok(TagResult {
            topical_tags,
            structural,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starbunk::llm::{EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse};
    use std::sync::Arc;

    struct MockLlm {
        response_json: &'static str,
    }

    #[async_trait::async_trait]
    impl starbunk::llm::LlmService for MockLlm {
        async fn generate(&self, _req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
            Ok(GenerateResponse {
                text: self.response_json.to_string(),
                prompt_tokens: 0,
                completion_tokens: 0,
            })
        }

        async fn embed(&self, _req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
            Ok(EmbedResponse { embeddings: vec![] })
        }
    }

    #[tokio::test]
    async fn parses_valid_response() {
        let llm = Arc::new(MockLlm {
            response_json: r#"{"topical_tags":["programming","rust"],"structural":{"addressee":"room","intent":"statement"}}"#,
        });
        let tagger = LlmTagger::new(llm);
        let result = tagger
            .tag_message("I love Rust", TaggingContext::default())
            .await
            .expect("should parse");
        assert_eq!(result.topical_tags, vec!["programming", "rust"]);
        assert_eq!(result.structural.addressee, Some(Addressee::Room));
        assert_eq!(result.structural.intent, Some(Intent::Statement));
    }

    #[tokio::test]
    async fn handles_empty_topical_tags() {
        let llm = Arc::new(MockLlm {
            response_json: r#"{"topical_tags":[],"structural":{"addressee":"self","intent":"low-effort"}}"#,
        });
        let tagger = LlmTagger::new(llm);
        let result = tagger
            .tag_message("lol", TaggingContext::default())
            .await
            .expect("should parse");
        assert!(result.topical_tags.is_empty());
        assert_eq!(result.structural.addressee, Some(Addressee::SelfAddr));
        assert_eq!(result.structural.intent, Some(Intent::LowEffort));
    }

    #[tokio::test]
    async fn prompt_includes_tag_guidance() {
        use std::sync::Mutex;

        struct CaptureLlm {
            captured: Mutex<Option<GenerateRequest>>,
        }

        #[async_trait::async_trait]
        impl starbunk::llm::LlmService for CaptureLlm {
            async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
                *self.captured.lock().unwrap() = Some(req);
                Ok(GenerateResponse {
                    text: r#"{"topical_tags":[],"structural":{"addressee":"room","intent":"statement"}}"#.to_string(),
                    prompt_tokens: 0,
                    completion_tokens: 0,
                })
            }
            async fn embed(&self, _req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
                Ok(EmbedResponse { embeddings: vec![] })
            }
        }

        let llm = Arc::new(CaptureLlm {
            captured: Mutex::new(None),
        });
        let tagger = LlmTagger::new(llm.clone());
        tagger
            .tag_message("test message", TaggingContext::default())
            .await
            .unwrap();

        let captured = llm.captured.lock().unwrap();
        let req = captured.as_ref().expect("request was captured");
        let system_content = req
            .messages
            .iter()
            .find(|m| matches!(m.role, starbunk::llm::Role::System))
            .map(|m| m.content.as_str())
            .unwrap_or("");
        assert!(system_content.contains("topical tags"));
        assert!(system_content.contains("Addressee must be one of"));
        assert!(system_content.contains("Intent must be one of"));
    }

    #[tokio::test]
    async fn prompt_includes_context_data() {
        use std::sync::Mutex;

        struct CaptureLlm {
            captured: Mutex<Option<GenerateRequest>>,
        }

        #[async_trait::async_trait]
        impl starbunk::llm::LlmService for CaptureLlm {
            async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> {
                *self.captured.lock().unwrap() = Some(req);
                Ok(GenerateResponse {
                    text: r#"{"topical_tags":[],"structural":{"addressee":"room","intent":"statement"}}"#.to_string(),
                    prompt_tokens: 0,
                    completion_tokens: 0,
                })
            }
            async fn embed(&self, _req: EmbedRequest) -> anyhow::Result<EmbedResponse> {
                Ok(EmbedResponse { embeddings: vec![] })
            }
        }

        let llm = Arc::new(CaptureLlm {
            captured: Mutex::new(None),
        });
        let tagger = LlmTagger::new(llm.clone());
        let ctx = TaggingContext {
            thread_context: "some thread".to_string(),
            active_conversations: vec!["active convo".to_string()],
            recent_conversations: vec!["recent convo".to_string()],
            currently_used_tags: vec!["existing-tag".to_string()],
        };
        tagger.tag_message("hello", ctx).await.unwrap();

        let captured = llm.captured.lock().unwrap();
        let req = captured.as_ref().expect("request was captured");
        let system_content = req
            .messages
            .iter()
            .find(|m| matches!(m.role, starbunk::llm::Role::System))
            .map(|m| m.content.as_str())
            .unwrap_or("");
        assert!(system_content.contains("some thread"));
        assert!(system_content.contains("active convo"));
        assert!(system_content.contains("recent convo"));
        assert!(system_content.contains("existing-tag"));
    }
}
