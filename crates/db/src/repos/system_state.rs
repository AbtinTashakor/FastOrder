use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use app::repos::system_state::SystemStateRepo;

#[derive(Clone)]
pub struct PgSystemStateRepo {
    pool: PgPool,
}

impl PgSystemStateRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SystemStateRepo for PgSystemStateRepo {

    async fn get_last_operator(&self) -> Result<Option<Uuid>> {
        let row = sqlx::query!(
            r#"
            SELECT value
            FROM system_state
            WHERE key = 'last_operator_id'
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.value.parse::<Uuid>().ok()))
    }

    async fn set_last_operator(&self, operator_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO system_state (key, value)
            VALUES ('last_operator_id', $1)
            ON CONFLICT (key)
            DO UPDATE SET value = $1
            "#,
            operator_id.to_string()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
