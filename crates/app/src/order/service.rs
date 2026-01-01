use uuid::Uuid;

use db::{
    order_repo::PgOrderRepo,
    models::OrderRow,
};

use crate::order::error::OrderError;

#[derive(Clone)]
pub struct OrderService {
    repo: PgOrderRepo,
}

impl OrderService {
    pub fn new(repo: PgOrderRepo) -> Self {
        Self { repo }
    }

    /// Create a new order from a confirming cart
    /// This is a transactional, idempotent use-case
    pub async fn create_from_cart(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> Result<OrderRow, OrderError> {
        self.repo
            .create_order_from_cart(user_id, cart_id)
            .await
            .map_err(|e| {
                let msg = e.to_string();

                if msg.contains("not confirming") {
                    OrderError::InvalidCart
                } else if msg.contains("cart is empty") {
                    OrderError::EmptyCart
                } else {
                    OrderError::Database
                }
            })
    }
}
