use sqlx::PgPool;
use std::env;

#[derive(Clone)]
pub struct BotContext {
    pub db: PgPool,
}

impl BotContext {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let db = PgPool::connect(&database_url).await?;

        Ok(Self { db })
    }
}
