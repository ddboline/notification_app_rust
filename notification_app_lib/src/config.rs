use stack_string::StackString;
use serde::{Serialize, Deserialize};
use url::Url;
use derive_more::{Into, Deref, FromStr};
use std::convert::TryFrom;
use anyhow::Error;

#[derive(Debug, Deserialize)]
pub struct ConfigInner {
    pub telegram_bot_token: Option<StackString>,
    pub remote_url: Option<UrlWrapper>,
    pub remote_token: Option<StackString>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Into, PartialEq, Deref, FromStr)]
#[serde(into = "String", try_from = "&str")]
pub struct UrlWrapper(Url);

impl From<UrlWrapper> for String {
    fn from(item: UrlWrapper) -> String {
        item.0.into_string()
    }
}

impl TryFrom<&str> for UrlWrapper {
    type Error = Error;
    fn try_from(item: &str) -> Result<Self, Self::Error> {
        item.parse().map_err(Into::into)
    }
}
