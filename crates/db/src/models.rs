use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(sqlx::FromRow)]
pub struct CustomerRow {
    pub id: Uuid,
    pub phone_number: String,
    pub telegram_user_id: Option<i64>,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct MenuItemRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    pub price: i64,
    pub is_available: bool,
}

#[derive(sqlx::FromRow)]
pub struct CartRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
}

#[derive(sqlx::FromRow)]
pub struct CartItemRow {
    pub menu_item_id: Uuid,
    pub quantity: i32,
}
