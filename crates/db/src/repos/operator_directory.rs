use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use app::repos::operator_directory::OperatorDirectory;

#[derive(Clone)]
pub struct PgOperatorDirectory {
    pool: PgPool,
}

impl PgOperatorDirectory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OperatorDirectory for PgOperatorDirectory {

    async fn list_available(&self) -> Result<Vec<Uuid>> {
        let rows = sqlx::query!(
            r#"
            SELECT u.id
            FROM users u
            JOIN user_roles ur ON ur.user_id = u.id
            JOIN roles r ON r.id = ur.role_id
            JOIN operator_state os ON os.operator_id = u.id
            WHERE r.name = 'operator'
              AND os.is_on_shift = true
            ORDER BY u.id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}
