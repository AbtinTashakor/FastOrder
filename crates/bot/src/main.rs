mod context;
mod handlers;
mod keyboards;

use teloxide::prelude::*;
use handlers::handle_message;
use context::BotContext;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let ctx = BotContext::new().await.expect("Context init failed");

    log::info!("🤖 FastOrder bot started");

    teloxide::repl(bot, move |bot, msg| {
        let ctx = ctx.clone();
        async move {
            handle_message(bot, msg, ctx).await?;
            respond(())
        }
    })
    .await;
}
