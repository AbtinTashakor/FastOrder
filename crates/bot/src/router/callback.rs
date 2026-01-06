use teloxide::prelude::*;

use crate::{
    context::BotContext,
    features::cart,
    router::callback_data::CallbackData,
};

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    ctx: BotContext,
) -> Result<(), teloxide::RequestError> {
    let data = match q.data.as_deref() {
        Some(d) => d,
        None => return Ok(()),
    };

    let parsed = match CallbackData::parse(data) {
        Some(p) => p,
        None => return Ok(()),
    };

    match parsed.feature.as_str() {
        "cart" => {
            if let Err(err) = cart::callbacks::handle(bot, q, ctx, parsed).await {
                log::error!("cart callback failed: {:?}", err);
            }
            Ok(())
        }

        // future:
        // "operator" => operator::callbacks::handle(...),

        _ => Ok(()),
    }
}
