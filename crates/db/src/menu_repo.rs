use anyhow::Result;
use sqlx::PgPool;

use crate::models::MenuItemRow;

pub async fn list_available_items(pool: &PgPool) -> Result<Vec<MenuItemRow>> {
    let rows = sqlx::query_as::<_, MenuItemRow>(
        r#"
        SELECT * FROM menu_items
        WHERE is_available = TRUE
        ORDER BY position ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
