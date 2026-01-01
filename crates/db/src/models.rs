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
    pub order_code: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct OrderItemSnapshotRow {
    pub title_snapshot: String,
    pub price_snapshot: i64,
    pub quantity: i32,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Operator,
    Customer,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub telegram_id: i64,
    pub telegram_username: Option<String>,
    pub phone: Option<String>,
    pub full_name: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartStatus {
    Active,
    Confirming,
    Locked
}

#[derive(Debug, Clone)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: CartStatus,
}

#[derive(Debug)]
pub struct CartItemView {
    pub menu_item_id: Uuid,
    pub title: String,
    pub quantity: i32,
    pub price_snapshot: i64,
}

#[derive(Debug)]
pub struct CartView {
    pub items: Vec<CartItemView>,
    pub total_price: i64,
}


impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Customer => "customer",
        }
    }
}


#[derive(Debug)]
pub enum AuthResult {
    AlreadyVerified {
        user_id: Uuid,
    },
    Verified {
        user_id: Uuid,
    },
    PhoneNotFound,
    InvalidPhone,
}


#[derive(Debug, Clone)]
pub struct MenuCategory {
    pub id: Uuid,
    pub title: String,
    pub position: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    pub price: i64,
    pub is_available: bool,
    pub category_title: String,
    pub position: i32,
}
