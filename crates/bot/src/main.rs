mod context;
mod handlers;

mod router;
mod features;

use context::BotContext;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;

use features::auth::handlers as auth;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let ctx = BotContext::new().await.expect("Context init failed");

    log::info!("🤖 FastOrder bot started");

    let message_handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, ctx: BotContext| async move {
            if msg.text() == Some("/start") {
                auth::handle_start(bot, msg, ctx).await?;
                return Ok(());
            }

            if msg.contact().is_some() {
                auth::handle_contact(bot, msg, ctx).await?;
                return Ok(());
            }

            router::message::handle_message(bot, msg, ctx).await
        },
    );

    let callback_handler = Update::filter_callback_query().endpoint(
        |bot: Bot, q: CallbackQuery, ctx: BotContext| async move {
            router::callback::handle_callback(bot, q, ctx).await?;
            Ok(())
        },
    );

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ctx])
        .build()
        .dispatch()
        .await;
}
