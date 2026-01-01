use uuid::Uuid;

/* ================= Projections ================= */

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
