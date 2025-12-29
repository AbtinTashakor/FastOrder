mod callbacks;
mod context;
mod handlers;
mod keyboards;
mod views;

use context::BotContext;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    use crate::handlers::{handle_callback, handle_message};
    use teloxide::dispatching::UpdateFilterExt;

    let bot = Bot::from_env();
    let ctx = BotContext::new().await.expect("Context init failed");

    log::info!("🤖 FastOrder bot started");

    let message_handler = Update::filter_message().endpoint(handle_message);

    let callback_handler = Update::filter_callback_query().endpoint(handle_callback);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ctx])
        .build()
        .dispatch()
        .await;
}
