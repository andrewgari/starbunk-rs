#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry = starbunk::telemetry::init("sheeshbot")?;
    sheeshbot::run().await
}
