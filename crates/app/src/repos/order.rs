use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::order::{Order, OrderStatus};

/// ─────────────────────────────
/// Repository contract (policy)
/// ─────────────────────────────
///
/// Defines what the order use-cases need from persistence.
/// Infrastructure (Postgres, API, etc.) must implement this.
#[async_trait]
pub trait OrderRepo: Send + Sync {

    /* ───────────── Creation ───────────── */

    async fn create_from_cart(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> anyhow::Result<Order>;

    /* ───────────── Queries ───────────── */

    async fn find_by_id(
        &self,
        order_id: Uuid,
    ) -> anyhow::Result<Order>;

    /* ───────────── State mutations ───────────── */

    async fn set_status(
        &self,
        order_id: Uuid,
        status: OrderStatus,
    ) -> anyhow::Result<()>;

    async fn assign_operator(
        &self,
        order_id: Uuid,
        operator_id: Option<Uuid>,
        assigned_at: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()>;

    async fn mark_seen(
        &self,
        order_id: Uuid,
        seen_at: DateTime<Utc>,
    ) -> anyhow::Result<()>;
}
