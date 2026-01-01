use std::env;

use sqlx::PgPool;

use app::{
    cart::service::CartService,
    menu::service::MenuService,
    order::service::OrderService,
    users::service::UserService,
};

use db::{
    cart_repo::PgCartRepo, menu_repo::PgMenuRepo, order_repo, user_repo::PgUserRepo
};

#[derive(Clone)]
pub struct BotContext {
   
    pub user_service: UserService<PgUserRepo>,
    pub cart_service: CartService<PgCartRepo>,
    pub menu_service: MenuService,
    pub order_service: OrderService,
}

impl BotContext {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = PgPool::connect(&database_url).await?;

        // ---------- users ----------
        let user_repo = PgUserRepo::new(pool.clone());
        let user_service = UserService::new(user_repo);

        // ---------- cart ----------
        let cart_repo = PgCartRepo::new(pool.clone());
        let cart_service = CartService::new(cart_repo);

        // ---------- menu ----------
        let menu_repo = PgMenuRepo::new(pool.clone());
        let menu_service = MenuService::new(menu_repo);

        // ---------- order ----------
        let order_repo = order_repo::PgOrderRepo::new(pool.clone());
        let order_service = OrderService::new(order_repo);

        Ok(Self {
            user_service,
            cart_service,
            menu_service,
            order_service,
        })
    }
}
