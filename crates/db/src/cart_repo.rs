use anyhow::{anyhow, Result};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::models::{CartItemRow, CartRow};

/// Gets the current active cart for customer or creates a new one.
/// Active cart is editable (+/-).
pub async fn get_or_create_active_cart(pool: &PgPool, customer_id: Uuid) -> Result<CartRow> {
    if let Some(cart) = sqlx::query_as::<_, CartRow>(
        r#"
        SELECT id, customer_id, status
        FROM carts
        WHERE customer_id = $1 AND status = 'active'
        "#,
    )
    .bind(customer_id)
    .fetch_optional(pool)
    .await?
    {
        return Ok(cart);
    }

    let cart = sqlx::query_as::<_, CartRow>(
        r#"
        INSERT INTO carts (customer_id, status)
        VALUES ($1, 'active')
        RETURNING id, customer_id, status
        "#,
    )
    .bind(customer_id)
    .fetch_one(pool)
    .await?;

    Ok(cart)
}

/// Transitions cart from active -> confirming (user pressed "Complete order").
/// In confirming state, cart is read-only (no +/-).
pub async fn mark_confirming(pool: &PgPool, cart_id: Uuid) -> Result<()> {
    let updated = sqlx::query(
        r#"
        UPDATE carts
        SET status = 'confirming',
            updated_at = NOW()
        WHERE id = $1 AND status = 'active'
        "#,
    )
    .bind(cart_id)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(anyhow!("cart not found or not active"));
    }

    Ok(())
}

/// Transitions cart from confirming -> active (user pressed "Edit order").
pub async fn mark_active(pool: &PgPool, cart_id: Uuid) -> Result<()> {
    let updated = sqlx::query(
        r#"
        UPDATE carts
        SET status = 'active',
            updated_at = NOW()
        WHERE id = $1 AND status = 'confirming'
        "#,
    )
    .bind(cart_id)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(anyhow!("cart not found or not confirming"));
    }

    Ok(())
}

/// Increments an item in the cart (active only).
/// On first insert, sets price_snapshot from menu_items.price.
/// On conflict, only increments quantity (keeps original snapshot).
pub async fn inc_item(pool: &PgPool, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
    // Guard: only allow modifications when cart is active
    // Also: if menu_item_id doesn't exist, insert should not happen.
    let res = sqlx::query(
        r#"
        WITH cart_guard AS (
            SELECT 1
            FROM carts
            WHERE id = $1 AND status = 'active'
        ),
        ins AS (
            INSERT INTO cart_items (cart_id, menu_item_id, quantity, price_snapshot)
            SELECT
                $1,
                $2,
                1,
                mi.price
            FROM cart_guard, menu_items mi
            WHERE mi.id = $2
            ON CONFLICT (cart_id, menu_item_id)
            DO UPDATE SET quantity = cart_items.quantity + 1
            RETURNING 1
        )
        SELECT COALESCE((SELECT 1 FROM ins), 0) AS inserted_or_updated
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .fetch_one(pool)
    .await?;

    // If guard failed OR menu item not found, CTE produces 0.
    let affected: i32 = res.get::<i32, _>("inserted_or_updated");
    if affected == 0 {
        return Err(anyhow!("cart not active or menu item not found"));
    }

    Ok(())
}

/// Decrements an item in the cart (active only).
/// Atomic: if quantity == 1 -> delete else -> decrement.
/// Keeps price_snapshot unchanged for remaining rows.
pub async fn dec_item(pool: &PgPool, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
    let rec = sqlx::query(
        r#"
        WITH cart_guard AS (
            SELECT 1
            FROM carts
            WHERE id = $1 AND status = 'active'
        ),
        existing AS (
            SELECT quantity
            FROM cart_items
            WHERE cart_id = $1 AND menu_item_id = $2
        ),
        deleted AS (
            DELETE FROM cart_items
            WHERE cart_id = $1 AND menu_item_id = $2
              AND (SELECT quantity FROM existing) = 1
              AND EXISTS (SELECT 1 FROM cart_guard)
            RETURNING 1 AS ok
        ),
        updated AS (
            UPDATE cart_items
            SET quantity = quantity - 1
            WHERE cart_id = $1 AND menu_item_id = $2
              AND (SELECT quantity FROM existing) > 1
              AND EXISTS (SELECT 1 FROM cart_guard)
            RETURNING 1 AS ok
        )
        SELECT COALESCE(
            (SELECT ok FROM deleted),
            (SELECT ok FROM updated),
            0
        ) AS affected
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .fetch_one(pool)
    .await?;

    let affected: i32 = rec.get::<i32, _>("affected");
    if affected == 0 {
        return Err(anyhow!("cart not active or item not found in cart"));
    }

    Ok(())
}

/// Lists cart items with snapshot price (used for confirming screen / totals).

pub async fn list_cart_items(pool: &PgPool, cart_id: Uuid) -> Result<Vec<CartItemRow>> {
    let rows = sqlx::query_as::<_, CartItemRow>(
        r#"
        SELECT
            mi.title              AS title,
            ci.menu_item_id       AS menu_item_id,
            ci.quantity           AS quantity,
            ci.price_snapshot     AS price_snapshot
        FROM cart_items ci
        JOIN menu_items mi ON mi.id = ci.menu_item_id
        WHERE ci.cart_id = $1
        ORDER BY mi.position, mi.title
        "#,
    )
    .bind(cart_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Clears cart items (active only) - used for "new order" / reset cart.
/// Keeps cart status as active.
pub async fn reset_cart(pool: &PgPool, cart_id: Uuid) -> Result<()> {
    let updated = sqlx::query(
        r#"
        DELETE FROM cart_items
        WHERE cart_id = $1
          AND EXISTS (SELECT 1 FROM carts WHERE id = $1 AND status = 'active')
        "#,
    )
    .bind(cart_id)
    .execute(pool)
    .await?;

    // If cart is not active, we treat as error to prevent clearing a confirming/locked cart.
    // However, if cart is active but empty, rows_affected may be 0. That's OK.
    let cart_active = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (SELECT 1 FROM carts WHERE id = $1 AND status = 'active')
        "#,
    )
    .bind(cart_id)
    .fetch_one(pool)
    .await?;

    if !cart_active {
        return Err(anyhow!("cart not found or not active"));
    }

    Ok(())
}

pub async fn get_confirming_cart(
    pool: &PgPool,
    customer_id: Uuid,
) -> Result<CartRow> {
    sqlx::query_as!(
        CartRow,
        r#"
        SELECT id, customer_id, status
        FROM carts
        WHERE customer_id = $1
          AND status = 'confirming'
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
        customer_id
    )
    .fetch_one(pool)
    .await
    .map_err(|_| anyhow!("cart not found or not confirming"))
}
