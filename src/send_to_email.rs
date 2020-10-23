use anyhow::{format_err, Error};
use structopt::StructOpt;
use stack_string::StackString;

use notification_app_lib::ses_client::SesInstance;
use notification_app_lib::config::Config;

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
    let src_email = config.sending_email_address.as_ref().ok_or_else(|| format_err!("No sending email address"))?;
    let ses = SesInstance::new(None);
    let sub = format!("Notification from {}", src_email);
    ses.send_email(src_email.as_str(), opts.email.as_str(), &sub, opts.message.as_str()).await?;
    Ok(())
}
