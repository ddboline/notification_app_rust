use actix_web::{middleware::Compress, web, App, HttpServer};
use anyhow::{format_err, Error};
use lazy_static::lazy_static;
use stack_string::StackString;
use std::{collections::HashSet, sync::Arc};

use notification_app_lib::config::{ApiTokenConfig, Config};

use crate::routes::notify_telegram;

lazy_static! {
    pub static ref CONFIG: Config = Config::init_config().expect("Failed to load config");
}

pub struct AppState {
    pub api_tokens: Arc<HashSet<StackString>>,
}

pub async fn start_app() -> Result<(), Error> {
    let api_tokens_path = CONFIG
        .api_tokens_path
        .as_ref()
        .ok_or_else(|| format_err!("No api token path set"))?;
    let api_tokens = Arc::new(ApiTokenConfig::new(api_tokens_path).await?.api_tokens());

    let port = CONFIG.port;

    HttpServer::new(move || {
        App::new()
            .data(AppState {
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
