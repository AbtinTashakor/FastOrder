use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;

use crate::models::operator_state::OperatorState;

#[async_trait]
pub trait OperatorStateRepo: Clone + Send + Sync + 'static {
    async fn get(&self, operator_id: Uuid) -> Result<OperatorState>;

    async fn set_on_shift(&self, operator_id: Uuid, on_shift: bool) -> Result<()>;

    async fn set_view_list(&self, operator_id: Uuid) -> Result<()>;

    async fn set_view_order(
        &self,
        operator_id: Uuid,
        order_id: Uuid,
    ) -> Result<()>;
}
