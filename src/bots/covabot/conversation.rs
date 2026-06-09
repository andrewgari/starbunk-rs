use crate::shared::llm::{EmbedRequest, LlmService};
use anyhow::Context as _;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

const T_HIGH: f32 = 0.75;
const T_LOW: f32 = 0.45;

pub struct ActiveConversation {
    pub id: String,
    pub centroid: Vec<f32>,
    pub last_activity: Instant,
    pub message_count: u32,
}

#[async_trait]
pub trait Tracker: Send + Sync {
    /// Assign `tags` in `channel_id` to one or more active conversations.
    /// Returns the conversation IDs the message was assigned to.
    async fn assign(
        &self,
        channel_id: &str,
        tags: &[String],
    ) -> anyhow::Result<Vec<String>>;
}

pub struct LlmTracker {
    llm: Arc<dyn LlmService>,
    live: RwLock<HashMap<String, Vec<ActiveConversation>>>,
}

impl LlmTracker {
    pub fn new(llm: Arc<dyn LlmService>) -> Self {
        Self {
            llm,
            live: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Tracker for LlmTracker {
    async fn assign(&self, channel_id: &str, tags: &[String]) -> anyhow::Result<Vec<String>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let emb_resp = self
            .llm
            .embed(EmbedRequest::new(tags.to_vec()))
            .await
            .context("conversation: embed failed")?;

        let msg_centroid = average_embeddings(&emb_resp.embeddings);
        let Some(msg_centroid) = msg_centroid else {
            return Ok(Vec::new());
        };

        let mut live = self.live.write().await;
        let convs = live.entry(channel_id.to_string()).or_default();

        let mut assigned: Vec<String> = Vec::new();
        let mut best_ambiguous: Option<(usize, f32)> = None; // (index, similarity)

        for (i, conv) in convs.iter_mut().enumerate() {
            let sim = cosine_similarity(&msg_centroid, &conv.centroid);
            if sim >= T_HIGH {
                assigned.push(conv.id.clone());
                update_centroid(conv, &msg_centroid);
                conv.last_activity = Instant::now();
            } else if sim >= T_LOW {
                let is_better = best_ambiguous.map(|(_, s)| sim > s).unwrap_or(true);
                if is_better {
                    best_ambiguous = Some((i, sim));
                }
            }
        }

        if assigned.is_empty() {
            if let Some((idx, _)) = best_ambiguous {
                let conv = &mut convs[idx];
                assigned.push(conv.id.clone());
                update_centroid(conv, &msg_centroid);
                conv.last_activity = Instant::now();
            } else {
                let id = format!("conv-{}", generate_id());
                convs.push(ActiveConversation {
                    id: id.clone(),
                    centroid: msg_centroid,
                    last_activity: Instant::now(),
                    message_count: 1,
                });
                assigned.push(id);
            }
        }

        Ok(assigned)
    }
}

fn update_centroid(conv: &mut ActiveConversation, msg_centroid: &[f32]) {
    conv.message_count += 1;
    let n = conv.message_count as f32;
    for (i, v) in conv.centroid.iter_mut().enumerate() {
        *v += (msg_centroid[i] - *v) / n;
    }
}

fn average_embeddings(embs: &[Vec<f32>]) -> Option<Vec<f32>> {
    let first = embs.first()?;
    let dim = first.len();
    let mut sum = vec![0f32; dim];
    let mut valid = 0u32;

    for emb in embs {
        if emb.len() != dim {
            continue;
        }
        for (i, v) in emb.iter().enumerate() {
            sum[i] += v;
        }
        valid += 1;
    }

    if valid == 0 {
        return None;
    }

    Some(sum.into_iter().map(|v| v / valid as f32).collect())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let (mut dot, mut norm_a, mut norm_b) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}

fn generate_id() -> String {
    use rand::Rng;
    let bytes: [u8; 8] = rand::thread_rng().gen();
    hex::encode(bytes)
}

// hex encoding without an extra dep
mod hex {
    pub fn encode(bytes: [u8; 8]) -> String {
        bytes.iter().fold(String::new(), |mut s, b| {
            s.push_str(&format!("{:02x}", b));
            s
        })
    }
}
