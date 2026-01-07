use teloxide::prelude::*;
use teloxide::types::KeyboardRemove;

use app::services::users::error::AuthError;
use app::services::users::phone::normalize_phone;

use crate::context::BotContext;
use crate::features::auth::views::*;

pub async fn handle_start(bot: Bot, msg: Message, ctx: BotContext) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // پیام خوش‌آمد
    bot.send_message(chat_id, WELCOME_TEXT).await?;

    // گرفتن telegram_id
    let telegram_id = match msg.from.as_ref() {
        Some(u) => u.id.0 as i64,
        None => return Ok(()),
    };

    // بررسی اینکه کاربر قبلاً احراز هویت شده یا نه
    match ctx.user_service.is_verified_customer(telegram_id).await {
        // 👌 کاربر قبلاً احراز هویت شده
        Ok(true) => {
            match ctx
                .user_service
                .get_verified_user_by_telegram(telegram_id)
                .await
            {
                Ok(user) => {
                    crate::features::customer::entry::enter(&bot, &ctx, chat_id, user.id).await?;
                }

                Err(err) => {
                    // از نظر منطقی نباید رخ بده
                    log::error!("verified user not found after check: {:?}", err);
                    bot.send_message(chat_id, "❌ خطایی رخ داد").await?;
                }
            }
        }

        // ❌ کاربر احراز هویت نشده
        Ok(false) => {
            bot.send_message(
                chat_id,
                "برای استفاده از سرویس، لطفاً شماره تلفن خودت رو ارسال کن 👇",
            )
            .reply_markup(request_phone_keyboard())
            .await?;
        }

        // خطای سیستمی
        Err(err) => {
            log::error!("start auth check failed: {:?}", err);
            bot.send_message(chat_id, "❌ خطایی رخ داد").await?;
        }
    }

    Ok(())
}

pub async fn handle_contact(bot: Bot, msg: Message, ctx: BotContext) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    let contact = match msg.contact() {
        Some(c) => c,
        None => return Ok(()),
    };

    let phone = match normalize_phone(&contact.phone_number) {
        Some(p) => p,
        None => {
            bot.send_message(chat_id, "❌ شماره تلفن نامعتبر است")
                .await?;
            return Ok(());
        }
    };

    let from = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    let telegram_id = from.id.0 as i64;
    let username = from.username.as_deref();
    let full_name = Some(from.first_name.as_str());

    match ctx
        .user_service
        .verify_contact_as_customer(telegram_id, username, full_name, &phone)
        .await
    {
        Ok(user) => {
            let name = user.full_name.as_deref().unwrap_or("");

            bot.send_message(chat_id, format!("{} عزیز خوش آمدی", name))
                .reply_markup(KeyboardRemove::new())
                .await?;

            // Call the main message handler after successful authentication
            crate::features::customer::entry::enter(&bot, &ctx, chat_id, user.id).await?;
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

        Err(err) => {
            log::error!("auth failed: {:?}", err);
            bot.send_message(chat_id, "❌ خطا در احراز هویت").await?;
        }
    }

    Ok(())
}
