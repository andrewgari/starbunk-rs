use poise::serenity_prelude as serenity;

struct Data {} // User data, passed to all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "Pong!"
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // Load the Discord bot token from the environment.
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment variable `DISCORD_TOKEN`");

    // Set gateway intents.
    // Note: If you want to use prefix commands, you must enable the `Message Content` intent
    // both in your Discord Developer Portal and by adding it to your intents here.
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Registering commands globally...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Bot is ready!");
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
