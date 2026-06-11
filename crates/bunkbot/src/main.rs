#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry = starbunk::telemetry::init("bunkbot")?;
    bunkbot::run().await
}
