use teloxide::prelude::*;
use teloxide::types::KeyboardRemove;

use crate::callbacks::cart::handle_cart_action;
use crate::context::BotContext;
use crate::keyboards::{order_entry_keyboard, request_phone_keyboard};
use crate::views::cart_view::render_cart_view;
use db::customer_repo;

const WELCOME_TEXT: &str = "👋 خوش اومدی به FastOrder!\nسفارش سریع، بدون تماس تلفنی.";

pub async fn handle_message(bot: Bot, msg: Message, ctx: BotContext) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    /* ─────────────────────────────
     * /start
     * ───────────────────────────── */
    if let Some(text) = msg.text() {
        if text == "/start" {
            // پیام خوش‌آمد
            bot.send_message(chat_id, WELCOME_TEXT).await?;

            let user_id = match msg.from.as_ref() {
                Some(u) => u.id.0 as i64,
                None => return Ok(()),
            };

            match customer_repo::find_by_telegram_id(&ctx.db, user_id).await {
                Ok(Some(_)) => {
                    // قبلاً احراز شده
                    bot.send_message(chat_id, "برای شروع سفارش، روی «🛒 سفارش جدید» بزن 👇")
                        .reply_markup(order_entry_keyboard())
                        .await?;
                }
                Ok(None) => {
                    // احراز نشده
                    bot.send_message(
                        chat_id,
                        "برای استفاده از سرویس، لطفاً شماره تلفن خودت رو ارسال کن 👇",
                    )
                    .reply_markup(request_phone_keyboard())
                    .await?;
                }
                Err(err) => {
                    log::error!("start lookup failed: {:?}", err);
                    bot.send_message(chat_id, "❌ خطایی رخ داد").await?;
                }
            }

            return Ok(());
        }
    }

    /* ─────────────────────────────
     * Contact (احراز هویت)
     * ───────────────────────────── */
    if let Some(contact) = msg.contact() {
        log::info!(
            "📱 Contact received | raw phone_number = '{}'",
            contact.phone_number
        );

        let phone = match normalize_phone(&contact.phone_number) {
            Some(p) => p,
            None => {
                bot.send_message(msg.chat.id, "❌ شماره تلفن نامعتبر است")
                    .await?;
                return Ok(());
            }
        };

        let user_id = match msg.from.as_ref() {
            Some(u) => u.id.0 as i64,
            None => return Ok(()),
        };

        match customer_repo::find_by_phone(&ctx.db, &phone).await {
            Ok(Some(customer)) => {
                if let Err(err) =
                    customer_repo::verify_and_bind_telegram(&ctx.db, customer.id, user_id).await
                {
                    log::error!("verify failed: {:?}", err);
                    bot.send_message(chat_id, "❌ خطا در احراز هویت").await?;
                    return Ok(());
                }

                // حذف کیبورد ارسال شماره
                bot.send_message(chat_id, "✅ احراز هویت با موفقیت انجام شد")
                    .reply_markup(KeyboardRemove::new())
                    .await?;

                // نمایش دکمه سفارش جدید
                bot.send_message(chat_id, "برای شروع سفارش، روی «🛒 سفارش جدید» بزن 👇")
                    .reply_markup(order_entry_keyboard())
                    .await?;
            }
            Ok(None) => {
                bot.send_message(chat_id, "❌ شماره شما ثبت نشده است")
                    .await?;
            }
            Err(err) => {
                log::error!("contact auth failed: {:?}", err);
                bot.send_message(chat_id, "❌ خطایی رخ داد").await?;
            }
        }

        return Ok(());
    }

    /* ─────────────────────────────
     * 🛒 سفارش جدید
     * ───────────────────────────── */
    if let Some(text) = msg.text() {
        if text == "🛒 سفارش جدید" {
            let user_id = match msg.from.as_ref() {
                Some(u) => u.id.0 as i64,
                None => return Ok(()),
            };

            let customer = match customer_repo::find_by_telegram_id(&ctx.db, user_id).await {
                Ok(Some(c)) => c,
                Ok(None) => {
                    bot.send_message(chat_id, "❌ ابتدا باید احراز هویت شوید")
                        .reply_markup(request_phone_keyboard())
                        .await?;
                    return Ok(());
                }
                Err(err) => {
                    log::error!("order entry lookup failed: {:?}", err);
                    bot.send_message(chat_id, "❌ خطایی رخ داد").await?;
                    return Ok(());
                }
            };

            // جمع‌کردن ReplyKeyboard
            bot.send_message(
                chat_id,
                "👇 منو اینجاست
با دکمه‌های + و − انتخاب کن
سفارشت بالا، به‌صورت لحظه‌ای نشون داده می‌شه
وقتی تموم شدی، روی «✅ تکمیل سفارش» بزن",
            )
            .reply_markup(KeyboardRemove::new())
            .await?;

            match render_cart_view(&ctx.db, customer.id).await {
                Ok((text, keyboard)) => {
                    bot.send_message(chat_id, text)
                        .reply_markup(keyboard)
                        .await?;
                }
                Err(err) => {
                    log::error!("render cart failed: {:?}", err);
                    bot.send_message(chat_id, "❌ خطا در نمایش منو").await?;
                }
            }

            return Ok(());
        }
    }

    Ok(())
}

/// Normalize Iranian phone numbers to +989XXXXXXXXX
fn normalize_phone(raw: &str) -> Option<String> {
    // 1) remove everything except digits
    let mut digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();

    // 2) normalize prefixes
    if digits.starts_with("0098") {
        digits = digits.trim_start_matches("0098").to_string();
    } else if digits.starts_with("98") {
        digits = digits.trim_start_matches("98").to_string();
    } else if digits.starts_with("0") {
        digits = digits.trim_start_matches('0').to_string();
    }

    // 3) after normalization we expect exactly 10 digits (9XXXXXXXXX)
    if digits.len() != 10 {
        log::warn!("invalid phone number after normalize: raw='{}'", raw);
        return None;
    }

    // 4) final canonical form
    Some(format!("+98{}", digits))
}

pub async fn handle_callback(bot: Bot, q: CallbackQuery, ctx: BotContext) -> ResponseResult<()> {
    if let Err(err) = handle_cart_action(bot, ctx, q).await {
        log::error!("callback error: {:?}", err);
    }

    respond(())
}
