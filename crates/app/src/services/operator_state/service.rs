use anyhow::Result;
use uuid::Uuid;

use crate::{
    models::operator_state::{OperatorState, OperatorView},
    repos::operator_state::OperatorStateRepo,
};

#[derive(Clone)]
pub struct OperatorStateService<R: OperatorStateRepo> {
    repo: R,
}

impl<R: OperatorStateRepo> OperatorStateService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /* ───────────── Query ───────────── */

    pub async fn get_state(&self, operator_id: Uuid) -> Result<OperatorState> {
        self.repo.get(operator_id).await
    }

    pub async fn is_on_shift(&self, operator_id: Uuid) -> Result<bool> {
        Ok(self.repo.get(operator_id).await?.is_on_shift)
    }

    pub async fn is_viewing_order(&self, operator_id: Uuid) -> Result<bool> {
        Ok(
            self.repo.get(operator_id).await?.current_view
                == OperatorView::Order
        )
    }

    /* ───────────── Commands ───────────── */

    pub async fn start_shift(&self, operator_id: Uuid) -> Result<()> {
        self.repo.set_on_shift(operator_id, true).await
    }

    pub async fn end_shift(&self, operator_id: Uuid) -> Result<()> {
        self.repo.set_on_shift(operator_id, false).await
    }

    pub async fn enter_list_view(&self, operator_id: Uuid) -> Result<()> {
        self.repo.set_view_list(operator_id).await
    }

    pub async fn enter_order_view(
        &self,
        operator_id: Uuid,
        order_id: Uuid,
    ) -> Result<()> {
        self.repo
            .set_view_order(operator_id, order_id)
            .await
    }
}


use crate::services::operator_state::assign_port::OperatorShiftControl;

#[async_trait::async_trait]
impl<R: crate::repos::operator_state::OperatorStateRepo> OperatorShiftControl
    for OperatorStateService<R>
{
    async fn end_shift(&self, operator_id: Uuid) -> Result<()> {
        OperatorStateService::end_shift(self, operator_id).await
    }
}
