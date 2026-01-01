use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{Cart, CartItemRow, CartItemView, CartRow, CartStatus, CartView};

#[async_trait]
pub trait CartRepo: Send + Sync {
    async fn find_active_cart(&self, user_id: Uuid) -> anyhow::Result<Option<Cart>>;
    async fn create_active_cart(&self, user_id: Uuid) -> anyhow::Result<Cart>;
    async fn find_confirming_cart(&self, user_id: Uuid) -> anyhow::Result<Option<Cart>>;

    async fn inc_item(&self, cart_id: Uuid, item_id: Uuid) -> anyhow::Result<()>;
    async fn dec_item(&self, cart_id: Uuid, item_id: Uuid) -> anyhow::Result<()>;
    async fn reset_cart(&self, cart_id: Uuid) -> anyhow::Result<()>;

    async fn mark_confirming(&self, cart_id: Uuid) -> anyhow::Result<()>;
    async fn mark_active(&self, cart_id: Uuid) -> anyhow::Result<()>;

    async fn get_cart_view(&self, cart_id: Uuid) -> anyhow::Result<CartView>;
}

/// ─────────────────────────────
/// Pg adapter
/// ─────────────────────────────

#[derive(Clone)]
pub struct PgCartRepo {
    pool: PgPool,
}

impl PgCartRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// ─────────────────────────────
/// CartRepo impl (app → db bridge)
/// ─────────────────────────────

#[async_trait]
impl CartRepo for PgCartRepo {
    /* ───────────── Find carts ───────────── */

    async fn find_active_cart(&self, user_id: Uuid) -> Result<Option<Cart>> {
        let row = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, user_id, status
            FROM carts
            WHERE user_id = $1
              AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Cart {
            id: r.id,
            user_id: r.user_id,
            status: CartStatus::Active,
        }))
    }

    async fn create_active_cart(&self, user_id: Uuid) -> Result<Cart> {
        let r = sqlx::query_as::<_, CartRow>(
            r#"
            INSERT INTO carts (user_id, status)
            VALUES ($1, 'active')
            RETURNING id, user_id, status
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Cart {
            id: r.id,
            user_id: r.user_id,
            status: CartStatus::Active,
        })
    }

    async fn find_confirming_cart(&self, user_id: Uuid) -> Result<Option<Cart>> {
        let row = sqlx::query_as::<_, CartRow>(
            r#"
            SELECT id, user_id, status
            FROM carts
            WHERE user_id = $1
              AND status = 'confirming'
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Cart {
            id: r.id,
            user_id: r.user_id,
            status: CartStatus::Confirming,
        }))
    }

    /* ───────────── Mutations (by cart_id) ───────────── */

    async fn inc_item(&self, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
        let rec = sqlx::query(
            r#"
            WITH guard AS (
                SELECT 1 FROM carts WHERE id = $1 AND status = 'active'
            ),
            upsert AS (
                INSERT INTO cart_items (cart_id, menu_item_id, quantity, price_snapshot)
                SELECT
                    $1,
                    $2,
                    1,
                    mi.price
                FROM guard, menu_items mi
                WHERE mi.id = $2
                ON CONFLICT (cart_id, menu_item_id)
                DO UPDATE SET quantity = cart_items.quantity + 1
                RETURNING 1
            )
            SELECT COALESCE((SELECT 1 FROM upsert), 0) AS ok
            "#,
        )
        .bind(cart_id)
        .bind(menu_item_id)
        .fetch_one(&self.pool)
        .await?;

        let ok: i32 = rec.get("ok");
        if ok == 0 {
            return Err(anyhow!("cart not active or item not found"));
        }

        Ok(())
    }

    async fn dec_item(&self, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
        let rec = sqlx::query(
            r#"
            WITH guard AS (
                SELECT 1 FROM carts WHERE id = $1 AND status = 'active'
            ),
            existing AS (
                SELECT quantity FROM cart_items
                WHERE cart_id = $1 AND menu_item_id = $2
            ),
            deleted AS (
                DELETE FROM cart_items
                WHERE cart_id = $1 AND menu_item_id = $2
                  AND (SELECT quantity FROM existing) = 1
                  AND EXISTS (SELECT 1 FROM guard)
                RETURNING 1
            ),
            updated AS (
                UPDATE cart_items
                SET quantity = quantity - 1
                WHERE cart_id = $1 AND menu_item_id = $2
                  AND (SELECT quantity FROM existing) > 1
                  AND EXISTS (SELECT 1 FROM guard)
                RETURNING 1
            )
            SELECT COALESCE(
                (SELECT 1 FROM deleted),
                (SELECT 1 FROM updated),
                0
            ) AS ok
            "#,
        )
        .bind(cart_id)
        .bind(menu_item_id)
        .fetch_one(&self.pool)
        .await?;

        let ok: i32 = rec.get("ok");
        if ok == 0 {
            return Err(anyhow!("cart not active or item not found"));
        }

        Ok(())
    }

    async fn reset_cart(&self, cart_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM cart_items
            WHERE cart_id = $1
              AND EXISTS (
                  SELECT 1 FROM carts WHERE id = $1 AND status = 'active'
              )
            "#,
        )
        .bind(cart_id)
        .execute(&self.pool)
        .await?;

        let active = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM carts WHERE id = $1 AND status = 'active'
            )
            "#,
        )
        .bind(cart_id)
        .fetch_one(&self.pool)
        .await?;

        if !active {
            return Err(anyhow!("cart not active"));
        }

        Ok(())
    }

    async fn mark_confirming(&self, cart_id: Uuid) -> Result<()> {
        let res = sqlx::query(
            r#"
            UPDATE carts
            SET status = 'confirming',
                updated_at = NOW()
            WHERE id = $1 AND status = 'active'
            "#,
        )
        .bind(cart_id)
        .execute(&self.pool)
        .await?;

        if res.rows_affected() == 0 {
            return Err(anyhow!("cart not active"));
        }

        Ok(())
    }

    async fn mark_active(&self, cart_id: Uuid) -> Result<()> {
        let res = sqlx::query(
            r#"
            UPDATE carts
            SET status = 'active',
                updated_at = NOW()
            WHERE id = $1 AND status = 'confirming'
            "#,
        )
        .bind(cart_id)
        .execute(&self.pool)
        .await?;

        if res.rows_affected() == 0 {
            return Err(anyhow!("cart not confirming"));
        }

        Ok(())
    }

    /* ───────────── Views (cart_id only) ───────────── */

    async fn get_cart_view(&self, cart_id: Uuid) -> Result<CartView> {
        let rows: Vec<CartItemRow> = sqlx::query_as::<_, CartItemRow>(
            r#"
            SELECT
                mi.title,
                ci.menu_item_id,
                ci.quantity,
                ci.price_snapshot
            FROM cart_items ci
            JOIN menu_items mi ON mi.id = ci.menu_item_id
            WHERE ci.cart_id = $1
            ORDER BY mi.position, mi.title
            "#,
        )
        .bind(cart_id)
        .fetch_all(&self.pool)
        .await?;

        let mut total_price = 0;

        let items = rows
            .into_iter()
            .map(|r| {
                total_price += r.price_snapshot * r.quantity as i64;
                CartItemView {
                    menu_item_id: r.menu_item_id,
                    title: r.title,
                    quantity: r.quantity,
                    price_snapshot: r.price_snapshot,
                }
            })
            .collect();

        Ok(CartView { items, total_price })
    }
}
