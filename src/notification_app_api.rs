use anyhow::Error;

use notification_app_api::app::start_app;

#[actix_web::main]
async fn main() -> Result<(), Error> {
    start_app().await?;
    Ok(())
}
