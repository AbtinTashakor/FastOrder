use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

/// Create PostgreSQL connection pool
pub async fn create_pool() -> Result<PgPool> {
    dotenvy::dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

/// Simple DB health check
pub async fn health_check(pool: &PgPool) -> Result<()> {
    let (value,): (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;

    println!("✅ DB health check passed (value = {})", value);
    Ok(())
}
