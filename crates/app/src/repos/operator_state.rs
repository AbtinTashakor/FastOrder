use anyhow::Result;
use uuid::Uuid;

use crate::models::operator_state::{OperatorState, OperatorView};

pub trait OperatorStateRepo: Clone + Send + Sync + 'static {
    fn get(&self, operator_id: Uuid) -> Result<OperatorState>;

    fn set_on_shift(&self, operator_id: Uuid, on_shift: bool) -> Result<()>;

    fn set_view_list(&self, operator_id: Uuid) -> Result<()>;

    fn set_view_order(&self, operator_id: Uuid, order_id: Uuid) -> Result<()>;
}
