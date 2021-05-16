use anyhow::Error;

use notification_app_api::app::start_app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    start_app().await?;
    Ok(())
}
