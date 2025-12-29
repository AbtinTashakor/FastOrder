use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CartItemRow, CartRow};

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

pub async fn inc_item(pool: &PgPool, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
    // Requires UNIQUE(cart_id, menu_item_id)
    sqlx::query(
        r#"
        INSERT INTO cart_items (cart_id, menu_item_id, quantity)
        VALUES ($1, $2, 1)
        ON CONFLICT (cart_id, menu_item_id)
        DO UPDATE SET quantity = cart_items.quantity + 1
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn dec_item(pool: &PgPool, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
    // Atomic: if quantity == 1 -> delete
    // else -> decrement
    let affected: (i64,) = sqlx::query_as(
        r#"
        WITH existing AS (
            SELECT quantity
            FROM cart_items
            WHERE cart_id = $1 AND menu_item_id = $2
        ),
        deleted AS (
            DELETE FROM cart_items
            WHERE cart_id = $1 AND menu_item_id = $2
              AND (SELECT quantity FROM existing) = 1
            RETURNING 1
        ),
        updated AS (
            UPDATE cart_items
            SET quantity = quantity - 1
            WHERE cart_id = $1 AND menu_item_id = $2
              AND (SELECT quantity FROM existing) > 1
            RETURNING 1
        )
        SELECT COALESCE(
            (SELECT 1 FROM deleted),
            (SELECT 1 FROM updated),
            0
        ) AS affected
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .fetch_one(pool)
    .await?;

    if affected.0 == 0 {
        return Err(anyhow!("item not found in cart"));
    }

    Ok(())
}

pub async fn remove_item(pool: &PgPool, cart_id: Uuid, menu_item_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM cart_items
        WHERE cart_id = $1 AND menu_item_id = $2
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_cart_items(pool: &PgPool, cart_id: Uuid) -> Result<Vec<CartItemRow>> {
    let rows = sqlx::query_as::<_, CartItemRow>(
        r#"
        SELECT menu_item_id, quantity
        FROM cart_items
        WHERE cart_id = $1
        "#,
    )
    .bind(cart_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn lock_cart(pool: &PgPool, cart_id: Uuid) -> Result<()> {
    let updated = sqlx::query(
        r#"
        UPDATE carts
        SET status = 'locked'
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
