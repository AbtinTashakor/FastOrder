use chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;

/* ================= DB Rows ================= */

#[derive(sqlx::FromRow, Debug)]
pub struct UserRow {
    pub id: Uuid,
    pub telegram_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub phone: Option<String>,
    pub full_name: Option<String>,
    pub is_active: bool,
}

/* ================= Menu ================= */

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

/* ================= Cart ================= */

#[derive(sqlx::FromRow, Debug)]
pub struct CartRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct CartItemRow {
    pub title: String,
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub price_snapshot: i64,
}

/* ================= Orders ================= */

#[derive(sqlx::FromRow, Debug)]
pub struct OrderRow {
    pub id: Uuid,
    pub user_id: Uuid,

    pub order_day: NaiveDate,
    pub daily_no: i32,
    pub order_code: String,
    pub total_price: i64,

    /* status & operator dispatch */
    pub status: String,
    pub operator_id: Option<Uuid>,
    pub assigned_at: Option<NaiveDateTime>,
    pub seen_at: Option<NaiveDateTime>,

    /* operator decision */
    pub prep_time_minutes: Option<i32>,
    pub rejection_reason: Option<String>,
    pub decided_at: Option<NaiveDateTime>,

    /* retry / audit */
    pub retry_count: i32,

    pub created_at: NaiveDateTime,
}

/* ================= Order Items ================= */

#[derive(sqlx::FromRow, Debug)]
pub struct OrderItemSnapshotRow {
    pub title_snapshot: String,
    pub price_snapshot: i64,
    pub quantity: i32,
}
