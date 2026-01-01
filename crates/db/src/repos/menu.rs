use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::MenuItemRow;

// app domain + policy
use app::{
    models::menu::MenuItem,
    repos::menu::MenuRepo,
};

#[derive(Clone)]
pub struct PgMenuRepo {
    pool: PgPool,
}

impl PgMenuRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /* ───────────────────── Internal query ───────────────────── */

    async fn list_available_items_inner(&self) -> Result<Vec<MenuItem>> {
        let rows: Vec<MenuItemRow> = sqlx::query_as!(
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
        .await?;

        Ok(rows.into_iter().map(map_menu_item).collect())
    }
}

/* ───────────────────────────────────────────────
   Trait implementation (Contract fulfillment)
   ─────────────────────────────────────────────── */

#[async_trait]
impl MenuRepo for PgMenuRepo {
    async fn list_available_items(&self) -> Result<Vec<MenuItem>> {
        self.list_available_items_inner().await
    }
}

/* ───────────────────── Mapping ───────────────────── */

fn map_menu_item(row: MenuItemRow) -> MenuItem {
    MenuItem {
        id: row.id,
        category_id: row.category_id,
        title: row.title,
        price: row.price,
        is_available: true, // چون فقط active ها query شدن
        category_title: row.category_title,
        position: row.position,
    }
}
