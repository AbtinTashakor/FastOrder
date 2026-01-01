use uuid::Uuid;

/* ================= Domain ================= */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Operator,
    Customer,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartStatus {
    Active,
    Confirming,
    Locked,
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

#[derive(Debug, Clone)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: CartStatus,
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

#[derive(Debug)]
pub enum AuthResult {
    AlreadyVerified { user_id: Uuid },
    Verified { user_id: Uuid },
    PhoneNotFound,
    InvalidPhone,
}
