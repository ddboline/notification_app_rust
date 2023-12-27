use anyhow::Error;
use aws_config::SdkConfig;
use aws_sdk_ses::{
    types::{Body, Content, Destination, Message},
    Client as SesClient,
};
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct SesInstance {
    ses_client: SesClient,
}

impl fmt::Debug for SesInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SesInstance")
    }
}

impl SesInstance {
    #[must_use]
    pub fn new(sdk_config: &SdkConfig) -> Self {
        Self::from_conf(sdk_config)
    }

    fn from_conf(config: &SdkConfig) -> Self {
        Self {
            ses_client: SesClient::new(config),
        }
    }

    /// # Errors
    /// Return error if api call fails
    pub async fn send_email(
        &self,
        src: impl Into<String>,
        dest: impl Into<String>,
        sub: impl Into<String>,
        msg: impl Into<String>,
    ) -> Result<(), Error> {
        let destination = Destination::builder()
            .set_to_addresses(Some(vec![dest.into()]))
            .build();
        let subject = Content::builder()
            .set_charset(Some("UTF-8".into()))
            .set_data(Some(sub.into()))
            .build()?;
        let html = Content::builder()
            .set_charset(Some("UTF-8".into()))
            .set_data(Some(msg.into()))
            .build()?;
        let body = Body::builder().text(html.clone()).html(html).build();
        let message = Message::builder().subject(subject).body(body).build();
        self.ses_client
            .send_email()
            .destination(destination)
            .source(src)
            .message(message)
            .send()
            .await?;
        Ok(())
    }

    /// # Errors
    /// Returns error if api call fails
    pub async fn get_statistics(&self) -> Result<(SesQuotas, EmailStats), Error> {
        let quota = self.ses_client.get_send_quota().send().await?;
        let stats = self
            .ses_client
            .get_send_statistics()
            .send()
            .await?
            .send_data_points
            .unwrap_or_default()
            .into_iter()
            .map(|point| EmailStats {
                bounces: point.bounces,
                complaints: point.complaints,
                delivery_attempts: point.delivery_attempts,
                rejects: point.rejects,
                min_timestamp: point
                    .timestamp
                    .and_then(|t| OffsetDateTime::from_unix_timestamp(t.as_secs_f64() as i64).ok()),
                ..EmailStats::default()
            })
            .fold(EmailStats::default(), |mut stats, point| {
                stats.bounces += point.bounces;
                stats.complaints += point.complaints;
                stats.delivery_attempts += point.delivery_attempts;
                stats.rejects += point.rejects;
                if let Some(timestamp) = point.min_timestamp {
                    if stats.min_timestamp.is_none() || Some(timestamp) < stats.min_timestamp {
                        stats.min_timestamp = Some(timestamp);
                    }
                    if stats.max_timestamp.is_none() || Some(timestamp) > stats.max_timestamp {
                        stats.max_timestamp = Some(timestamp);
                    }
                }
                stats
            });
        let quota = SesQuotas {
            max_24_hour_send: quota.max24_hour_send,
            max_send_rate: quota.max_send_rate,
            sent_last_24_hours: quota.sent_last24_hours,
        };
        Ok((quota, stats))
    }
}

#[derive(Default, Debug)]
pub struct SesQuotas {
    pub max_24_hour_send: f64,
    pub max_send_rate: f64,
    pub sent_last_24_hours: f64,
}

#[derive(Default, Debug)]
pub struct EmailStats {
    pub bounces: i64,
    pub complaints: i64,
    pub delivery_attempts: i64,
    pub rejects: i64,
    pub min_timestamp: Option<OffsetDateTime>,
    pub max_timestamp: Option<OffsetDateTime>,
}
