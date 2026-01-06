use anyhow::Result;
use uuid::Uuid;

/// Minimal interface needed by AssignService
#[async_trait::async_trait]
pub trait OperatorShiftControl: Send + Sync {
    async fn end_shift(&self, operator_id: Uuid) -> Result<()>;
}
