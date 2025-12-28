use db::{create_pool, health_check};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = create_pool().await?;
    health_check(&pool).await?;
    Ok(())
}
