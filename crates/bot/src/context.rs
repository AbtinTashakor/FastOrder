use sqlx::PgPool;

#[derive(Clone)]
pub struct BotContext {
    pub db: PgPool,
}
