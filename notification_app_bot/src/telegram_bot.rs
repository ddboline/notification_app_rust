use telegram_bot::{Api, ChatId, CanSendMessage, UserId};
use std::sync::Arc;
use anyhow::Error;
use std::collections::HashMap;
use tokio::sync::RwLock;
use lazy_static::lazy_static;
use tokio::fs;

use crate::failure_count::FailureCount;

use notification_app_lib::config::{Config, ApiTokenConfig};

type UserIds = RwLock<HashMap<UserId, Option<ChatId>>>;

lazy_static! {
    static ref TELEGRAM_USERIDS: UserIds = RwLock::new(HashMap::new());
    static ref FAILURE_COUNT: FailureCount = FailureCount::new(5);
}

pub struct TelegramBot {
    api: Arc<Api>,
    config: Config,
}

impl TelegramBot {
    pub fn new(bot_token: &str, config: &Config) -> Self {
        Self {
            api: Arc::new(Api::new(bot_token)),
            config: config.clone(),
        }
    }

    pub async fn send_message(&self, chat: ChatId, msg: &str) -> Result<(), Error> {
        self.api.spawn(chat.text(msg));
        Ok(())
    }

    async fn fill_telegram_user_ids(&self) -> Result<(), Error> {
        loop {
            FAILURE_COUNT.check()?;
            let mut modified = None;
            if let Some(api_tokens_path) = &self.config.api_tokens_path {
                if let Some(new_modified) = fs::metadata(api_tokens_path).await.ok().and_then(|metadata| {
                    metadata.modified().ok()
                }) {
                    if modified.is_none() || Some(new_modified) > modified {
                        let config = ApiTokenConfig::new(api_tokens_path).await?;
                    }
                }
            }
        }
    }
}