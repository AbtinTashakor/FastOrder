use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, MaybeInaccessibleMessage};
use uuid::Uuid;

use app::services::cart::service::CartState;

use crate::context::BotContext;
use crate::views::cart_view::{
    confirming_keyboard, render_cart_by_state, render_confirming_view, CartRenderResult,
};

pub async fn handle_cart_action(bot: Bot, ctx: BotContext, q: CallbackQuery) -> Result<()> {
    let data = q.data.as_deref().unwrap_or("");

    let msg = match q.message.as_ref() {
        Some(MaybeInaccessibleMessage::Regular(m)) => m,
        _ => {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    };

    let telegram_id = q.from.id.0 as i64;

    let user = match ctx
        .user_service
        .get_verified_user_by_telegram(telegram_id)
        .await
    {
        Ok(u) => u,
        Err(_) => {
            bot.answer_callback_query(q.id).await?;
            return Ok(());
        }
    };

    let action = parse_action(data);

    // 🔑 resolve cart_id once
    let cart_state = ctx.cart_service.resolve_cart_state(user.id).await?;

    match action {
        /* ───────────── Editing (active cart only) ───────────── */
        CartAction::Inc(item_id) => {
            let CartState::Active(cart_id) = cart_state else {
                bot.answer_callback_query(q.id).await?;
                return Ok(());
            };

            ctx.cart_service.inc_item_by_cart(cart_id, item_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        CartAction::Dec(item_id) => {
            let CartState::Active(cart_id) = cart_state else {
                bot.answer_callback_query(q.id).await?;
                return Ok(());
            };

            ctx.cart_service.dec_item_by_cart(cart_id, item_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        CartAction::Reset => {
            let CartState::Active(cart_id) = cart_state else {
                bot.answer_callback_query(q.id).await?;
                return Ok(());
            };

            ctx.cart_service.reset_by_cart(cart_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        /* ───────────── Flow: active → confirming ───────────── */
        CartAction::Complete => {
            let CartState::Active(cart_id) = cart_state else {
                bot.answer_callback_query(q.id).await?;
                return Ok(());
            };

            ctx.cart_service.mark_confirming(cart_id).await?;

            let text = render_confirming_view(&ctx, cart_id).await?;

            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(confirming_keyboard())
                .await?;
        }

        /* ───────────── Flow: confirming → active ───────────── */
        CartAction::Edit => {
            let CartState::Confirming(cart_id) = cart_state else {
                bot.answer_callback_query(q.id).await?;
                return Ok(());
            };

            ctx.cart_service.mark_active(cart_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        /* ───────────── Checkout ───────────── */
        CartAction::Confirm => {
            let CartState::Confirming(cart_id) = cart_state else {
                bot.answer_callback_query(q.id).await?;
                return Ok(());
            };

            let order = ctx.order_service.create_from_cart(user.id, cart_id).await?;

            let text = format!("✅ سفارش شما ثبت شد\n\n🧾 کد سفارش: {}", order.order_code);

            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
        }

        CartAction::Noop => {}
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

/* ───────────────────────────── */

#[derive(Debug)]
enum CartAction {
    Noop,
    Inc(Uuid),
    Dec(Uuid),
    Reset,
    Complete,
    Edit,
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
    user_id: Uuid,
) -> anyhow::Result<()> {
    let render = render_cart_by_state(ctx, user_id).await?;

    let edit_result = match render {
        CartRenderResult::Active { text, keyboard } => {
            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .await
        }

        CartRenderResult::Confirming { text, keyboard } => {
            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .await
        }
    };

    match edit_result {
        Ok(_) => Ok(()),
        Err(err) if is_message_not_modified(&err) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn is_message_not_modified(err: &teloxide::RequestError) -> bool {
    matches!(
        err,
        teloxide::RequestError::Api(teloxide::ApiError::MessageNotModified)
    )
}
