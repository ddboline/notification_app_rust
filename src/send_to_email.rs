use anyhow::{format_err, Error};
use clap::Parser;
use stack_string::{format_sstr, StackString};

use notification_app_lib::{config::Config, ses_client::SesInstance};

#[derive(Parser)]
struct SendToEmailOpts {
    #[clap(short, long)]
    email: StackString,
    #[clap(short, long)]
    message: StackString,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = SendToEmailOpts::parse();
    let config = Config::init_config()?;
    let sdk_config = aws_config::load_from_env().await;
    tokio::spawn(async move {
        let src_email = config
            .sending_email_address
            .as_ref()
            .ok_or_else(|| format_err!("No sending email address"))?;
        let ses = SesInstance::new(&sdk_config);
        let sub = format_sstr!("Notification from {src_email}");

        ses.send_email(
            src_email.as_str(),
            opts.email.as_str(),
            &sub,
            opts.message.as_str(),
        )
        .await
    })
    .await
    .unwrap()
}
