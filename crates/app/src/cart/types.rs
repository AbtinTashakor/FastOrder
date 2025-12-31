use uuid::Uuid;

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
