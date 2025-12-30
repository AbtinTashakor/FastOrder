use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub telegram_id: i64,
    pub telegram_username: Option<String>,
    pub phone: Option<String>,
    pub full_name: Option<String>,
    pub is_verified: bool,
    pub is_active: bool,
}

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
