use anyhow::{anyhow, Result};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{OrderItemSnapshotRow, OrderRow};

/// Creates an order from a cart atomically:
/// - transitions cart status: confirming -> locked (and validates ownership)
/// - reads cart items snapshot (title from menu_items, price from cart_items.price_snapshot)
/// - inserts into orders + order_items
/// - clears cart_items after successful order creation
/// - assigns daily_no based on daily_counters table
///
/// Expected schema:
/// carts.status IN ('active', 'confirming', 'locked')
/// cart_items has: (cart_id, menu_item_id, quantity, price_snapshot)

pub async fn create_order_from_cart(
    pool: &PgPool,
    customer_id: Uuid,
    cart_id: Uuid,
) -> Result<OrderRow> {
    let mut tx = pool.begin().await?;

    // 1) lock cart
    let locked = sqlx::query!(
        r#"
        UPDATE carts
        SET status = 'locked',
            updated_at = NOW()
        WHERE id = $1
          AND customer_id = $2
          AND status = 'confirming'
        RETURNING id
        "#,
        cart_id,
        customer_id
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
    let order_code = format!("FO-{}", daily_no);

    // 6) insert order
    let order = sqlx::query_as::<_, OrderRow>(
        r#"
        INSERT INTO orders (
            customer_id,
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
            customer_id,
            order_day,
            daily_no,
            order_code,
            total_price,
            status,
            prep_time_minutes,
            created_at
        "#,
    )
    .bind(customer_id)
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

pub async fn accept_order(pool: &PgPool, order_id: Uuid, prep_time_minutes: i32) -> Result<()> {
    let updated = sqlx::query(
        r#"
        UPDATE orders
        SET status = 'accepted', prep_time_minutes = $2
        WHERE id = $1 AND status = 'pending'
        "#,
    )
    .bind(order_id)
    .bind(prep_time_minutes)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(anyhow!("order not found or not pending"));
    }
    Ok(())
}

pub async fn reject_order(pool: &PgPool, order_id: Uuid) -> Result<()> {
    let updated = sqlx::query(
        r#"
        UPDATE orders
        SET status = 'rejected', prep_time_minutes = NULL
        WHERE id = $1 AND status = 'pending'
        "#,
    )
    .bind(order_id)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(anyhow!("order not found or not pending"));
    }
    Ok(())
}

// ----------------- helpers -----------------

async fn lock_confirming_cart(
    tx: &mut Transaction<'_, Postgres>,
    customer_id: Uuid,
    cart_id: Uuid,
) -> Result<()> {
    // carts.status is expected: 'active' | 'confirming' | 'locked'
    // Only allow checkout when confirming, then lock it.
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        UPDATE carts
        SET status = 'locked',
            updated_at = NOW()
        WHERE id = $1
          AND customer_id = $2
          AND status = 'confirming'
        RETURNING id
        "#,
    )
    .bind(cart_id)
    .bind(customer_id)
    .fetch_optional(&mut **tx)
    .await?;

    match row {
        Some((_id,)) => Ok(()),
        None => Err(anyhow!(
            "cart not found, not owned by customer, or not confirming"
        )),
    }
}

async fn load_cart_items_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    cart_id: Uuid,
) -> Result<Vec<OrderItemSnapshotRow>> {
    // We expect:
    // cart_items(cart_id, menu_item_id, quantity, price_snapshot)
    // menu_items(id, title)
    let rows = sqlx::query_as::<_, OrderItemSnapshotRow>(
        r#"
        SELECT
            mi.title        AS title_snapshot,
            ci.price_snapshot AS price_snapshot,
            ci.quantity     AS quantity
        FROM cart_items ci
        JOIN menu_items mi ON mi.id = ci.menu_item_id
        WHERE ci.cart_id = $1
        ORDER BY mi.title ASC
        "#,
    )
    .bind(cart_id)
    .fetch_all(&mut **tx)
    .await?;

    Ok(rows)
}
