use anyhow::Error;

use notification_app_api::app::start_app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tokio::spawn(async move { start_app().await })
        .await
        .unwrap()
}
