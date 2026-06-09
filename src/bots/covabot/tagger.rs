use crate::shared::llm::{GenerateRequest, LlmMessage, LlmService, OutputFormat, ResponseSchema};
use anyhow::Context as _;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralTags {
    pub addressee: Option<Addressee>,
    pub intent: Option<Intent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    async fn tag_message(
        &self,
        content: &str,
        ctx: TaggingContext,
    ) -> anyhow::Result<TagResult>;
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
    async fn tag_message(
        &self,
        content: &str,
        ctx: TaggingContext,
    ) -> anyhow::Result<TagResult> {
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

        let resp = self.llm.generate(req).await.context("tagger: generate failed")?;

        serde_json::from_str(&resp.text).context("tagger: failed to parse JSON response")
    }
}
