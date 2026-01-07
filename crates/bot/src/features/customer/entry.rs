use teloxide::prelude::*;

use crate::context::BotContext;
use crate::features::cart::handlers::send_menu;


pub async fn enter(
    bot: &Bot,
    ctx: &BotContext,
    chat_id: ChatId,
    user_id: uuid::Uuid,
) -> ResponseResult<()> {
    send_menu(bot, ctx, chat_id, user_id).await
}
