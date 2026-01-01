use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;


use crate::models::{Role, User};

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>>;
    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>>;
    async fn bind_telegram(
        &self,
        user_id: Uuid,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> Result<()>;
    async fn set_phone_and_verify(&self, user_id: Uuid, phone: &str) -> Result<()>;
    async fn assign_role(&self, user_id: Uuid, role: Role) -> Result<()>;
    async fn has_role(&self, user_id: Uuid, role: Role) -> Result<bool>;
}

#[derive(Clone)]
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_user(
    id: Uuid,
    telegram_id: i64,
    telegram_username: Option<String>,
    phone: Option<String>,
    full_name: Option<String>,
    is_active: bool,
) -> User {
    User {
        id,
        telegram_id,
        telegram_username,
        phone,
        full_name,
        is_active,
    }
}

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn find_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT
                id,
                telegram_id,
                telegram_username,
                phone,
                full_name,
                is_active
            FROM users
            WHERE telegram_id = $1
              AND deleted_at IS NULL
            "#,
            telegram_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            map_user(
                r.id,
                r.telegram_id.unwrap_or(telegram_id),
                r.telegram_username,
                r.phone,
                r.full_name,
                r.is_active,
            )
        }))
    }

    async fn set_phone_and_verify(&self, user_id: Uuid, phone: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET phone = $2,
                updated_at = now()
            WHERE id = $1
              AND deleted_at IS NULL
            "#,
            user_id,
            phone
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn assign_role(&self, user_id: Uuid, role: Role) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES (
                $1,
                (SELECT id FROM roles WHERE name = $2)
            )
            ON CONFLICT DO NOTHING
            "#,
            user_id,
            role.as_str()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn has_role(&self, user_id: Uuid, role: Role) -> Result<bool> {
        let row = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM user_roles ur
                JOIN roles r ON r.id = ur.role_id
                WHERE ur.user_id = $1
                  AND r.name = $2
            ) AS "exists!"
            "#,
            user_id,
            role.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.exists)
    }

    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
        SELECT
            id,
            telegram_id,
            telegram_username,
            phone,
            full_name,
            is_active
        FROM users
        WHERE phone = $1
          AND deleted_at IS NULL
        "#,
            phone
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            map_user(
                r.id,
                r.telegram_id.unwrap_or(0),
                r.telegram_username,
                r.phone,
                r.full_name,
                r.is_active,
            )
        }))
    }
    async fn bind_telegram(
        &self,
        user_id: Uuid,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
        UPDATE users
        SET telegram_id = $2,
            telegram_username = COALESCE($3, telegram_username),
            full_name = COALESCE($4, full_name),
            updated_at = now()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
            user_id,
            telegram_id,
            telegram_username,
            full_name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
