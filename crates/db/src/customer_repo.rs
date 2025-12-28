use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::CustomerRow;

pub async fn find_by_phone(pool: &PgPool, phone: &str) -> Result<Option<CustomerRow>> {
    let row = sqlx::query_as::<_, CustomerRow>(
        r#"
        SELECT * FROM customers
        WHERE phone_number = $1
        "#,
    )
    .bind(phone)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn verify_and_bind_telegram(
    pool: &PgPool,
    customer_id: Uuid,
    telegram_user_id: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE customers
        SET telegram_user_id = $1, is_verified = TRUE
        WHERE id = $2
        "#,
    )
    .bind(telegram_user_id)
    .bind(customer_id)
    .execute(pool)
    .await?;

    Ok(())
}
