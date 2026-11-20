#![allow(clippy::double_must_use)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry = starbunk::telemetry::init("bluebot")?;
    bluebot::run().await
}
