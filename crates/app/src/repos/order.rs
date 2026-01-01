use async_trait::async_trait;
use uuid::Uuid;

use crate::models::order::Order;

/// ─────────────────────────────
/// Repository contract (policy)
/// ─────────────────────────────
///
/// Defines what the order use-cases need from persistence.
/// Infrastructure (Postgres, API, etc.) must implement this.
#[async_trait]
pub trait OrderRepo: Send + Sync {
    async fn create_from_cart(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> anyhow::Result<Order>;
}
