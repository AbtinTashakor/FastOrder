use uuid::Uuid;

use crate::models::order::Order;
use crate::services::order::error::OrderError;
use crate::repos::order::OrderRepo;

/// ─────────────────────────────
/// Order service (use-cases)
/// ─────────────────────────────

#[derive(Clone)]
pub struct OrderService<R: OrderRepo + Clone> {
    repo: R,
}

impl<R: OrderRepo + Clone> OrderService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Create a new order from a confirming cart
    /// Transactional + idempotent
    pub async fn create_from_cart(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> Result<Order, OrderError> {
        self.repo
            .create_from_cart(user_id, cart_id)
            .await
            .map_err(map_order_error)
    }
}

/* ─────────────────────────────
   Error mapping (infra → domain)
   ───────────────────────────── */

fn map_order_error(err: anyhow::Error) -> OrderError {
    let msg = err.to_string();

    if msg.contains("not confirming") {
        OrderError::InvalidCart
    } else if msg.contains("cart is empty") {
        OrderError::EmptyCart
    } else {
        OrderError::Database
    }
}
