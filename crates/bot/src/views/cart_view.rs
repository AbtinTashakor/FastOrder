use anyhow::Result;
use sqlx::PgPool;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use uuid::Uuid;
use thousands::Separable;
use db::{cart_repo, menu_repo};

pub async fn render_cart_view(
    pool: &PgPool,
    customer_id: Uuid,
) -> Result<(String, InlineKeyboardMarkup)> {
    let cart = cart_repo::get_or_create_active_cart(pool, customer_id).await?;
    let items = cart_repo::list_cart_items(pool, cart.id).await?;
    let menu_items = menu_repo::list_available_items(pool).await?;

    let mut total: i64 = 0;
    let mut text = String::from("🍔 سفارش شما\n\n");
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for cart_item in items {
        if let Some(menu) = menu_items.iter().find(|m| m.id == cart_item.menu_item_id) {
            let line_total = menu.price * cart_item.quantity as i64;
            total += line_total;

            text.push_str(&format!(
                "{} × {} — {}\n",
                menu.title, cart_item.quantity, line_total.separate_with_commas()
            ));

            keyboard.push(vec![
                InlineKeyboardButton::callback("➖", format!("c:-:{}", menu.id)),
                InlineKeyboardButton::callback("➕", format!("c:+:{}", menu.id)),
                InlineKeyboardButton::callback("🗑", format!("c:x:{}", menu.id)),
            ]);
        }
    }

    if total == 0 {
        text.push_str("سبد خرید خالیه 🍽\n");
    } else {
        text.push_str("\n────────────\n");
        text.push_str(&format!("💰 جمع کل: {} تومان\n", total.separate_with_commas()));

        keyboard.push(vec![
            InlineKeyboardButton::callback("✅ تکمیل سفارش", "cart:checkout"),
        ]);
    }

    Ok((text, InlineKeyboardMarkup::new(keyboard)))
}
