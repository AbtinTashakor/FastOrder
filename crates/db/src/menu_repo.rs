use anyhow::Result;
use sqlx::PgPool;

use crate::models::MenuItemRow;

pub async fn list_available_items(pool: &PgPool) -> Result<Vec<MenuItemRow>> {
    let rows = sqlx::query_as::<_, MenuItemRow>(
        r#"
        SELECT
            mi.id,
            mi.category_id,
            mc.title AS category_title,
            mi.title,
            mi.price,
            mi.position
        FROM menu_items mi
        JOIN menu_categories mc ON mc.id = mi.category_id
        WHERE mi.is_available = TRUE
          AND mc.is_active = TRUE
        ORDER BY
            mc.position ASC,
            mi.position ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
