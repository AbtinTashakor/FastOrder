use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::{Role, User};

/// ─────────────────────────────
/// Repository contract (policy)
/// ─────────────────────────────
///
/// Defines what user use-cases need from persistence.
/// Infrastructure (Postgres, etc.) must implement this.
#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_telegram_id(
        &self,
        telegram_id: i64,
    ) -> anyhow::Result<Option<User>>;

    async fn find_by_phone(
        &self,
        phone: &str,
    ) -> anyhow::Result<Option<User>>;

    async fn bind_telegram(
        &self,
        user_id: Uuid,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn set_phone_and_verify(
        &self,
        user_id: Uuid,
        phone: &str,
    ) -> anyhow::Result<()>;

    async fn assign_role(
        &self,
        user_id: Uuid,
        role: Role,
    ) -> anyhow::Result<()>;

    async fn has_role(
        &self,
        user_id: Uuid,
        role: Role,
    ) -> anyhow::Result<bool>;
}
