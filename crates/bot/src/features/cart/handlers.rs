use teloxide::prelude::*;
use teloxide::types::KeyboardRemove;

use crate::context::BotContext;
use crate::features::cart::views::{render_cart_by_state, CartRenderResult};

/// هندل پیام‌های مربوط به سبد خرید (فقط مشتری احراز هویت‌شده)
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    ctx: BotContext,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    /* ─────────────────────────────
     * 🛒 سفارش جدید
     * ───────────────────────────── */
    if msg.text() == Some("🛒 سفارش جدید") {
        let telegram_id = match msg.from.as_ref() {
            Some(u) => u.id.0 as i64,
            None => return Ok(()),
        };

        let user = match ctx
            .user_service
            .get_verified_user_by_telegram(telegram_id)
            .await
        {
            Ok(u) => u,
            Err(_) => {
                // اگر به اینجا برسد یعنی guard بالاتر درست کار نکرده
                bot.send_message(chat_id, "❌ ابتدا باید احراز هویت شوید")
                    .await?;
                return Ok(());
            }
        };

        send_menu(&bot, &ctx, chat_id, user.id).await?;
        return Ok(());
    }

    Ok(())
}

/// نمایش منو و وضعیت فعلی سبد خرید
pub async fn send_menu(
    bot: &Bot,
    ctx: &BotContext,
    chat_id: ChatId,
    user_id: uuid::Uuid,
) -> ResponseResult<()> {
    bot.send_message(
        chat_id,
        "👇 منو اینجاست\n\
         با دکمه‌های + و − انتخاب کن\n\
         سفارشت بالا، به‌صورت لحظه‌ای نشون داده می‌شه\n\
         وقتی تموم شدی، روی «✅ تکمیل سفارش» بزن",
    )
    .reply_markup(KeyboardRemove::new())
    .await?;

    match render_cart_by_state(ctx, user_id).await {
        Ok(render) => match render {
            CartRenderResult::Active { text, keyboard } => {
                bot.send_message(chat_id, text)
                    .reply_markup(keyboard)
                    .await?;
            }

            CartRenderResult::Confirming { text, keyboard } => {
                bot.send_message(chat_id, text)
                    .reply_markup(keyboard)
                    .await?;
            }
        },

        Err(err) => {
            log::error!("render_cart_by_state failed: {:?}", err);
            bot.send_message(chat_id, "❌ خطا در نمایش سبد خرید")
                .await?;
        }
    }

    Ok(())
}
