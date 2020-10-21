use stack_string::StackString;
use serde::{Serialize, Deserialize};
use url::Url;
use derive_more::{Into, Deref, FromStr};
use std::convert::TryFrom;
use anyhow::{format_err, Error};
use std::path::{PathBuf, Path};
use std::sync::Arc;
use std::ops::Deref;

#[derive(Default, Debug, Deserialize)]
pub struct ConfigInner {
    pub telegram_bot_token: Option<StackString>,
    pub remote_url: Option<UrlWrapper>,
    pub remote_token: Option<StackString>,
    pub api_tokens_path: Option<PathBuf>,
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

#[derive(Default, Debug, Clone)]
pub struct Config(Arc<ConfigInner>);

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_config() -> Result<Self, Error> {
        let fname = Path::new("config.env");
        let config_dir = dirs::config_dir().ok_or_else(|| format_err!("No CONFIG directory"))?;
        let default_fname = config_dir.join("notification_app_rust").join("config.env");

        let env_file = if fname.exists() {
            fname
        } else {
            &default_fname
        };

        dotenv::dotenv().ok();

        if env_file.exists() {
            dotenv::from_path(env_file).ok();
        }

        let conf: ConfigInner = envy::from_env()?;

        Ok(Self(Arc::new(conf)))
    }
}

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
