use std::env;

use sqlx::PgPool;

use app::{
    services::{
        users::service::UserService,
        cart::service::CartService,
        menu::service::MenuService,
        order::service::OrderService,
    },
};

use db::repos::{
    PgUserRepo,
    PgCartRepo,
    PgMenuRepo,
    PgOrderRepo,
};

#[derive(Clone)]
pub struct BotContext {
    pub user_service: UserService<PgUserRepo>,
    pub cart_service: CartService<PgCartRepo>,
    pub menu_service: MenuService<PgMenuRepo>,
    pub order_service: OrderService<PgOrderRepo>,
}

impl BotContext {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = PgPool::connect(&database_url).await?;

        /* ───────────── Users ───────────── */
        let user_repo = PgUserRepo::new(pool.clone());
        let user_service = UserService::new(user_repo);

        /* ───────────── Cart ───────────── */
        let cart_repo = PgCartRepo::new(pool.clone());
        let cart_service = CartService::new(cart_repo);

        /* ───────────── Menu ───────────── */
        let menu_repo = PgMenuRepo::new(pool.clone());
        let menu_service = MenuService::new(menu_repo);

        /* ───────────── Order ───────────── */
        let order_repo = PgOrderRepo::new(pool.clone());
        let order_service = OrderService::new(order_repo);

        Ok(Self {
            user_service,
            cart_service,
            menu_service,
            order_service,
        })
    }
}
