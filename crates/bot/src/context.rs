use sqlx::PgPool;
use std::env;

use app::users::services::UserService;
use db::user_repo::PgUserRepo;

#[derive(Clone)]
pub struct BotContext {
    pub db: PgPool, // 👈 فعلاً نگه می‌داریم
    pub user_service: UserService<PgUserRepo>, // 👈 جدید
}

impl BotContext {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let pool = PgPool::connect(&database_url).await?;

        // repo + service برای auth
        let user_repo = PgUserRepo::new(pool.clone());
        let user_service = UserService::new(user_repo);

        Ok(Self {
            db: pool,
            user_service,
        })
    }
}
