use anyhow::Result;
use uuid::Uuid;
use async_trait::async_trait;

#[async_trait]
pub trait SystemStateRepo: Send + Sync {
    async fn get_last_operator(&self) -> anyhow::Result<Option<Uuid>>;
    async fn set_last_operator(&self, operator_id: Uuid) -> anyhow::Result<()>;
}
