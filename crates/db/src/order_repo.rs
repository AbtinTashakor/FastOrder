use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::OrderRow;

#[derive(Clone)]
pub struct PgOrderRepo {
    pool: PgPool,
}

impl PgOrderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_order_from_cart(
        &self,
        user_id: Uuid,
        cart_id: Uuid,
    ) -> Result<OrderRow> {
        let mut tx = self.pool.begin().await?;

        // 1) lock cart
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
        .fetch_optional(&mut *tx)
        .await?;

        if locked.is_none() {
            return Err(anyhow!("cart not found or not confirming"));
        }

        // 2) load cart items
        let items = sqlx::query!(
            r#"
            SELECT
                mi.title          AS title,
                ci.quantity       AS quantity,
                ci.price_snapshot AS price_snapshot
            FROM cart_items ci
            JOIN menu_items mi ON mi.id = ci.menu_item_id
            WHERE ci.cart_id = $1
            "#,
            cart_id
        )
        .fetch_all(&mut *tx)
        .await?;

        if items.is_empty() {
            return Err(anyhow!("cart is empty"));
        }

        // 3) total price
        let total_price: i64 = items
            .iter()
            .map(|i| i.price_snapshot * i.quantity as i64)
            .sum();

        // 4) ensure daily counter row
        sqlx::query!(
            r#"
            INSERT INTO daily_counters (day, last_no)
            VALUES (CURRENT_DATE, 0)
            ON CONFLICT (day) DO NOTHING
            "#
        )
        .execute(&mut *tx)
        .await?;

        // 5) increment daily counter
        let counter = sqlx::query!(
            r#"
            UPDATE daily_counters
            SET last_no = last_no + 1
            WHERE day = CURRENT_DATE
            RETURNING last_no
            "#
        )
        .fetch_one(&mut *tx)
        .await?;

        let daily_no = counter.last_no;
        let order_code = format!(
            "FO-{}-{}",
            chrono::Local::now().format("%Y%m%d"),
            daily_no
        );

        // 6) insert order
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
                'pending'
            )
            RETURNING
                id,
                user_id,
                order_day,
                daily_no,
                order_code,
                total_price,
                status,
                prep_time_minutes,
                created_at
            "#
        )
        .bind(user_id)
        .bind(daily_no)
        .bind(&order_code)
        .bind(total_price)
        .fetch_one(&mut *tx)
        .await?;

        // 7) insert order items
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
                order.id,
                item.title,
                item.price_snapshot,
                item.quantity
            )
            .execute(&mut *tx)
            .await?;
        }

        // 8) clear cart
        sqlx::query!(
            r#"
            DELETE FROM cart_items
            WHERE cart_id = $1
            "#,
            cart_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(order)
    }
}
