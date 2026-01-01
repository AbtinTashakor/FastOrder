use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;


use app::{
    models::user::{Role, User},
    repos::user::UserRepo,
};


use crate::models::UserRow;

#[derive(Clone)]
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /* ───────────────────── Mapping ───────────────────── */

    fn map_user(row: UserRow) -> User {
        User {
            id: row.id,
            telegram_id: row.telegram_id.unwrap_or(0),
            telegram_username: row.telegram_username,
            phone: row.phone,
            full_name: row.full_name,
            is_active: row.is_active,
        }
    }

    /* ───────────── Queries (inner) ───────────── */

    async fn find_by_telegram_id_inner(
        &self,
        telegram_id: i64,
    ) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
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
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_user))
    }

    async fn find_by_phone_inner(
        &self,
        phone: &str,
    ) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
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
        )
        .bind(phone)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_user))
    }

    /* ───────────── Commands (inner) ───────────── */

    async fn bind_telegram_inner(
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
                updated_at = NOW()
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

    async fn set_phone_and_verify_inner(
        &self,
        user_id: Uuid,
        phone: &str,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET phone = $2,
                is_active = true,
                updated_at = NOW()
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

    async fn assign_role_inner(
        &self,
        user_id: Uuid,
        role: Role,
    ) -> Result<()> {
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

    async fn has_role_inner(
        &self,
        user_id: Uuid,
        role: Role,
    ) -> Result<bool> {
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
}

/* ───────────────────────────────────────────────
   Trait implementation (Contract fulfillment)
   ─────────────────────────────────────────────── */

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn find_by_telegram_id(
        &self,
        telegram_id: i64,
    ) -> Result<Option<User>> {
        self.find_by_telegram_id_inner(telegram_id).await
    }

    async fn find_by_phone(
        &self,
        phone: &str,
    ) -> Result<Option<User>> {
        self.find_by_phone_inner(phone).await
    }

    async fn bind_telegram(
        &self,
        user_id: Uuid,
        telegram_id: i64,
        telegram_username: Option<&str>,
        full_name: Option<&str>,
    ) -> Result<()> {
        self.bind_telegram_inner(
            user_id,
            telegram_id,
            telegram_username,
            full_name,
        )
        .await
    }

    async fn set_phone_and_verify(
        &self,
        user_id: Uuid,
        phone: &str,
    ) -> Result<()> {
        self.set_phone_and_verify_inner(user_id, phone).await
    }

    async fn assign_role(
        &self,
        user_id: Uuid,
        role: Role,
    ) -> Result<()> {
        self.assign_role_inner(user_id, role).await
    }

    async fn has_role(
        &self,
        user_id: Uuid,
        role: Role,
    ) -> Result<bool> {
        self.has_role_inner(user_id, role).await
    }
}
