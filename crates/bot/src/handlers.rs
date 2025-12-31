use app::users::error::AuthError;
use teloxide::prelude::*;
use teloxide::types::KeyboardRemove;

use crate::callbacks::cart::handle_cart_action;
use crate::context::BotContext;
use crate::keyboards::{order_entry_keyboard, request_phone_keyboard};
use crate::views::cart_view::{render_cart_by_state, CartRenderResult};

use app::users::phone::normalize_phone;

const WELCOME_TEXT: &str = "👋 خوش اومدی به FastOrder!\nسفارش سریع، بدون تماس تلفنی.";

pub async fn handle_message(bot: Bot, msg: Message, ctx: BotContext) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    /* ─────────────────────────────
     * /start
     * ───────────────────────────── */
    if msg.text() == Some("/start") {
        bot.send_message(chat_id, WELCOME_TEXT).await?;

        let telegram_id = match msg.from.as_ref() {
            Some(u) => u.id.0 as i64,
            None => return Ok(()),
        };

        match ctx.user_service.is_verified_customer(telegram_id).await {
            Ok(true) => {
                bot.send_message(chat_id, "برای شروع سفارش، روی «🛒 سفارش جدید» بزن 👇")
                    .reply_markup(order_entry_keyboard())
                    .await?;
            }

            Ok(false) => {
                bot.send_message(
                    chat_id,
                    "برای استفاده از سرویس، لطفاً شماره تلفن خودت رو ارسال کن 👇",
                )
                .reply_markup(request_phone_keyboard())
                .await?;
            }

            Err(err) => {
                log::error!("start auth check failed: {:?}", err);
                bot.send_message(chat_id, "❌ خطایی رخ داد").await?;
            }
        }

        return Ok(());
    }

    /* ─────────────────────────────
     * Contact (احراز هویت)
     * ───────────────────────────── */
    if let Some(contact) = msg.contact() {
        let phone = match normalize_phone(&contact.phone_number) {
            Some(p) => p,
            None => {
                bot.send_message(chat_id, "❌ شماره تلفن نامعتبر است")
                    .await?;
                return Ok(());
            }
        };

        let telegram_id = match msg.from.as_ref() {
            Some(u) => u.id.0 as i64,
            None => return Ok(()),
        };

        let username = msg.from.as_ref().and_then(|u| u.username.as_deref());
        let full_name = msg.from.as_ref().map(|u| u.first_name.as_str());

        match ctx
            .user_service
            .verify_contact_as_customer(telegram_id, username, full_name, &phone)
            .await
        {
            Ok(user) => {
                bot.send_message(chat_id, "✅ احراز هویت با موفقیت انجام شد")
                    .reply_markup(KeyboardRemove::new())
                    .await?;

                // مستقیم وارد منو شو
                send_menu(&bot, &ctx, chat_id, user.id).await?;
            }

            Err(AuthError::PhoneNotRegistered) => {
                bot.send_message(
                    chat_id,
                    "❌ شماره شما در سیستم ثبت نشده است.\n\
                     لطفاً با رستوران تماس بگیرید.",
                )
                .reply_markup(KeyboardRemove::new())
                .await?;
            }

            Err(AuthError::InvalidPhone) => {
                bot.send_message(chat_id, "❌ شماره تلفن نامعتبر است")
                    .reply_markup(KeyboardRemove::new())
                    .await?;
            }

            Err(err) => {
                log::error!("contact auth failed: {:?}", err);
                bot.send_message(chat_id, "❌ خطا در احراز هویت").await?;
            }
        }

        return Ok(());
    }

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
                bot.send_message(chat_id, "❌ ابتدا باید احراز هویت شوید")
                    .reply_markup(request_phone_keyboard())
                    .await?;
                return Ok(());
            }
        };

        send_menu(&bot, &ctx, chat_id, user.id).await?;
        return Ok(());
    }

    Ok(())
}

async fn send_menu(
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

    match render_cart_by_state(&ctx, user_id).await {
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

pub async fn handle_callback(bot: Bot, q: CallbackQuery, ctx: BotContext) -> ResponseResult<()> {
    if let Err(err) = handle_cart_action(bot, ctx, q).await {
        log::error!("callback error: {:?}", err);
    }

    respond(())
}
