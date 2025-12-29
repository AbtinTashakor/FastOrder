use teloxide::prelude::*;
use teloxide::types::KeyboardRemove;
use teloxide::types::Message;

use crate::context::BotContext;
use crate::keyboards::request_phone_keyboard;
use db::customer_repo;

use crate::callbacks::cart::handle_cart_action;
use teloxide::types::CallbackQuery;


/// هندلر اصلی پیام‌ها
pub async fn handle_message(bot: Bot, msg: Message, ctx: BotContext) -> ResponseResult<()> {
    // ===== /start =====
    if let Some(text) = msg.text() {
        if text == "/start" {
            bot.send_message(
                msg.chat.id,
                "👋 به FastOrder خوش آمدید\n\n\
                 برای ادامه، لطفاً شماره تلفن خود را ارسال کنید.",
            )
            .reply_markup(request_phone_keyboard())
            .await?;

            return Ok(());
        }
    }

    // ===== دریافت Contact از تلگرام =====
    if let Some(contact) = msg.contact() {
        log::info!(
            "📱 Contact received | raw phone_number = '{}'",
            contact.phone_number
        );

        log::info!(
            "📱 Contact details | user_id = {:?}, first_name = {:?}, last_name = {:?}",
            contact.user_id,
            contact.first_name,
            contact.last_name
        );

        let phone = match normalize_phone(&contact.phone_number) {
            Some(p) => p,
            None => {
                bot.send_message(msg.chat.id, "❌ شماره تلفن نامعتبر است")
                    .await?;
                return Ok(());
            }
        };

        match customer_repo::find_by_phone(&ctx.db, &phone).await {
            // مشتری معتبر
            Ok(Some(customer)) => {
                if let Some(user) = msg.from.as_ref() {
                    if let Err(e) = customer_repo::verify_and_bind_telegram(
                        &ctx.db,
                        customer.id,
                        user.id.0 as i64,
                    )
                    .await
                    {
                        // خطای DB → لاگ، ولی بات crash نکنه
                        log::error!(
                            "Failed to bind telegram user {} to customer {}: {:?}",
                            user.id.0,
                            customer.id,
                            e
                        );
                    }
                }

                bot.send_message(
                    msg.chat.id,
                    "✅ احراز هویت انجام شد.\nمی‌توانید سفارش خود را ثبت کنید.",
                )
                .reply_markup(KeyboardRemove::new())
                .await?;
            }

            // شماره در whitelist نیست
            Ok(None) => {
                bot.send_message(
                    msg.chat.id,
                    "❌ شماره شما در لیست مشتریان ثبت نشده است.\n\
                     لطفاً با رستوران تماس بگیرید.",
                )
                .await?;
            }

            // خطای دیتابیس
            Err(e) => {
                log::error!("Database error during auth: {:?}", e);

                bot.send_message(
                    msg.chat.id,
                    "⚠️ خطای داخلی رخ داد.\nلطفاً کمی بعد دوباره تلاش کنید.",
                )
                .await?;
            }
        }

        return Ok(());
    }

    // ===== سایر پیام‌ها (فعلاً نادیده گرفته می‌شوند) =====
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
