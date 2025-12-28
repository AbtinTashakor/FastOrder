use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CartRow, CartItemRow};

pub async fn get_or_create_active_cart(
    pool: &PgPool,
    customer_id: Uuid,
) -> Result<CartRow> {
    if let Some(cart) = sqlx::query_as::<_, CartRow>(
        "SELECT * FROM carts WHERE customer_id = $1 AND status = 'active'"
    )
    .bind(customer_id)
    .fetch_optional(pool)
    .await? {
        return Ok(cart);
    }

    let cart = sqlx::query_as::<_, CartRow>(
        r#"
        INSERT INTO carts (customer_id, status)
        VALUES ($1, 'active')
        RETURNING *
        "#,
    )
    .bind(customer_id)
    .fetch_one(pool)
    .await?;

    Ok(cart)
}

pub async fn upsert_cart_item(
    pool: &PgPool,
    cart_id: Uuid,
    menu_item_id: Uuid,
    delta: i32,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO cart_items (cart_id, menu_item_id, quantity)
        VALUES ($1, $2, GREATEST($3, 1))
        ON CONFLICT (cart_id, menu_item_id)
        DO UPDATE SET quantity = cart_items.quantity + $3
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .bind(delta)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_cart_items(
    pool: &PgPool,
    cart_id: Uuid,
) -> Result<Vec<CartItemRow>> {
    let rows = sqlx::query_as::<_, CartItemRow>(
        "SELECT menu_item_id, quantity FROM cart_items WHERE cart_id = $1"
    )
    .bind(cart_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn lock_cart(pool: &PgPool, cart_id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE carts SET status = 'locked' WHERE id = $1"
    )
    .bind(cart_id)
    .execute(pool)
    .await?;
    Ok(())
}
