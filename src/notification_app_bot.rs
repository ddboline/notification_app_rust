use anyhow::{format_err, Error};

use notification_app_bot::telegram_bot::TelegramBot;
use notification_app_lib::config::Config;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::init_config()?;

    let telegram_bot_token = config
        .telegram_bot_token
        .as_ref()
        .ok_or_else(|| format_err!("No Telegram Token"))?;

    let bot = TelegramBot::new(&telegram_bot_token, &config);
    bot.run().await?;

    Ok(())
}
