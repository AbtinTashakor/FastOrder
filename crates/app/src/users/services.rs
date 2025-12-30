use async_trait::async_trait;
use uuid::Uuid;

use super::types::{Role, User};

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_telegram_id(&self, telegram_id: i64) -> anyhow::Result<Option<User>>;
    async fn create_user(
        &self,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> anyhow::Result<User>;

    async fn set_phone_and_verify(&self, user_id: Uuid, phone: &str) -> anyhow::Result<()>;
    async fn assign_role(&self, user_id: Uuid, role: Role) -> anyhow::Result<()>;
    async fn has_role(&self, user_id: Uuid, role: Role) -> anyhow::Result<bool>;
}

pub struct UserService<R: UserRepo> {
    repo: R,
}

impl<R: UserRepo> UserService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    /// /start entry: user must exist
    pub async fn ensure_user(
        &self,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> anyhow::Result<User> {
        if let Some(u) = self.repo.find_by_telegram_id(telegram_id).await? {
            return Ok(u);
        }
        self.repo.create_user(telegram_id, telegram_username, full_name).await
    }

    /// Contact verification flow
    pub async fn verify_contact_as_customer(
        &self,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
        phone: &str,
    ) -> anyhow::Result<User> {
        let user = self.ensure_user(telegram_id, telegram_username, full_name).await?;

        self.repo.set_phone_and_verify(user.id, phone).await?;
        self.repo.assign_role(user.id, Role::Customer).await?;

        // return fresh state (optional: refetch)
        let user = self.repo
            .find_by_telegram_id(telegram_id)
            .await?
            .expect("user must exist after ensure_user");
        Ok(user)
    }
}
