use app::cart::service::CartService;
use app::users::service::UserService;

use db::cart_repo::PgCartRepo;
use db::user_repo::PgUserRepo;

use sqlx::PgPool;
use std::env;

#[derive(Clone)]
pub struct BotContext {
    pub db: PgPool, // 👈 فعلاً نگه می‌داریم (برای viewها)
    pub user_service: UserService<PgUserRepo>,
    pub cart_service: CartService<PgCartRepo>, // 👈 جدید
}

impl BotContext {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = PgPool::connect(&database_url).await?;

        let user_repo = PgUserRepo::new(pool.clone());
        let user_service = UserService::new(user_repo);

        let cart_repo = PgCartRepo::new(pool.clone());
        let cart_service = CartService::new(cart_repo);

        Ok(Self {
            db: pool,
            user_service,
            cart_service,
        })
    }
}
