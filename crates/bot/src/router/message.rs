use teloxide::prelude::*;

use crate::{context::BotContext, handlers};

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    ctx: BotContext,
) -> Result<(), teloxide::RequestError> {
    handlers::handle_message(bot, msg, ctx).await
}
