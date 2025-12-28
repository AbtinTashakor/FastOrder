use teloxide::prelude::*;
use crate::context::BotContext;

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    ctx: BotContext,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if text == "/start" {
            bot.send_message(
                msg.chat.id,
                "👋 Welcome to FastOrder\nPlease share your phone number to continue."
            )
            .await?;
        }
    }

    Ok(())
}
