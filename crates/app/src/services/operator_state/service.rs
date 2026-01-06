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

    pub fn get_state(&self, operator_id: Uuid) -> Result<OperatorState> {
        self.repo.get(operator_id)
    }

    pub fn is_on_shift(&self, operator_id: Uuid) -> Result<bool> {
        Ok(self.repo.get(operator_id)?.is_on_shift)
    }

    pub fn is_viewing_order(&self, operator_id: Uuid) -> Result<bool> {
        Ok(self.repo.get(operator_id)?.current_view == OperatorView::Order)
    }

    /* ───────────── Commands ───────────── */

    pub fn start_shift(&self, operator_id: Uuid) -> Result<()> {
        self.repo.set_on_shift(operator_id, true)
    }

    pub fn end_shift(&self, operator_id: Uuid) -> Result<()> {
        self.repo.set_on_shift(operator_id, false)
    }

    pub fn enter_list_view(&self, operator_id: Uuid) -> Result<()> {
        self.repo.set_view_list(operator_id)
    }

    pub fn enter_order_view(
        &self,
        operator_id: Uuid,
        order_id: Uuid,
    ) -> Result<()> {
        self.repo.set_view_order(operator_id, order_id)
    }
}
