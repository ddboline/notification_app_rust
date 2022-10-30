use anyhow::{format_err, Error};
use clap::Parser;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder,
};
use stack_string::StackString;

use notification_app_lib::config::{Config, TelegramMessage};

#[derive(Parser)]
struct SendToTelegram {
    #[clap(short, long)]
    recipient: StackString,
    #[clap(short, long)]
    message: StackString,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = SendToTelegram::parse();
    let config = Config::init_config()?;
    let payload = TelegramMessage {
        recipient: opts.recipient.clone(),
        message: opts.message.clone(),
    };
    tokio::spawn(async move {
        let url = config
            .remote_url
            .as_ref()
            .ok_or_else(|| format_err!("No remote url"))?;
        let auth_token = config
            .remote_token
            .as_ref()
            .ok_or_else(|| format_err!("No remote token"))?;
        let bearer = format!("Bearer {auth_token}");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer)?);
        let client = ClientBuilder::new().default_headers(headers).build()?;

        client
            .post(url.as_ref())
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    })
    .await
    .unwrap()
}
