use axum::http::{header::CONTENT_TYPE, StatusCode};
use deadqueue::unlimited::Queue;
use log::debug;
use stack_string::{format_sstr, StackString};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::spawn};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use notification_app_bot::telegram_bot::TelegramBot;
use notification_app_lib::config::{ApiTokenConfig, Config, TelegramMessage};

use crate::{
    errors::ServiceError as Error,
    routes::{notify_telegram_router, ApiDoc},
};

#[derive(Clone)]
pub struct AppState {
    pub queue: Arc<Queue<TelegramMessage>>,
    pub api_tokens: Arc<HashSet<StackString>>,
}

/// # Errors
/// Returns error if app initialization fails
pub async fn start_app() -> Result<(), Error> {
    let config = Config::init_config()?;
    let queue = Arc::new(Queue::new());
    let api_tokens_path = config
        .api_tokens_path
        .as_ref()
        .ok_or_else(|| Error::BadRequest(format_sstr!("No api token path set")))?;
    let api_tokens = Arc::new(ApiTokenConfig::new(api_tokens_path).await?.api_tokens());

    let telegram_bot_token = config
        .telegram_bot_token
        .as_ref()
        .ok_or_else(|| Error::BadRequest(format_sstr!("No Telegram Token")))?;
    let bot = TelegramBot::new(telegram_bot_token.as_str(), &config, queue.clone());
    let bot = spawn(async move { bot.run().await });

    let app = AppState { queue, api_tokens };

    run_api(app, config.port).await?;
    bot.await??;
    Ok(())
}

async fn run_api(app: AppState, port: u32) -> Result<(), Error> {
    let app = Arc::new(app);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(notify_telegram_router(&app))
        .split_for_parts();

    let spec_json = serde_json::to_string_pretty(&api)?;
    let spec_yaml = serde_yml::to_string(&api)?;

    let router = router
        .route(
            "/notify/openapi/json",
            axum::routing::get(|| async move {
                (
                    StatusCode::OK,
                    [(CONTENT_TYPE, mime::APPLICATION_JSON.essence_str())],
                    spec_json,
                )
            }),
        )
        .route(
            "/notify/openapi/yaml",
            axum::routing::get(|| async move {
                (StatusCode::OK, [(CONTENT_TYPE, "text/yaml")], spec_yaml)
            }),
        );

    let addr: SocketAddr = format_sstr!("127.0.0.1:{port}").parse()?;
    debug!("{addr:?}");
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use anyhow::Error;
    use axum::http::{header::AUTHORIZATION, StatusCode};
    use deadqueue::unlimited::Queue;
    use maplit::{hashmap, hashset};
    use stack_string::format_sstr;
    use std::sync::Arc;

    use crate::app::{run_api, AppState};

    #[tokio::test]
    async fn test_run_app() -> Result<(), Error> {
        let api_tokens = Arc::new(hashset! {"12345".into()});
        let queue = Arc::new(Queue::new());
        let app = {
            let queue = queue.clone();
            AppState { queue, api_tokens }
        };

        let test_port = 12345;
        tokio::task::spawn({
            async move {
                env_logger::init();
                run_api(app, test_port).await.unwrap()
            }
        });
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let client = reqwest::Client::new();
        let data = hashmap! {
            "recipient" => "ddboline",
            "message" => "test message",
        };

        let url = format_sstr!("http://localhost:{test_port}/notify");
        let response = client
            .post(url.as_str())
            .header(AUTHORIZATION, "Bearer 12345")
            .json(&data)
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(response.status(), StatusCode::CREATED);
        let text = response.text().await?;
        assert_eq!(text, "message sent");

        let url = format_sstr!("http://localhost:{test_port}/notify/openapi/yaml");
        let spec_yaml = client
            .get(url.as_str())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        tokio::fs::write("../scripts/openapi.yaml", &spec_yaml).await?;

        while let Some(message) = queue.try_pop() {
            assert_eq!(message.recipient, "ddboline");
            assert_eq!(message.message, "test message");
            println!("{message:?}");
        }
        Ok(())
    }
}
