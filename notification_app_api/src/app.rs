use actix_web::{middleware::Compress, web, App, HttpServer};
use anyhow::{format_err, Error};
use deadqueue::unlimited::Queue;
use stack_string::StackString;
use std::{collections::HashSet, sync::Arc};

use notification_app_bot::telegram_bot::TelegramBot;
use notification_app_lib::config::{ApiTokenConfig, Config, TelegramMessage};

use crate::routes::notify_telegram;

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
    actix_rt::spawn(async move { bot.run().await.unwrap() });

    let port = config.port;

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                queue: queue.clone(),
                api_tokens: api_tokens.clone(),
            })
            .wrap(Compress::default())
            .service(web::resource("/notify").route(web::post().to(notify_telegram)))
    })
    .bind(&format!("127.0.0.1:{}", port))?
    .run()
    .await
    .map_err(Into::into)
}
