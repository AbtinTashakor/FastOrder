use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub async fn create_pool() -> Result<PgPool> {
    dotenvy::dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    Ok(pool)
}
