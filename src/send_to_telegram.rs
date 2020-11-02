use anyhow::{format_err, Error};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    ClientBuilder,
};
use stack_string::StackString;
use structopt::StructOpt;

use notification_app_lib::config::{Config, TelegramMessage};

#[derive(StructOpt)]
struct SendToTelegram {
    #[structopt(short, long)]
    recipient: StackString,
    #[structopt(short, long)]
    message: StackString,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = SendToTelegram::from_args();
    let config = Config::init_config()?;
    let payload = TelegramMessage {
        recipient: opts.recipient.clone(),
        message: opts.message.clone(),
    };
    let url = config
        .remote_url
        .as_ref()
        .ok_or_else(|| format_err!("No remote url"))?;
    let auth_token = config
        .remote_token
        .as_ref()
        .ok_or_else(|| format_err!("No remote token"))?;
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(auth_token.as_str())?);
    let client = ClientBuilder::new().default_headers(headers).build()?;
    client
        .post(url.as_ref())
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
