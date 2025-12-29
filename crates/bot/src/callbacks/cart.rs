use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::MaybeInaccessibleMessage;
use uuid::Uuid;

use crate::context::BotContext;
use crate::views::cart_view::render_cart_view;

use db::{cart_repo, customer_repo};

pub async fn handle_cart_action(
    bot: Bot,
    ctx: BotContext,
    q: CallbackQuery,
) -> Result<()> {
    let data = q.data.as_deref().unwrap_or("");

    // فقط پیام‌های قابل دسترسی رو edit می‌کنیم
    let msg = match q.message.as_ref() {
        Some(MaybeInaccessibleMessage::Regular(m)) => m,
        _ => {
            // پیام inaccessible — فقط spinner رو بخوابون و تمام
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    };

    // user_id رو از خود CallbackQuery می‌گیریم (نه از msg)
    let user_id = q.from.id.0 as i64;

    // customer از DB
    let customer = match customer_repo::find_by_telegram_id(&ctx.db, user_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
        Err(err) => {
            log::error!("cart callback: failed to load customer: {:?}", err);
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    };

    // cart فعال
    let cart = cart_repo::get_or_create_active_cart(&ctx.db, customer.id).await?;

    // اکشن
    match parse_action(data) {
        CartAction::Noop => {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
        CartAction::Checkout => {
            // قفل کردن cart
            if let Err(err) = cart_repo::lock_cart(&ctx.db, cart.id).await {
                log::error!("checkout: lock_cart failed: {:?}", err);
            }

            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
        CartAction::Inc(item_id) => {
            cart_repo::inc_item(&ctx.db, cart.id, item_id).await?;
        }
        CartAction::Dec(item_id) => {
            cart_repo::dec_item(&ctx.db, cart.id, item_id).await?;
        }
        CartAction::Del(item_id) => {
            cart_repo::remove_item(&ctx.db, cart.id, item_id).await?;
        }
    }

    // رندر جدید
    let (new_text, new_keyboard) = render_cart_view(&ctx.db, customer.id).await?;

    // 🛑 Guard: اگر متن همونه، ادیت نکن (برای جلوگیری از message is not modified)
    if let Some(old_text) = msg.text() {
        if old_text == new_text {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    }

    // edit
    bot.edit_message_text(msg.chat.id, msg.id, new_text)
        .reply_markup(new_keyboard)
        .await?;

    // spinner off
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

#[derive(Debug)]
enum CartAction {
    Noop,
    Checkout,
    Inc(Uuid),
    Dec(Uuid),
    Del(Uuid),
}

fn parse_action(data: &str) -> CartAction {
    // cart:inc:<uuid>
    // cart:dec:<uuid>
    // cart:del:<uuid>
    // cart:noop:<uuid>
    // cart:checkout
    let mut parts = data.split(':');

    let ns = parts.next().unwrap_or("");
    if ns != "cart" {
        return CartAction::Noop;
    }

    match parts.next().unwrap_or("") {
        "checkout" => CartAction::Checkout,
        "noop" => CartAction::Noop,
        "inc" => parse_uuid(parts.next()).map(CartAction::Inc).unwrap_or(CartAction::Noop),
        "dec" => parse_uuid(parts.next()).map(CartAction::Dec).unwrap_or(CartAction::Noop),
        "del" => parse_uuid(parts.next()).map(CartAction::Del).unwrap_or(CartAction::Noop),
        _ => CartAction::Noop,
    }
}

fn parse_uuid(s: Option<&str>) -> Option<Uuid> {
    s.and_then(|v| Uuid::parse_str(v).ok())
}
