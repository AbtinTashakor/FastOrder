use teloxide::prelude::*;

use crate::context::BotContext;

/// Global message entry point
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    ctx: BotContext,
) -> ResponseResult<()> {
    crate::features::cart::handlers::handle_message(bot, msg, ctx).await
}

