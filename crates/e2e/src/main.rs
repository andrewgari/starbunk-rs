use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Message, Ready};
use tokio::sync::mpsc::UnboundedSender;

#[derive(serde::Deserialize, Debug)]
struct TestSuite {
    tests: Vec<TestCase>,
}

#[derive(serde::Deserialize, Debug)]
struct TestCase {
    name: String,
    sender: String, // "human" or "bot"
    message: String,
    expect: Option<String>,
    expect_no_response: Option<bool>,
    timeout_ms: Option<u64>,
}

struct TestListener {
    msg_tx: UnboundedSender<Message>,
    channel_id: u64,
}

#[async_trait]
impl EventHandler for TestListener {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        tracing::info!("E2E Listener: Connected as {}", ready.user.name);
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.channel_id.get() == self.channel_id {
            let _ = self.msg_tx.send(msg);
        }
    }
}

fn get_default_suite_content() -> String {
    r#"{
  "tests": [
    {
      "name": "Bunkbot ping response",
      "sender": "human",
      "message": "ping bunkbot",
      "expect": "Pong from bunkbot!",
      "timeout_ms": 2500
    },
    {
      "name": "Bunkbot ignores bot pings",
      "sender": "bot",
      "message": "ping bunkbot",
      "expect_no_response": true,
      "timeout_ms": 2500
    },
    {
      "name": "Bluebot trigger on 'blue'",
      "sender": "human",
      "message": "Did somebody say blue?",
      "expect": "Did somebody say Blu?",
      "timeout_ms": 2500
    },
    {
      "name": "Bluebot ignores bot trigger",
      "sender": "bot",
      "message": "Did somebody say blue?",
      "expect_no_response": true,
      "timeout_ms": 2500
    },
    {
      "name": "Bluebot word boundary trigger failure",
      "sender": "human",
      "message": "This bluetooth speaker is great",
      "expect_no_response": true,
      "timeout_ms": 2500
    }
  ]
}"#
    .to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging — use the shared OTEL pipeline per AGENTS.md
    let _telemetry = starbunk::telemetry::init("starbunk-e2e")?;

    tracing::info!("E2E Runner: Starting test run");

    // 2. Load required E2E configuration
    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable is required"))?;
    let channel_id_val = std::env::var("E2E_CHANNEL_ID")
        .map_err(|_| anyhow::anyhow!("E2E_CHANNEL_ID environment variable is required"))?
        .parse::<u64>()
        .map_err(|e| anyhow::anyhow!("invalid E2E_CHANNEL_ID: {}", e))?;

    // 3. Start listener client to monitor Discord channel responses
    let (msg_tx, mut msg_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let listener = TestListener {
        msg_tx,
        channel_id: channel_id_val,
    };

    let intents = serenity::all::GatewayIntents::GUILD_MESSAGES
        | serenity::all::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(listener)
        .await
        .map_err(|e| anyhow::anyhow!("failed to build listener client: {}", e))?;

    let http = client.http.clone();

    tokio::spawn(async move {
        if let Err(e) = client.start().await {
            tracing::error!("Listener client exited with error: {}", e);
        }
    });

    // Wait for client to connect and initialize cache
    tracing::info!("E2E Runner: Waiting for Discord Gateway connections...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Retrieve bot account details
    let current_user = http
        .get_current_user()
        .await
        .map_err(|e| anyhow::anyhow!("failed to retrieve bot profile: {}", e))?;
    let bot_user_id = current_user.id;
    tracing::info!(
        "E2E Runner: Verified Bot account: {} (ID: {})",
        current_user.name,
        bot_user_id
    );

    // 4. Fetch or create E2E webhook
    let channel_id = serenity::all::ChannelId::new(channel_id_val);
    let webhooks = channel_id
        .webhooks(&http)
        .await
        .map_err(|e| anyhow::anyhow!("failed to retrieve channel webhooks: {}", e))?;

    let webhook = if let Some(wh) = webhooks
        .into_iter()
        .find(|w| w.name.as_deref() == Some("Starbunk E2E Webhook"))
    {
        wh
    } else {
        tracing::info!(
            "E2E Runner: Creating 'Starbunk E2E Webhook' in channel {}",
            channel_id
        );
        channel_id
            .create_webhook(
                &http,
                serenity::all::CreateWebhook::new("Starbunk E2E Webhook"),
            )
            .await
            .map_err(|e| anyhow::anyhow!("failed to create E2E webhook: {}", e))?
    };

    let webhook_url = webhook
        .url()
        .map_err(|e| anyhow::anyhow!("failed to resolve webhook URL: {}", e))?;
    // Do not log webhook_url — it contains the secret token
    tracing::info!(webhook_id = %webhook.id, "E2E Runner: Resolved webhook");

    // 5. Configure environment variables for the bots
    std::env::set_var("E2E_WEBHOOK_ID", webhook.id.to_string());
    // E2E mode must be enabled for E2eDebugHandler to activate
    std::env::set_var("E2E_MODE", "true");

    // 6. Spawn configured bots in background tasks (if E2E_START_BOTS is true)
    let e2e_start_bots = std::env::var("E2E_START_BOTS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

    if e2e_start_bots {
        if std::env::var("E2E_GUILD_ID").is_err() {
            anyhow::bail!(
                "E2E_GUILD_ID must be set when E2E_START_BOTS=true — \
                 spawned bots require a guild ID to operate in E2E mode"
            );
        }
        let bots_env =
            std::env::var("E2E_TEST_BOTS").unwrap_or_else(|_| "bunkbot,bluebot".to_string());
        let bots: Vec<&str> = bots_env.split(',').map(|s| s.trim()).collect();
        tracing::info!("E2E Runner: Spawning bots inside runner: {:?}", bots);

        for bot in bots {
            match bot {
                "bluebot" => {
                    tokio::spawn(async {
                        if let Err(e) = bluebot::run().await {
                            tracing::error!("bluebot exited with error: {}", e);
                        }
                    });
                }
                "bunkbot" => {
                    tokio::spawn(async {
                        if let Err(e) = bunkbot::run().await {
                            tracing::error!("bunkbot exited with error: {}", e);
                        }
                    });
                }
                "covabot" => {
                    tokio::spawn(async {
                        if let Err(e) = covabot::run().await {
                            tracing::error!("covabot exited with error: {}", e);
                        }
                    });
                }
                "djcova" => {
                    tokio::spawn(async {
                        if let Err(e) = djcova::run().await {
                            tracing::error!("djcova exited with error: {}", e);
                        }
                    });
                }
                "ratbot" => {
                    tokio::spawn(async {
                        if let Err(e) = ratbot::run().await {
                            tracing::error!("ratbot exited with error: {}", e);
                        }
                    });
                }
                _ => {
                    tracing::warn!("Unknown bot configured for E2E: {}", bot);
                }
            }
        }
        // Give spawned bots a moment to connect to Gateway and log in
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // 6b. Run health checks against the /health endpoints of spawned bots
        let bots_env =
            std::env::var("E2E_TEST_BOTS").unwrap_or_else(|_| "bunkbot,bluebot".to_string());
        let bots: Vec<&str> = bots_env.split(',').map(|s| s.trim()).collect();
        tracing::info!(
            "E2E Runner: Querying /health endpoints for spawned bots: {:?}",
            bots
        );

        let reqwest_client = reqwest::Client::new();
        for bot in &bots {
            let port = match *bot {
                "bluebot" => 8081,
                "bunkbot" => 8082,
                "covabot" => 8083,
                "djcova" => 8084,
                "ratbot" => 8085,
                _ => {
                    tracing::warn!("E2E Runner: Unknown bot '{}', skipping health check", bot);
                    continue;
                }
            };
            let url = format!("http://localhost:{}/health", port);
            tracing::info!("E2E Runner: Checking health of {} at {}", bot, url);
            match reqwest_client.get(&url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(body) = resp.text().await {
                            tracing::info!(
                                "E2E Runner: Health check for {} succeeded: {}",
                                bot,
                                body
                            );
                        } else {
                            tracing::warn!("E2E Runner: Health check for {} succeeded but body could not be read", bot);
                        }
                    } else {
                        anyhow::bail!(
                            "E2E Runner: Health check for {} returned failure status: {}",
                            bot,
                            resp.status()
                        );
                    }
                }
                Err(e) => {
                    anyhow::bail!(
                        "E2E Runner: Health check for {} failed to connect: {}",
                        bot,
                        e
                    );
                }
            }
        }
        tracing::info!("E2E Runner: All spawned bots' health checks passed.");
    }

    // 7. Parse test suite configuration
    let suite_path = std::env::var("E2E_SUITE_PATH")
        .unwrap_or_else(|_| format!("{}/suites/bunkbot_bluebot.json", env!("CARGO_MANIFEST_DIR")));
    let suite_content = std::fs::read_to_string(&suite_path).unwrap_or_else(|_| {
        tracing::info!("E2E Runner: E2E_SUITE_PATH not found or readable. Falling back to default built-in test suite.");
        get_default_suite_content()
    });

    let suite: TestSuite = serde_json::from_str(&suite_content)
        .map_err(|e| anyhow::anyhow!("failed to parse test suite JSON: {}", e))?;

    // 8. Execute E2E test suite
    let reqwest_client = reqwest::Client::new();
    let mut passed = 0;
    let mut failed = 0;
    let total = suite.tests.len();

    tracing::info!(
        "E2E Runner: Executing {} tests against channel {}",
        total,
        channel_id_val
    );

    for (i, test) in suite.tests.iter().enumerate() {
        tracing::info!("[E2E Test {}/{}] Running: {}", i + 1, total, test.name);

        // Flush any lingering message events
        while msg_rx.try_recv().is_ok() {}

        // Format E2E payload prefix to flag simulated bot/human authors
        let payload_content = match test.sender.as_str() {
            "bot" => format!("[E2E_BOT] {}", test.message),
            _ => format!("[E2E_HUMAN] {}", test.message),
        };

        let payload = serde_json::json!({
            "content": payload_content,
            "username": if test.sender == "bot" { "E2E_Simulated_Bot" } else { "E2E_Simulated_Human" }
        });

        // Fire the test message via Discord Webhook; treat non-2xx as errors
        let send_res = reqwest_client
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await
            .and_then(|r| r.error_for_status());

        if let Err(e) = send_res {
            tracing::error!("FAILED: Webhook request failed: {}", e);
            failed += 1;
            continue;
        }

        // Wait for bot response with configured timeout
        let timeout_ms = test.timeout_ms.unwrap_or(2500);
        let timeout_dur = tokio::time::Duration::from_millis(timeout_ms);
        let start = tokio::time::Instant::now();

        let mut got_expected = false;
        let mut got_unexpected = false;
        let mut last_response = String::new();

        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout_dur {
                break;
            }

            let remaining = timeout_dur - elapsed;
            match tokio::time::timeout(remaining, msg_rx.recv()).await {
                Ok(Some(msg)) => {
                    // Check if the message is from our bot's user account (bot under test)
                    if msg.author.id == bot_user_id {
                        last_response = msg.content.clone();
                        if let Some(ref expected) = test.expect {
                            if msg.content.contains(expected) {
                                got_expected = true;
                                break;
                            } else {
                                got_unexpected = true;
                            }
                        } else if test.expect_no_response.unwrap_or(false) {
                            got_unexpected = true;
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => break, // Timeout elapsed
            }
        }

        let success = if test.expect_no_response.unwrap_or(false) {
            !got_unexpected
        } else {
            got_expected && !got_unexpected
        };

        if success {
            tracing::info!("PASSED: {}", test.name);
            passed += 1;
        } else {
            if test.expect_no_response.unwrap_or(false) {
                tracing::error!(
                    "FAILED: {}. Expected no reply, but bot sent a message: {:?}",
                    test.name,
                    last_response
                );
            } else {
                tracing::error!(
                    "FAILED: {}. Expected matching: {:?}. Got response: {:?}",
                    test.name,
                    test.expect,
                    last_response
                );
            }
            failed += 1;
        }
    }

    let success_rate = (passed as f64 / total as f64) * 100.0;
    tracing::info!("========================================");
    tracing::info!("E2E RUN SUMMARY:");
    tracing::info!("Total:   {}", total);
    tracing::info!("Passed:  {} ({:.1}%)", passed, success_rate);
    tracing::info!("Failed:  {}", failed);
    tracing::info!("========================================");

    if failed > 0 {
        tracing::error!("E2E Suite failed: {} tests failed.", failed);
        std::process::exit(1);
    } else {
        tracing::info!("E2E Suite completed successfully.");
        std::process::exit(0);
    }
}
