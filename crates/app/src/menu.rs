use uuid::Uuid;

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
}
