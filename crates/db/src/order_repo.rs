use anyhow::{anyhow, Result};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{OrderItemSnapshotRow, OrderRow};

/// Creates an order from a cart atomically:
/// - locks the cart (active -> locked)
/// - reads cart items joined with menu_items (snapshot title/price)
/// - inserts into orders + order_items
pub async fn create_order_from_cart(pool: &PgPool, cart_id: Uuid) -> Result<OrderRow> {
    let mut tx = pool.begin().await?;

    // 1) Lock cart (must be active)
    let customer_id: Uuid = lock_cart_and_get_customer(&mut tx, cart_id).await?;

    // 2) Read items snapshot
    let items = load_cart_items_snapshot(&mut tx, cart_id).await?;
    if items.is_empty() {
        return Err(anyhow!("cart is empty"));
    }

    // 3) Calculate total
    let total_price: i64 = items
        .iter()
        .map(|i| i.price_snapshot.saturating_mul(i.quantity as i64))
        .sum();

    // 4) Insert order
    let order: OrderRow = sqlx::query_as(
        r#"
        INSERT INTO orders (customer_id, total_price, status)
        VALUES ($1, $2, 'pending')
        RETURNING id, customer_id, total_price, status, prep_time_minutes, created_at
        "#,
    )
    .bind(customer_id)
    .bind(total_price)
    .fetch_one(&mut *tx)
    .await?;

    // 5) Insert order_items (snapshot)
    for it in &items {
        sqlx::query(
            r#"
            INSERT INTO order_items (order_id, title_snapshot, price_snapshot, quantity)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(order.id)
        .bind(&it.title_snapshot)
        .bind(it.price_snapshot)
        .bind(it.quantity)
        .execute(&mut *tx)
        .await?;
    }

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

async fn lock_cart_and_get_customer(
    tx: &mut Transaction<'_, Postgres>,
    cart_id: Uuid,
) -> Result<Uuid> {
    // carts.status is expected: 'active' | 'locked'
    // Lock only if active, otherwise fail
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        UPDATE carts
        SET status = 'locked'
        WHERE id = $1 AND status = 'active'
        RETURNING customer_id
        "#,
    )
    .bind(cart_id)
    .fetch_optional(&mut **tx)
    .await?;

    match row {
        Some((customer_id,)) => Ok(customer_id),
        None => Err(anyhow!("cart not found or not active")),
    }
}

async fn load_cart_items_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    cart_id: Uuid,
) -> Result<Vec<OrderItemSnapshotRow>> {
    // We expect:
    // cart_items(cart_id, menu_item_id, quantity)
    // menu_items(id, title, price)
    let rows = sqlx::query_as::<_, OrderItemSnapshotRow>(
        r#"
        SELECT
            mi.title  AS title_snapshot,
            mi.price  AS price_snapshot,
            ci.quantity AS quantity
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
