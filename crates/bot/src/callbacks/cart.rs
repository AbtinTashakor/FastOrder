use anyhow::Result;
use teloxide::prelude::*;
use uuid::Uuid;

use crate::context::BotContext;
use crate::views::cart_view::render_cart_view;
use db::cart_repo;
use db::customer_repo;
use db::models::CustomerRow;

pub async fn handle_cart_action(bot: Bot, ctx: BotContext, q: CallbackQuery) -> Result<()> {
    let data = q.data.as_deref().unwrap_or("");
    let user_id = q.from.id.0 as i64;

    let customer: CustomerRow = match customer_repo::find_by_telegram_id(
        &ctx.db, // ❗ db نه pool
        user_id,
    )
    .await?
    {
        Some(c) => c,
        None => {
            log::warn!("unauthorized callback user: {}", user_id);
            return Ok(());
        }
    };

    let cart = cart_repo::get_or_create_active_cart(&ctx.db, customer.id).await?;

    if let Some(item_id) = parse_item_id(data) {
        match data.chars().nth(2) {
            Some('+') => {
                cart_repo::inc_item(&ctx.db, cart.id, item_id).await?;
            }
            Some('-') => {
                let _ = cart_repo::dec_item(&ctx.db, cart.id, item_id).await;
            }
            Some('x') => {
                cart_repo::remove_item(&ctx.db, cart.id, item_id).await?;
            }
            _ => {}
        }
    }

    let (text, keyboard) = render_cart_view(&ctx.db, customer.id).await?;

    if let Some(msg) = q.message {
        bot.edit_message_text(msg.chat().id, msg.id(), text)
            .reply_markup(keyboard)
            .await?;
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

fn parse_item_id(data: &str) -> Option<Uuid> {
    // c:+:<uuid>
    let parts: Vec<&str> = data.split(':').collect();
    if parts.len() == 3 {
        Uuid::parse_str(parts[2]).ok()
    } else {
        None
    }
}
