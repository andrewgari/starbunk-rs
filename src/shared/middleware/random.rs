use super::MessageFilter;
use rand::Rng;
use serenity::all::{Context, Message};
use std::sync::Arc;

/// Passes with the given probability in `[0.0, 1.0]`.
/// A probability of 1.0 always passes; 0.0 never passes.
pub fn chance(probability: f64) -> Arc<dyn MessageFilter> {
    Arc::new(ChanceFilter(probability))
}

struct ChanceFilter(f64);

impl MessageFilter for ChanceFilter {
    fn check(&self, _ctx: &Context, _msg: &Message) -> bool {
        rand::thread_rng().gen::<f64>() < self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_msg() -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1",
            "channel_id": "1",
            "author": {
                "id": "1",
                "username": "testuser",
                "bot": false,
                "discriminator": "0",
                "public_flags": 0
            },
            "content": "hello",
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": [],
            "embeds": [],
            "pinned": false,
            "type": 0
        }))
        .expect("test message")
    }

    fn check_filter(filter: &dyn MessageFilter, msg: &Message) -> bool {
        // SAFETY: these filters declare `_ctx` and never dereference ctx.
        // A dangling pointer is used only to satisfy the type signature.
        let ctx_ptr = std::ptr::NonNull::<Context>::dangling();
        filter.check(unsafe { ctx_ptr.as_ref() }, msg)
    }

    #[test]
    fn chance_one_always_passes() {
        let msg = build_msg();
        let filter = chance(1.0);
        for _ in 0..20 {
            assert!(check_filter(&*filter, &msg));
        }
    }

    #[test]
    fn chance_zero_never_passes() {
        let msg = build_msg();
        let filter = chance(0.0);
        for _ in 0..20 {
            assert!(!check_filter(&*filter, &msg));
        }
    }
}
