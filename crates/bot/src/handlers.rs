use teloxide::prelude::*;
use teloxide::types::Message;

use crate::context::BotContext;
use crate::keyboards::request_phone_keyboard;
use db::customer_repo;

/// هندلر اصلی پیام‌ها
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    ctx: BotContext,
) -> ResponseResult<()> {
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
        let phone = normalize_phone(&contact.phone_number);

        match customer_repo::find_by_phone(&ctx.db, &phone).await {
            // مشتری معتبر
            Ok(Some(customer)) => {
                if let Some(user) = msg.from() {
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

/// نرمال‌سازی شماره تلفن
fn normalize_phone(raw: &str) -> String {
    raw.replace(' ', "")
        .replace('-', "")
        .replace('(', "")
        .replace(')', "")
        .trim()
        .to_string()
}
