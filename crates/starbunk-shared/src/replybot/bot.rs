use super::strategy::Strategy;
use crate::discord::MessageService;
use serenity::all::{Context, Message};
use std::sync::Arc;

/// Dispatches incoming Discord messages through an ordered list of strategies.
/// The first strategy whose `should_trigger` returns true wins; the rest are
/// skipped. Optionally pre-condition guards run before `should_trigger`.
pub struct ReplyBot {
    strategies: Vec<Box<dyn Strategy>>,
    sender: Arc<dyn MessageService>,
}

impl ReplyBot {
    pub fn new(sender: Arc<dyn MessageService>, strategies: Vec<Box<dyn Strategy>>) -> Self {
        Self { strategies, sender }
    }

    pub async fn handle(&self, ctx: &Context, msg: &Message) {
        for strategy in &self.strategies {
            // Check optional per-strategy condition.
            if let Some(cond) = strategy.condition() {
                if !cond.check(ctx, msg) {
                    continue;
                }
            }

            if !strategy.should_trigger(ctx, msg).await {
                continue;
            }

            let resp = strategy.response(ctx, msg).await;

            if let Some(identity) = strategy.identity(ctx, msg).await {
                if let Err(e) = self
                    .sender
                    .send_with_identity(msg.channel_id, &resp, identity)
                    .await
                {
                    tracing::error!(
                        strategy = strategy.name(),
                        channel = %msg.channel_id,
                        "failed to send identified response: {}",
                        e
                    );
                }
            } else if let Err(e) = self.sender.send(msg.channel_id, &resp).await {
                tracing::error!(
                    strategy = strategy.name(),
                    channel = %msg.channel_id,
                    "failed to send response: {}",
                    e
                );
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discord::Identity;
    use async_trait::async_trait;
    use serenity::all::{ChannelId, MessageId};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    // --- helpers ---

    fn build_msg() -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1",
            "channel_id": "99",
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

    /// Returns a dangling pointer to a Context. Only safe to pass to handlers
    /// whose mock strategies never dereference the context.
    fn fake_ctx() -> *const Context {
        std::ptr::NonNull::<Context>::dangling().as_ptr()
    }

    // --- mock MessageService ---

    struct MockSender {
        send_count: AtomicUsize,
        identity_count: AtomicUsize,
        last_content: Mutex<String>,
    }

    impl MockSender {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                send_count: AtomicUsize::new(0),
                identity_count: AtomicUsize::new(0),
                last_content: Mutex::new(String::new()),
            })
        }
    }

    fn stub_message() -> Message {
        build_msg()
    }

    #[async_trait]
    impl MessageService for MockSender {
        async fn send(&self, _channel_id: ChannelId, content: &str) -> anyhow::Result<Message> {
            self.send_count.fetch_add(1, Ordering::SeqCst);
            *self.last_content.lock().unwrap() = content.to_string();
            Ok(stub_message())
        }
        async fn send_with_identity(
            &self,
            _channel_id: ChannelId,
            content: &str,
            _identity: Identity,
        ) -> anyhow::Result<Message> {
            self.identity_count.fetch_add(1, Ordering::SeqCst);
            *self.last_content.lock().unwrap() = content.to_string();
            Ok(stub_message())
        }
        async fn reply(
            &self,
            _channel_id: ChannelId,
            _message_id: MessageId,
            _content: &str,
        ) -> anyhow::Result<Message> {
            Ok(stub_message())
        }
        async fn edit(
            &self,
            _channel_id: ChannelId,
            _message_id: MessageId,
            _content: &str,
        ) -> anyhow::Result<Message> {
            Ok(stub_message())
        }
        async fn delete(
            &self,
            _channel_id: ChannelId,
            _message_id: MessageId,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        async fn close(&self) {}
    }

    // --- mock Strategy ---

    struct MockStrategy {
        name: &'static str,
        trigger: bool,
        response: &'static str,
        trigger_count: AtomicUsize,
        use_identity: bool,
    }

    impl MockStrategy {
        fn new(name: &'static str, trigger: bool, response: &'static str) -> Self {
            Self {
                name,
                trigger,
                response,
                trigger_count: AtomicUsize::new(0),
                use_identity: false,
            }
        }

        fn with_identity(mut self) -> Self {
            self.use_identity = true;
            self
        }
    }

    #[async_trait]
    impl Strategy for MockStrategy {
        fn name(&self) -> &str {
            self.name
        }

        async fn should_trigger(&self, _ctx: &Context, _msg: &Message) -> bool {
            self.trigger_count.fetch_add(1, Ordering::SeqCst);
            self.trigger
        }

        async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
            self.response.to_string()
        }

        async fn identity(&self, _ctx: &Context, _msg: &Message) -> Option<Identity> {
            if self.use_identity {
                Some(Identity {
                    username: "TestBot".to_string(),
                    avatar_url: "http://example.com/avatar.png".to_string(),
                    ..Default::default()
                })
            } else {
                None
            }
        }
    }

    // --- tests ---

    #[tokio::test]
    async fn sends_response_of_first_triggering_strategy() {
        let sender = MockSender::new();
        let bot = ReplyBot::new(
            sender.clone(),
            vec![Box::new(MockStrategy::new("s1", true, "hello from s1"))],
        );
        let msg = build_msg();
        let ctx = fake_ctx();
        bot.handle(unsafe { &*ctx }, &msg).await;
        assert_eq!(sender.send_count.load(Ordering::SeqCst), 1);
        assert_eq!(*sender.last_content.lock().unwrap(), "hello from s1");
    }

    #[tokio::test]
    async fn first_match_wins_second_strategy_not_called() {
        let sender = MockSender::new();
        let s2 = MockStrategy::new("s2", true, "hello from s2");
        // We can't inspect trigger_count after moving s2 into the box, so we
        // verify via the response content: s1 fires first.
        let bot = ReplyBot::new(
            sender.clone(),
            vec![
                Box::new(MockStrategy::new("s1", true, "first")),
                Box::new(s2),
            ],
        );
        let msg = build_msg();
        let ctx = fake_ctx();
        bot.handle(unsafe { &*ctx }, &msg).await;
        assert_eq!(sender.send_count.load(Ordering::SeqCst), 1);
        assert_eq!(*sender.last_content.lock().unwrap(), "first");
    }

    #[tokio::test]
    async fn non_triggering_strategy_produces_no_send() {
        let sender = MockSender::new();
        let bot = ReplyBot::new(
            sender.clone(),
            vec![Box::new(MockStrategy::new("s1", false, "nope"))],
        );
        let msg = build_msg();
        let ctx = fake_ctx();
        bot.handle(unsafe { &*ctx }, &msg).await;
        assert_eq!(sender.send_count.load(Ordering::SeqCst), 0);
        assert_eq!(sender.identity_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn uses_send_with_identity_when_identity_returned() {
        let sender = MockSender::new();
        let bot = ReplyBot::new(
            sender.clone(),
            vec![Box::new(
                MockStrategy::new("s1", true, "persona reply").with_identity(),
            )],
        );
        let msg = build_msg();
        let ctx = fake_ctx();
        bot.handle(unsafe { &*ctx }, &msg).await;
        assert_eq!(sender.identity_count.load(Ordering::SeqCst), 1);
        assert_eq!(sender.send_count.load(Ordering::SeqCst), 0);
        assert_eq!(*sender.last_content.lock().unwrap(), "persona reply");
    }

    #[tokio::test]
    async fn condition_failure_skips_should_trigger() {
        use crate::middleware::MessageFilter;
        use std::sync::Arc;

        struct AlwaysFail;
        impl MessageFilter for AlwaysFail {
            fn check(&self, _ctx: &Context, _msg: &Message) -> bool {
                false
            }
        }

        use crate::replybot::strategy::WithCondition;
        let inner = MockStrategy::new("s1", true, "should not send");
        let conditioned = WithCondition::new(Arc::new(AlwaysFail), inner);

        let sender = MockSender::new();
        let bot = ReplyBot::new(sender.clone(), vec![Box::new(conditioned)]);
        let msg = build_msg();
        let ctx = fake_ctx();
        bot.handle(unsafe { &*ctx }, &msg).await;
        assert_eq!(sender.send_count.load(Ordering::SeqCst), 0);
    }

    // suppress unused field warnings on AtomicBool in MockStrategy
    #[allow(dead_code)]
    fn _use_atomic_bool(_: AtomicBool) {}
}
