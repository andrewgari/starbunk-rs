#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry = starbunk::telemetry::init("ratbot")?;
    ratbot::run().await
}
