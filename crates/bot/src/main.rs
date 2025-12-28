use anyhow::Result;
use db::pool::create_pool;
use teloxide::prelude::*;
use teloxide::update_listeners::polling_default;

mod context;
use context::BotContext;

mod handlers;
use handlers::handle_message;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let pool = create_pool().await?;

    let ctx = BotContext { db: pool };

    log::info!("🤖 FastOrder bot started");

    let handler = Update::filter_message().endpoint(handle_message);

    let listener = polling_default(bot.clone()).await;

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ctx])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("polling error"),
        )
        .await;
    
    Ok(())
}
