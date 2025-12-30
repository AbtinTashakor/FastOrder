use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, MaybeInaccessibleMessage};
use uuid::Uuid;

use crate::context::BotContext;
use crate::views::cart_view::{confirming_keyboard, render_cart_view, render_confirming_view};

use db::{cart_repo, customer_repo, order_repo};

pub async fn handle_cart_action(bot: Bot, ctx: BotContext, q: CallbackQuery) -> Result<()> {
    let data = q.data.as_deref().unwrap_or("");

    let msg = match q.message.as_ref() {
        Some(MaybeInaccessibleMessage::Regular(m)) => m,
        _ => {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    };

    let user_id = q.from.id.0 as i64;

    let customer = match customer_repo::find_by_telegram_id(&ctx.db, user_id).await? {
        Some(c) => c,
        None => {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    };

    let action = parse_action(data);

    match action {
        // ───────────── Editing (active only) ─────────────
        CartAction::Inc(item_id) => {
            let cart = cart_repo::get_or_create_active_cart(&ctx.db, customer.id).await?;
            cart_repo::inc_item(&ctx.db, cart.id, item_id).await?;
            render_and_edit(&bot, &ctx, msg, customer.id).await?;
        }

        CartAction::Dec(item_id) => {
            let cart = cart_repo::get_or_create_active_cart(&ctx.db, customer.id).await?;
            cart_repo::dec_item(&ctx.db, cart.id, item_id).await?;
            render_and_edit(&bot, &ctx, msg, customer.id).await?;
        }

        CartAction::Reset => {
            let cart = cart_repo::get_or_create_active_cart(&ctx.db, customer.id).await?;
            cart_repo::reset_cart(&ctx.db, cart.id).await?;
            render_and_edit(&bot, &ctx, msg, customer.id).await?;
        }

        // ───────────── Flow: active → confirming ─────────────
        CartAction::Complete => {
            let cart = cart_repo::get_or_create_active_cart(&ctx.db, customer.id).await?;
            cart_repo::mark_confirming(&ctx.db, cart.id).await?;

            let text = render_confirming_view(&ctx.db, cart.id).await?;

            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(confirming_keyboard())
                .await?;
        }

        // ───────────── Flow: confirming → active ─────────────
        CartAction::Edit => {
            let cart = cart_repo::get_confirming_cart(&ctx.db, customer.id).await?;
            cart_repo::mark_active(&ctx.db, cart.id).await?;
            render_and_edit(&bot, &ctx, msg, customer.id).await?;
        }

        // ───────────── Checkout ─────────────
        CartAction::Confirm => {
            let cart = cart_repo::get_confirming_cart(&ctx.db, customer.id).await?;
            let order = order_repo::create_order_from_cart(&ctx.db, customer.id, cart.id).await?;

            let text = format!("✅ سفارش شما ثبت شد\n\n🧾 کد سفارش: FO-{}", order.daily_no);

            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
        }

        CartAction::Noop => {}
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

#[derive(Debug)]
enum CartAction {
    Noop,

    // editing
    Inc(Uuid),
    Dec(Uuid),
    Reset,

    // flow
    Complete,
    Edit,

    // checkout
    Confirm,
}

fn parse_action(data: &str) -> CartAction {
    let mut parts = data.split(':');

    if parts.next() != Some("cart") {
        return CartAction::Noop;
    }

    match parts.next().unwrap_or("") {
        "inc" => parse_uuid(parts.next())
            .map(CartAction::Inc)
            .unwrap_or(CartAction::Noop),
        "dec" => parse_uuid(parts.next())
            .map(CartAction::Dec)
            .unwrap_or(CartAction::Noop),
        "reset" => CartAction::Reset,

        "complete" => CartAction::Complete,
        "edit" => CartAction::Edit,
        "confirm" => CartAction::Confirm,

        _ => CartAction::Noop,
    }
}

fn parse_uuid(s: Option<&str>) -> Option<Uuid> {
    s.and_then(|v| Uuid::parse_str(v).ok())
}

async fn render_and_edit(
    bot: &Bot,
    ctx: &BotContext,
    msg: &Message,
    customer_id: Uuid,
) -> Result<()> {
    let (text, keyboard) = render_cart_view(&ctx.db, customer_id).await?;

    if msg.text().map(|t| t == text).unwrap_or(false) {
        return Ok(());
    }

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
