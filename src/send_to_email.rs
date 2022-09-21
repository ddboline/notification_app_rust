use anyhow::{format_err, Error};
use stack_string::{format_sstr, StackString};
use structopt::StructOpt;

use notification_app_lib::{config::Config, ses_client::SesInstance};

#[derive(StructOpt)]
struct SendToEmailOpts {
    #[structopt(short, long)]
    email: StackString,
    #[structopt(short, long)]
    message: StackString,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = SendToEmailOpts::from_args();
    let config = Config::init_config()?;
    tokio::spawn(async move {
        let src_email = config
            .sending_email_address
            .as_ref()
            .ok_or_else(|| format_err!("No sending email address"))?;
        let ses = SesInstance::new(None);
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
