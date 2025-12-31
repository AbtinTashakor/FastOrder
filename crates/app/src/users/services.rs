use async_trait::async_trait;
use uuid::Uuid;

use super::types::{Role, User};
use crate::users::auth_error::AuthError;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_telegram_id(&self, telegram_id: i64) -> anyhow::Result<Option<User>>;

    async fn find_by_phone(&self, phone: &str) -> anyhow::Result<Option<User>>;

    async fn bind_telegram(
        &self,
        user_id: Uuid,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn set_phone_and_verify(&self, user_id: Uuid, phone: &str) -> anyhow::Result<()>;

    async fn assign_role(&self, user_id: Uuid, role: Role) -> anyhow::Result<()>;

    async fn has_role(&self, user_id: Uuid, role: Role) -> anyhow::Result<bool>;
}

#[derive(Clone)]
pub struct UserService<R: UserRepo + Clone> {
    repo: R,
}

impl<R: UserRepo + Clone> UserService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }


    /// Get verified customer by telegram id
    /// Used by bot before any cart/order action
    pub async fn get_verified_user_by_telegram(
        &self,
        telegram_id: i64,
    ) -> Result<User, AuthError> {
        let user = match self.repo.find_by_telegram_id(telegram_id).await {
            Ok(Some(u)) => u,
            Ok(None) => return Err(AuthError::NotVerified),
            Err(_) => return Err(AuthError::Internal),
        };

        let has_role = self
            .repo
            .has_role(user.id, Role::Customer)
            .await
            .map_err(|_| AuthError::Internal)?;

        if !has_role {
            return Err(AuthError::NotVerified);
        }

        Ok(user)
    }


    /// Check if telegram user is already a verified customer
    /// ❌ does NOT create user
    /// ❌ does NOT modify database
    pub async fn is_verified_customer(&self, telegram_id: i64) -> Result<bool, AuthError> {
        let user = match self.repo.find_by_telegram_id(telegram_id).await {
            Ok(Some(u)) => u,
            Ok(None) => return Ok(false),
            Err(_) => return Err(AuthError::Internal),
        };

        self.repo
            .has_role(user.id, Role::Customer)
            .await
            .map_err(|_| AuthError::Internal)
    }

    /// Verify contact and activate existing customer
    /// ❌ does NOT create user
    pub async fn verify_contact_as_customer(
        &self,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
        phone: &str,
    ) -> Result<User, AuthError> {
        // 1️⃣ user must already exist by phone
        let user = match self.repo.find_by_phone(phone).await {
            Ok(Some(u)) => u,
            Ok(None) => return Err(AuthError::PhoneNotRegistered),
            Err(_) => return Err(AuthError::Internal),
        };

        // 2️⃣ bind telegram
        self.repo
            .bind_telegram(user.id, telegram_id, telegram_username, full_name)
            .await
            .map_err(|_| AuthError::Internal)?;

        // 3️⃣ verify + assign role
        self.repo
            .set_phone_and_verify(user.id, phone)
            .await
            .map_err(|_| AuthError::Internal)?;

        self.repo
            .assign_role(user.id, Role::Customer)
            .await
            .map_err(|_| AuthError::Internal)?;

        // 4️⃣ return fresh state
        self.repo
            .find_by_telegram_id(telegram_id)
            .await
            .map_err(|_| AuthError::Internal)?
            .ok_or(AuthError::Internal)
    }

    /// Generic role check (used later in callbacks / admin)
    pub async fn has_role(&self, user_id: Uuid, role: Role) -> Result<bool, AuthError> {
        self.repo
            .has_role(user_id, role)
            .await
            .map_err(|_| AuthError::Internal)
    }
}
