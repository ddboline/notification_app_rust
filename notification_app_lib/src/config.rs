use anyhow::{format_err, Error};
use derive_more::{Deref, FromStr, Into};
use serde::{Deserialize, Serialize};
use stack_string::StackString;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use url::Url;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ConfigInner {
    pub telegram_bot_token: Option<StackString>,
    pub remote_url: Option<UrlWrapper>,
    pub remote_token: Option<StackString>,
    pub api_tokens_path: Option<PathBuf>,
    pub sending_email_address: Option<StackString>,
    #[serde(default = "default_port")]
    pub port: u32,
}

fn default_port() -> u32 {
    4083
}

#[derive(Serialize, Deserialize, Clone, Debug, Into, PartialEq, Deref, FromStr)]
#[serde(into = "String", try_from = "String")]
pub struct UrlWrapper(Url);

impl From<UrlWrapper> for String {
    fn from(item: UrlWrapper) -> String {
        item.0.into()
    }
}

impl TryFrom<&str> for UrlWrapper {
    type Error = Error;
    fn try_from(item: &str) -> Result<Self, Self::Error> {
        item.parse().map_err(Into::into)
    }
}

impl TryFrom<String> for UrlWrapper {
    type Error = Error;
    fn try_from(item: String) -> Result<Self, Self::Error> {
        item.parse().map_err(Into::into)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Config(Arc<ConfigInner>);

impl Config {
    /// # Errors
    /// Return error if deserializing environment variables fails
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

#[derive(Debug, Default)]
pub struct ApiTokenConfig(HashMap<StackString, ApiTokenEntry>);

impl ApiTokenConfig {
    /// # Errors
    /// Return error if parsing toml fails
    pub async fn new(p: &Path) -> Result<Self, Error> {
        let data = fs::read_to_string(p).await?;
        let config: HashMap<String, ApiTokenEntry> = toml::from_str(&data)?;
        Ok(Self(
            config.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        ))
    }

    #[must_use]
    pub fn api_tokens(&self) -> HashSet<StackString> {
        self.0
            .values()
            .filter_map(|token| token.api_token.clone())
            .collect()
    }

    /// # Errors
    /// Return error if userid not found
    pub fn add_chatid(&mut self, userid: i64, chatid: i64) -> Result<(), Error> {
        for entry in self.0.values_mut() {
            if entry.telegram_userid == Some(userid) {
                entry.telegram_chatid = Some(chatid);
                return Ok(());
            }
        }
        Err(format_err!("Userid not found"))
    }
}

impl Deref for ApiTokenConfig {
    type Target = HashMap<StackString, ApiTokenEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ApiTokenEntry {
    pub email: Option<StackString>,
    pub telegram_userid: Option<i64>,
    pub telegram_chatid: Option<i64>,
    pub api_token: Option<StackString>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TelegramMessage {
    pub recipient: StackString,
    pub message: StackString,
}

#[cfg(test)]
mod tests {
    use anyhow::Error;
    use std::{env::var_os, io::Write};
    use tempfile::NamedTempFile;

    use crate::config::{ApiTokenConfig, Config};

    #[test]
    fn test_config() -> Result<(), Error> {
        let config = Config::init_config()?;

        let api_tokens_path = var_os("API_TOKENS_PATH").unwrap();
        assert_eq!(
            api_tokens_path,
            config.api_tokens_path.as_ref().unwrap().as_os_str()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_api_token_config() -> Result<(), Error> {
        let mut temp = NamedTempFile::new()?;
        let data = include_str!("../../tests/data/test_api_tokens.toml");
        temp.write_all(data.as_bytes())?;
        let config = ApiTokenConfig::new(temp.path()).await?;

        let api_tokens = config.get("user").unwrap();

        assert_eq!(
            api_tokens.email.as_ref().unwrap().as_str(),
            "user@localhost"
        );
        assert_eq!(api_tokens.telegram_userid, Some(8675309));
        assert_eq!(api_tokens.telegram_chatid, Some(8675310));

        Ok(())
    }
}
