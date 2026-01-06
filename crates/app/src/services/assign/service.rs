use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::{
    models::order::OrderStatus,
    repos::{
        order::OrderRepo,
        operator_directory::OperatorDirectory,
        system_state::SystemStateRepo,
    },
    services::operator_state::service::OperatorStateService,
};

#[derive(Clone)]
pub struct AssignService<
    O: OrderRepo,
    D: OperatorDirectory,
    S: SystemStateRepo,
    OS: Clone + Send + Sync,
> {
    orders: O,
    directory: D,
    system_state: S,
    operator_state: OS, // OperatorStateService
}

impl<
        O: OrderRepo,
        D: OperatorDirectory,
        S: SystemStateRepo,
        OS: Clone + Send + Sync,
    > AssignService<O, D, S, OS>
{
    pub fn new(
        orders: O,
        directory: D,
        system_state: S,
        operator_state: OS,
    ) -> Self {
        Self {
            orders,
            directory,
            system_state,
            operator_state,
        }
    }

    /* ───────────────── Assign ───────────────── */

    pub async fn assign_order(&self, order_id: Uuid) -> Result<()> {
        let operators = self.directory.list_available().await?;

        if operators.is_empty() {
            self.orders
                .set_status(order_id, OrderStatus::WaitingForOperator)
                .await?;
            self.orders.assign_operator(order_id, None, None).await?;
            return Ok(());
        }

        let chosen = self.pick_next_operator(&operators).await?;
        let now = Utc::now();

        self.orders
            .assign_operator(order_id, Some(chosen), Some(now))
            .await?;
        self.orders
            .set_status(order_id, OrderStatus::AssignedUnseen)
            .await?;

        self.system_state.set_last_operator(chosen).await?;
        Ok(())
    }

    async fn pick_next_operator(&self, operators: &[Uuid]) -> Result<Uuid> {
        let last = self.system_state.get_last_operator().await?;

        if let Some(last_id) = last {
            if let Some(idx) = operators.iter().position(|&id| id == last_id) {
                return Ok(operators[(idx + 1) % operators.len()]);
            }
        }

        Ok(operators[0])
    }

    /* ─────────────── State transitions ─────────────── */

    pub async fn mark_seen(&self, order_id: Uuid) -> Result<()> {
        self.orders.mark_seen(order_id, Utc::now()).await?;
        self.orders
            .set_status(order_id, OrderStatus::AssignedInReview)
            .await?;
        Ok(())
    }

    /* ───────────────── Timeouts ───────────────── */

    pub async fn handle_seen_timeout(
        &self,
        order_id: Uuid,
        assigned_at: DateTime<Utc>,
        limit: Duration,
    ) -> Result<()> {
        if Utc::now() - assigned_at < limit {
            return Ok(());
        }

        // اپراتور اصلاً سفارش رو ندیده
        self.assign_order(order_id).await
    }

    pub async fn handle_decision_timeout(
        &self,
        order_id: Uuid,
        seen_at: DateTime<Utc>,
        limit: Duration,
        operator_id: Uuid,
    ) -> Result<()> {
        if Utc::now() - seen_at < limit {
            return Ok(());
        }

        // اپراتور دید ولی تصمیم نگرفت → شیفت قطع
        self.operator_state.end_shift(operator_id).await?;

        self.assign_order(order_id).await
    }
}
