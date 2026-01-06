use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use app::{
    models::operator_state::{OperatorState, OperatorView},
    repos::operator_state::OperatorStateRepo,
};

#[derive(Clone)]
pub struct PgOperatorStateRepo {
    pool: PgPool,
}

impl PgOperatorStateRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OperatorStateRepo for PgOperatorStateRepo {

    async fn get(&self, operator_id: Uuid) -> Result<OperatorState> {
        let row = sqlx::query!(
            r#"
            SELECT operator_id, is_on_shift, current_view, current_order_id
            FROM operator_state
            WHERE operator_id = $1
            "#,
            operator_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or_else(|| anyhow!("operator_state not found"))?;

        Ok(OperatorState {
            operator_id: row.operator_id,
            is_on_shift: row.is_on_shift,
            current_view: OperatorView::from_str(&row.current_view),
            current_order_id: row.current_order_id,
        })
    }

    async fn set_on_shift(&self, operator_id: Uuid, on_shift: bool) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE operator_state
            SET is_on_shift = $2,
                updated_at = NOW()
            WHERE operator_id = $1
            "#,
            operator_id,
            on_shift
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_view_list(&self, operator_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE operator_state
            SET current_view = 'LIST',
                current_order_id = NULL,
                updated_at = NOW()
            WHERE operator_id = $1
            "#,
            operator_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_view_order(
        &self,
        operator_id: Uuid,
        order_id: Uuid,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE operator_state
            SET current_view = 'ORDER',
                current_order_id = $2,
                updated_at = NOW()
            WHERE operator_id = $1
            "#,
            operator_id,
            order_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
