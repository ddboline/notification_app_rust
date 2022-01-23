use anyhow::{format_err, Error};
use deadqueue::unlimited::Queue;
use rweb::Filter;
use stack_string::{format_sstr, StackString};
use std::{collections::HashSet, fmt::Write, net::SocketAddr, sync::Arc};
use tokio::task::spawn;

use notification_app_bot::telegram_bot::TelegramBot;
use notification_app_lib::config::{ApiTokenConfig, Config, TelegramMessage};

use crate::{errors::error_response, routes::notify_telegram};

#[derive(Clone)]
pub struct AppState {
    pub queue: Arc<Queue<TelegramMessage>>,
    pub api_tokens: Arc<HashSet<StackString>>,
}

pub async fn start_app() -> Result<(), Error> {
    let config = Config::init_config()?;
    let queue = Arc::new(Queue::new());
    let api_tokens_path = config
        .api_tokens_path
        .as_ref()
        .ok_or_else(|| format_err!("No api token path set"))?;
    let api_tokens = Arc::new(ApiTokenConfig::new(api_tokens_path).await?.api_tokens());

    let telegram_bot_token = config
        .telegram_bot_token
        .as_ref()
        .ok_or_else(|| format_err!("No Telegram Token"))?;
    let bot = TelegramBot::new(telegram_bot_token.as_str(), &config, queue.clone());
    let bot = spawn(async move { bot.run().await });

    let port = config.port;

    let app = AppState { queue, api_tokens };

    let notify_telegram_path = notify_telegram(app.clone());

    let routes = notify_telegram_path.recover(error_response);
    let addr: SocketAddr = format_sstr!("127.0.0.1:{port}").parse()?;
    rweb::serve(routes).bind(addr).await;
    bot.await??;
    Ok(())
}
