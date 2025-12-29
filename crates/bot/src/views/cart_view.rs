use std::collections::HashMap;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use uuid::Uuid;
use thousands::Separable;
use db::{cart_repo, menu_repo};

pub async fn render_cart_view(
    pool: &sqlx::PgPool,
    customer_id: Uuid,
) -> anyhow::Result<(String, InlineKeyboardMarkup)> {
    let cart = cart_repo::get_or_create_active_cart(pool, customer_id).await?;
    let cart_items = cart_repo::list_cart_items(pool, cart.id).await?;
    let menu_items = menu_repo::list_available_items(pool).await?;

    let cart_map: HashMap<Uuid, i32> = cart_items
        .into_iter()
        .map(|i| (i.menu_item_id, i.quantity))
        .collect();

    let mut total: i64 = 0;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut text = String::new();
    text.push_str("🛒 سبد خرید شما\n");
    text.push_str("برای افزودن یا ویرایش آیتم‌ها از دکمه‌ها استفاده کن 👇\n\n");

    for item in menu_items {
        let qty = cart_map.get(&item.id).copied().unwrap_or(0);
        total += (qty as i64) * item.price;

        let price_label = format!("{}", item.price.separate_with_commas());

        let mut row: Vec<InlineKeyboardButton> = Vec::new();

        // ➖
        if qty > 0 {
            row.push(
                InlineKeyboardButton::callback(
                    "➖",
                    format!("cart:dec:{}", item.id),
                )
            );
        }

        // لیبل وسط (اسم + تعداد + قیمت)
        let label = if qty > 0 {
            format!(
                "{} × {} — {}",
                item.title, qty, price_label
            )
        } else {
            format!(
                "{} — {}",
                 item.title, price_label
            )
        };

        row.push(
            InlineKeyboardButton::callback(
                label,
                format!("cart:noop:{}", item.id),
            )
        );

        // ➕
        row.push(
            InlineKeyboardButton::callback(
                "➕",
                format!("cart:inc:{}", item.id),
            )
        );

        // 🗑
        if qty > 0 {
            row.push(
                InlineKeyboardButton::callback(
                    "🗑",
                    format!("cart:del:{}", item.id),
                )
            );
        }

        keyboard.push(row);
    }

    text.push_str(&format!("\n💰 جمع کل: {} تومان", total.separate_with_commas()));

    if total > 0 {
        keyboard.push(vec![
            InlineKeyboardButton::callback(
                "✅ تکمیل سفارش",
                "cart:checkout".to_string(),
            )
        ]);
    }

    Ok((text, InlineKeyboardMarkup::new(keyboard)))
}
