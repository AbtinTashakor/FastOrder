use std::collections::HashMap;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use thousands::Separable;
use uuid::Uuid;

use db::{cart_repo, menu_repo};

fn category_emoji(title: &str) -> &str {
    match title {
        "پیتزا" => "🍕",
        "سوخاری" => "🍗",
        "پیش غذا" | "پیش‌غذا" => "🥗",
        _ => "📂",
    }
}

pub async fn render_cart_view(
    pool: &sqlx::PgPool,
    customer_id: Uuid,
) -> anyhow::Result<(String, InlineKeyboardMarkup)> {
    let cart = cart_repo::get_or_create_active_cart(pool, customer_id).await?;
    let cart_items = cart_repo::list_cart_items(pool, cart.id).await?;
    let menu_items = menu_repo::list_available_items(pool).await?;
    // ⬆️ فرض: menu_items به ترتیب category مرتب شده

    let cart_map: HashMap<Uuid, i32> = cart_items
        .into_iter()
        .map(|i| (i.menu_item_id, i.quantity))
        .collect();

    let mut total: i64 = 0;
    let mut text = String::new();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    // ---------- متن بالای پیام ----------
    text.push_str("🛒 سفارش شما:\n");

    for item in &menu_items {
        if let Some(qty) = cart_map.get(&item.id) {
            if *qty > 0 {
                total += (*qty as i64) * item.price;
                text.push_str(&format!("• {} × {}\n", item.title, qty));
            }
        }
    }

    if total == 0 {
        text.push_str("— هنوز چیزی انتخاب نکردی —\n");
    }

    text.push_str(&format!(
        "\n💰 جمع کل: {} تومان\n\n",
        total.separate_with_commas()
    ));

    // ---------- کیبورد ----------
    let mut last_category_id: Option<Uuid> = None;

    for item in menu_items {
        // --- header کتگوری (وقتی عوض می‌شه) ---
        if last_category_id != Some(item.category_id) {
            let emoji = category_emoji(&item.category_title);
            keyboard.push(vec![InlineKeyboardButton::callback(
                format!("{} {}", emoji, item.category_title),
                "noop".to_string(),
            )]);
            last_category_id = Some(item.category_id);
        }

        let qty = cart_map.get(&item.id).copied().unwrap_or(0);

        let label = if qty > 0 {
            format!(
                "{} × {} — {}",
                item.title,
                qty,
                item.price.separate_with_commas()
            )
        } else {
            format!("{} — {}", item.title, item.price.separate_with_commas())
        };

        // دکمه بزرگ آیتم
        keyboard.push(vec![InlineKeyboardButton::callback(
            label,
            "noop".to_string(),
        )]);

        // دکمه‌های کنترل
        let mut controls = Vec::new();

        if qty > 0 {
            controls.push(InlineKeyboardButton::callback(
                "−",
                format!("cart:dec:{}", item.id),
            ));
        } else {
            controls.push(InlineKeyboardButton::callback("-", "noop".to_string()));
        }

        controls.push(InlineKeyboardButton::callback(
            "+",
            format!("cart:inc:{}", item.id),
        ));

        keyboard.push(controls);
    }

    if total > 0 {
        keyboard.push(vec![
            InlineKeyboardButton::callback("✅ تکمیل سفارش", "cart:checkout".to_string()),
            InlineKeyboardButton::callback("🔄 سفارش جدید", "cart:reset".to_string()),
        ]);
    }

    Ok((text, InlineKeyboardMarkup::new(keyboard)))
}
