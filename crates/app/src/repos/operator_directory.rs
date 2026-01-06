use anyhow::Result;
use uuid::Uuid;

#[async_trait]
pub trait OperatorDirectory: Send + Sync {
    async fn list_available(&self) -> anyhow::Result<Vec<Uuid>>;
}
