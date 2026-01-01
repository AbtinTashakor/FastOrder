use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgMenuRepo {
    pool: PgPool,
}

impl PgMenuRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns all active menu items ordered by category position then item position
    pub async fn list_available_items(&self) -> Result<Vec<MenuItemRow>, sqlx::Error> {
        sqlx::query_as!(
            MenuItemRow,
            r#"
            SELECT
                mi.id,
                mi.title,
                mi.price,
                mi.category_id,
                mc.title  AS category_title,
                mi.position
            FROM menu_items mi
            JOIN menu_categories mc
              ON mc.id = mi.category_id
             AND mc.is_active = true
            ORDER BY mc.position, mi.position
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
}

/* ---------- DB ROW ---------- */

#[derive(Debug)]
pub struct MenuItemRow {
    pub id: Uuid,
    pub title: String,
    pub price: i64,
    pub category_id: Uuid,
    pub category_title: String,
    pub position: i32,
}
