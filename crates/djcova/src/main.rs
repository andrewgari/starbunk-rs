#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry = starbunk_shared::telemetry::init("djcova");
    djcova::run().await
}
