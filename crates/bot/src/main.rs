mod context;
mod handlers;


mod router;
mod features;

use context::BotContext;
use teloxide::prelude::*;
use teloxide::dispatching::UpdateFilterExt;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let ctx = BotContext::new().await.expect("Context init failed");

    log::info!("🤖 FastOrder bot started");

    //  message routing
    let message_handler =
        Update::filter_message().endpoint(router::message::handle_message);

    //  callback routing
    let callback_handler =
        Update::filter_callback_query().endpoint(router::callback::handle_callback);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ctx])
        .build()
        .dispatch()
        .await;
}
