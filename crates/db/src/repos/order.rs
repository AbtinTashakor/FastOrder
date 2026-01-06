use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::OrderRow;

// app domain
use app::{
    models::order::{Order, OrderStatus},
    repos::order::OrderRepo,
};

#[derive(Clone)]
pub struct PgOrderRepo {
    pool: PgPool,
}

impl PgOrderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /* ───────────────────── Public API (infra) ───────────────────── */

    async fn create_from_cart_inner(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> Result<Order> {
        let mut tx = self.pool.begin().await?;

        self.lock_cart(&mut tx, user_id, cart_id).await?;

        let items = self.load_cart_items(&mut tx, cart_id).await?;
        if items.is_empty() {
            return Err(anyhow!("cart is empty"));
        }

        let total_price: i64 = items
            .iter()
            .map(|i| i.price_snapshot * i.quantity as i64)
            .sum();

        let daily_no = self.next_daily_number(&mut tx).await?;
        let order_code = format!(
            "FO-{}-{}",
            chrono::Local::now().format("%Y%m%d"),
            daily_no
        );

        let row = self
            .insert_order(
                &mut tx,
                user_id,
                daily_no,
                &order_code,
                total_price,
            )
            .await?;

        self.insert_order_items(&mut tx, row.id, items).await?;
        self.clear_cart(&mut tx, cart_id).await?;

        tx.commit().await?;
        Ok(Self::map_order(row)?)
    }

    /* ───────────────────── Mapping ───────────────────── */

    fn map_order(row: OrderRow) -> Result<Order> {
        Ok(Order {
            id: row.id,
            user_id: row.user_id,
            order_day: row.order_day,
            daily_no: row.daily_no,
            order_code: row.order_code,
            total_price: row.total_price,

            status: OrderStatus::try_from(row.status.as_str())?,

            operator_id: row.operator_id,
            assigned_at: row.assigned_at,
            seen_at: row.seen_at,

            prep_time_minutes: row.prep_time_minutes,
            retry_count: row.retry_count,

            created_at: row.created_at,
        })
    }

    /* ───────────────────── Internals ───────────────────── */

    async fn lock_cart(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> Result<()> {
        let locked = sqlx::query!(
            r#"
            UPDATE carts
            SET status = 'locked',
                updated_at = NOW()
            WHERE id = $1
              AND user_id = $2
              AND status = 'confirming'
            RETURNING id
            "#,
            cart_id,
            user_id
        )
        .fetch_optional(&mut **tx)
        .await?;

        if locked.is_none() {
            return Err(anyhow!("cart not confirming"));
        }

        Ok(())
    }

    async fn load_cart_items(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        cart_id: Uuid,
    ) -> Result<Vec<CartItemSnapshot>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mi.title          AS "title!",
                ci.quantity       AS quantity,
                ci.price_snapshot AS price_snapshot
            FROM cart_items ci
            JOIN menu_items mi ON mi.id = ci.menu_item_id
            WHERE ci.cart_id = $1
            "#,
            cart_id
        )
        .fetch_all(&mut **tx)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| CartItemSnapshot {
                title: r.title,
                quantity: r.quantity,
                price_snapshot: r.price_snapshot,
            })
            .collect())
    }

    async fn next_daily_number(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<i32> {
        sqlx::query!(
            r#"
            INSERT INTO daily_counters (day, last_no)
            VALUES (CURRENT_DATE, 0)
            ON CONFLICT (day) DO NOTHING
            "#
        )
        .execute(&mut **tx)
        .await?;

        let row = sqlx::query!(
            r#"
            UPDATE daily_counters
            SET last_no = last_no + 1
            WHERE day = CURRENT_DATE
            RETURNING last_no
            "#
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(row.last_no)
    }

    async fn insert_order(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        daily_no: i32,
        order_code: &str,
        total_price: i64,
    ) -> Result<OrderRow> {
        let order = sqlx::query_as::<_, OrderRow>(
            r#"
            INSERT INTO orders (
                user_id,
                order_day,
                daily_no,
                order_code,
                total_price,
                status
            )
            VALUES (
                $1,
                CURRENT_DATE,
                $2,
                $3,
                $4,
                'PENDING_ASSIGN'
            )
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(daily_no)
        .bind(order_code)
        .bind(total_price)
        .fetch_one(&mut **tx)
        .await?;

        Ok(order)
    }

    async fn insert_order_items(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        order_id: Uuid,
        items: Vec<CartItemSnapshot>,
    ) -> Result<()> {
        for item in items {
            sqlx::query!(
                r#"
                INSERT INTO order_items (
                    order_id,
                    title_snapshot,
                    price_snapshot,
                    quantity
                )
                VALUES ($1, $2, $3, $4)
                "#,
                order_id,
                item.title,
                item.price_snapshot,
                item.quantity
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    async fn clear_cart(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        cart_id: Uuid,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM cart_items
            WHERE cart_id = $1
            "#,
            cart_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

/* ───────────────────── Trait implementation ───────────────────── */

#[async_trait]
impl OrderRepo for PgOrderRepo {
    async fn create_from_cart(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> Result<Order> {
        self.create_from_cart_inner(user_id, cart_id).await
    }
}

/* ───────────────────── Internal DTO ───────────────────── */

struct CartItemSnapshot {
    title: String,
    quantity: i32,
    price_snapshot: i64,
}
