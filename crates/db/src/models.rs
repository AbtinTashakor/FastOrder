use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct CustomerRow {
    pub id: Uuid,
    pub phone_number: String,
    pub telegram_user_id: Option<i64>,
    pub created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug)]
pub struct MenuCategoryRow {
    pub id: Uuid,
    pub title: String,
    pub position: i32,
    pub is_active: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct MenuItemRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_title: String,
    pub title: String,
    pub price: i64,
    pub position: i32,
}

#[derive(sqlx::FromRow, Debug)]
pub struct CartRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String, // 'active' | 'locked'
}

#[derive(sqlx::FromRow, Debug)]
pub struct CartItemRow {
    pub title: String,
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub price_snapshot: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct OrderRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total_price: i64,
    pub status: String, // 'pending' | 'accepted' | 'rejected'
    pub prep_time_minutes: Option<i32>,
    pub created_at: NaiveDateTime,
    pub daily_no: i32,
}

#[derive(sqlx::FromRow, Debug)]
pub struct OrderItemSnapshotRow {
    pub title_snapshot: String,
    pub price_snapshot: i64,
    pub quantity: i32,
}
