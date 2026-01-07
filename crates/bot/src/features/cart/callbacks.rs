use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, MaybeInaccessibleMessage};
use uuid::Uuid;

use app::services::cart::service::CartState;

use crate::{
    context::BotContext,
    router::callback_data::CallbackData,
    features::cart::views::{
        confirming_keyboard, render_cart_by_state, render_confirming_view, CartRenderResult,
    },
};

pub async fn handle(
    bot: Bot,
    q: CallbackQuery,
    ctx: BotContext,
    cb: CallbackData,
) -> Result<()> {
    let msg = match q.message.as_ref() {
        Some(MaybeInaccessibleMessage::Regular(m)) => m,
        _ => {
            let _ = bot.answer_callback_query(q.id).await;
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
            let _ = bot.answer_callback_query(q.id).await;
            return Ok(());
        }
    };

    let cart_state = ctx.cart_service.resolve_cart_state(user.id).await?;

    match cb.action.as_str() {
        "inc" => {
            let item_id = parse_uuid(cb.payload.get(0))?;
            let CartState::Active(cart_id) = cart_state else { return Ok(()); };

            ctx.cart_service.inc_item_by_cart(cart_id, item_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        "dec" => {
            let item_id = parse_uuid(cb.payload.get(0))?;
            let CartState::Active(cart_id) = cart_state else { return Ok(()); };

            ctx.cart_service.dec_item_by_cart(cart_id, item_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        "reset" => {
            let CartState::Active(cart_id) = cart_state else { return Ok(()); };

            ctx.cart_service.reset_by_cart(cart_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        "complete" => {
            let CartState::Active(cart_id) = cart_state else { return Ok(()); };

            ctx.cart_service.mark_confirming(cart_id).await?;
            let text = render_confirming_view(&ctx, cart_id).await?;

            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(confirming_keyboard())
                .await?;
        }

        "edit" => {
            let CartState::Confirming(cart_id) = cart_state else { return Ok(()); };

            ctx.cart_service.mark_active(cart_id).await?;
            render_and_edit(&bot, &ctx, msg, user.id).await?;
        }

        "confirm" => {
            let CartState::Confirming(cart_id) = cart_state else { return Ok(()); };

            let order = ctx.order_service.create_from_cart(user.id, cart_id).await?;
            let text = format!("✅ سفارش شما ثبت شد\n\n🧾 کد سفارش: {}", order.order_code);

            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
        }

        _ => {}
    }

    let _ = bot.answer_callback_query(q.id).await;
    Ok(())
}

/* ───────────────────────────── */

fn parse_uuid(value: Option<&String>) -> Result<Uuid> {
    value
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| anyhow::anyhow!("invalid uuid in callback payload"))
}

async fn render_and_edit(
    bot: &Bot,
    ctx: &BotContext,
    msg: &Message,
    user_id: Uuid,
) -> Result<()> {
    let render = render_cart_by_state(ctx, user_id).await?;

    let result = match render {
        CartRenderResult::Active { text, keyboard }
        | CartRenderResult::Confirming { text, keyboard } => {
            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .await
        }
    };

    match result {
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
