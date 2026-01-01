use std::collections::HashMap;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use thousands::Separable;
use uuid::Uuid;

use app::cart::{service::CartState, CartView};


use crate::context::BotContext;

fn category_emoji(title: &str) -> &str {
    match title {
        "پیتزا" => "🍕",
        "سوخاری" => "🍗",
        "پیش غذا" | "پیش‌غذا" => "🥗",
        _ => "📂",
    }
}

/// ─────────────────────────────
/// Active cart view (editable)
/// ─────────────────────────────
pub async fn render_cart_view(
    ctx: &BotContext,
    cart_id: Uuid,
) -> anyhow::Result<(String, InlineKeyboardMarkup)> {
    let CartView { items, total_price } = ctx.cart_service.get_cart_view(cart_id).await?;

    let menu_items = ctx.menu_service.get_menu_items().await?;

    let cart_map: HashMap<Uuid, i32> = items
        .into_iter()
        .map(|i| (i.menu_item_id, i.quantity))
        .collect();

    let mut text = String::from("🛒 سفارش شما:\n");
    let mut keyboard = Vec::new();

    for item in &menu_items {
        if let Some(qty) = cart_map.get(&item.id) {
            if *qty > 0 {
                text.push_str(&format!("• {} × {}\n", item.title, qty));
            }
        }
    }

    if total_price == 0 {
        text.push_str("— هنوز چیزی انتخاب نکردی —\n");
    }

    text.push_str(&format!(
        "\n💰 جمع کل: {} تومان\n\n",
        total_price.separate_with_commas()
    ));

    // ---------- کیبورد ----------
    let mut last_category_id: Option<Uuid> = None;

    for item in menu_items {
        // header کتگوری
        if last_category_id != Some(item.category_id) {
            keyboard.push(vec![InlineKeyboardButton::callback(
                format!(
                    "{} {}",
                    category_emoji(&item.category_title),
                    item.category_title
                ),
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

        keyboard.push(vec![InlineKeyboardButton::callback(
            label,
            "noop".to_string(),
        )]);

        let mut controls = Vec::new();

        if qty > 0 {
            controls.push(InlineKeyboardButton::callback(
                "➖",
                format!("cart:dec:{}", item.id),
            ));
        } else {
            controls.push(InlineKeyboardButton::callback("-", "noop".to_string()));
        }

        controls.push(InlineKeyboardButton::callback(
            "➕",
            format!("cart:inc:{}", item.id),
        ));

        keyboard.push(controls);
    }

    if total_price > 0 {
        keyboard.push(vec![
            InlineKeyboardButton::callback("✅ تکمیل سفارش", "cart:complete".to_string()),
            InlineKeyboardButton::callback("🔄 سفارش جدید", "cart:reset".to_string()),
        ]);
    }

    Ok((text, InlineKeyboardMarkup::new(keyboard)))
}

/// ─────────────────────────────
/// Confirming cart view (read-only)
/// ─────────────────────────────
pub async fn render_confirming_view(ctx: &BotContext, cart_id: Uuid) -> anyhow::Result<String> {
    let CartView { items, total_price } = ctx.cart_service.get_cart_view(cart_id).await?;

    let mut lines = Vec::new();

    for item in items {
        let line_total = item.price_snapshot * item.quantity as i64;
        lines.push(format!(
            "• {} × {} — {} تومان",
            item.title,
            item.quantity,
            line_total.separate_with_commas()
        ));
    }

    Ok(format!(
        "🧾 *خلاصه سفارش شما:*\n\n{}\n\n\
         ───────────────\n\
         💰 *جمع کل:* {} تومان\n\n\
         ❓ آیا سفارش نهایی شود؟",
        lines.join("\n"),
        total_price.separate_with_commas()
    ))
}

pub async fn render_cart_by_state(
    ctx: &BotContext,
    user_id: Uuid,
) -> anyhow::Result<CartRenderResult> {
    match ctx.cart_service.resolve_cart_state(user_id).await? {
        CartState::Active(cart_id) => {
            let (text, keyboard) = render_cart_view(ctx, cart_id).await?;
            Ok(CartRenderResult::Active { text, keyboard })
        }

        CartState::Confirming(cart_id) => {
            let text = render_confirming_view(ctx, cart_id).await?;
            Ok(CartRenderResult::Confirming {
                text,
                keyboard: confirming_keyboard(),
            })
        }

        CartState::New => {
            // فقط تضمین می‌کنیم cart فعال ساخته شود
            let cart = ctx.cart_service.complete_new_cart(user_id).await?;

            let (text, keyboard) = render_cart_view(ctx, cart.id).await?;
            Ok(CartRenderResult::Active { text, keyboard })
        }
    }
}

pub fn confirming_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✏️ ویرایش سفارش", "cart:edit"),
        InlineKeyboardButton::callback("✅ تأیید نهایی", "cart:confirm"),
    ]])
}

pub enum CartRenderResult {
    Active {
        text: String,
        keyboard: InlineKeyboardMarkup,
    },
    Confirming {
        text: String,
        keyboard: InlineKeyboardMarkup,
    },
}
