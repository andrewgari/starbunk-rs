#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry = starbunk::telemetry::init("ragtime")?;
    let conn_str = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set for ragtime"))?;
    ragtime::run(&conn_str).await
}
