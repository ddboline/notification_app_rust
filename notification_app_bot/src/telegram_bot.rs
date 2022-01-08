use anyhow::{format_err, Error};
use deadqueue::unlimited::Queue;
use futures::try_join;
use lazy_static::lazy_static;
use log::error;
use std::{collections::HashMap, sync::Arc};
use telegram_bot::{
    Api, CanReplySendMessage, CanSendMessage, ChatId, ChatRef, MessageKind, ToChatRef, UpdateKind,
    UserId,
};
use tokio::{
    fs,
    sync::RwLock,
    time::{self, timeout},
};
use tokio_stream::StreamExt;
use stack_string::format_sstr;
use std::fmt::Write;

use crate::failure_count::FailureCount;

use notification_app_lib::config::{ApiTokenConfig, Config, TelegramMessage};

type UserIds = RwLock<HashMap<UserId, Option<ChatId>>>;

lazy_static! {
    static ref TELEGRAM_USERIDS: UserIds = RwLock::new(HashMap::new());
    static ref API_TOKEN_CONFIG: RwLock<ApiTokenConfig> = RwLock::new(ApiTokenConfig::default());
    static ref FAILURE_COUNT: FailureCount = FailureCount::new(5);
}

pub struct TelegramBot {
    api: Arc<Api>,
    config: Config,
    queue: Arc<Queue<TelegramMessage>>,
}

impl TelegramBot {
    pub fn new(bot_token: &str, config: &Config, queue: Arc<Queue<TelegramMessage>>) -> Self {
        Self {
            api: Arc::new(Api::new(bot_token)),
            config: config.clone(),
            queue,
        }
    }

    pub async fn run(&self) -> Result<(), Error> {
        let fill_task = self.fill_telegram_user_ids();
        let notification_task = self.notification_handler();
        let bot_task = self.telegram_worker();
        try_join!(fill_task, notification_task, bot_task).map(|_| ())
    }

    pub async fn send_message(&self, chat: ChatId, msg: &str) -> Result<(), Error> {
        self.api.spawn(chat.text(msg));
        Ok(())
    }

    async fn telegram_worker(&self) -> Result<(), Error> {
        loop {
            FAILURE_COUNT.check()?;
            match timeout(time::Duration::from_secs(3600), self.bot_handler()).await {
                Ok(Ok(_)) | Err(_) => FAILURE_COUNT.reset()?,
                Ok(Err(_)) => FAILURE_COUNT.increment()?,
            }
        }
    }

    async fn bot_handler(&self) -> Result<(), Error> {
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            FAILURE_COUNT.check()?;
            if let UpdateKind::Message(message) = update?.kind {
                FAILURE_COUNT.check()?;
                if let MessageKind::Text { ref data, .. } = message.kind {
                    FAILURE_COUNT.check()?;
                    if TELEGRAM_USERIDS.read().await.contains_key(&message.from.id) {
                        FAILURE_COUNT.check()?;
                        if let ChatRef::Id(chat_id) = message.chat.to_chat_ref() {
                            if data.starts_with("/init") {
                                self.update_telegram_chat_id(message.from.id, chat_id)
                                    .await?;
                                let reply = format_sstr!(
                                    "Initializing chat_id {}",
                                    chat_id
                                );
                                self.api
                                    .send(
                                        message.text_reply(reply.as_str()),
                                    )
                                    .await?;
                            } else if TELEGRAM_USERIDS
                                .read()
                                .await
                                .get(&message.from.id)
                                .unwrap()
                                .is_none()
                            {
                                self.api
                                    .send(message.text_reply(
                                        "No chatid set, please entry '/init' to initialize",
                                    ))
                                    .await?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn notification_handler(&self) -> Result<(), Error> {
        loop {
            FAILURE_COUNT.check()?;
            match timeout(time::Duration::from_secs(3600), self.queue.pop()).await {
                Ok(message) => {
                    FAILURE_COUNT.reset()?;
                    match self.process_message(&message).await {
                        Ok(_) => FAILURE_COUNT.reset()?,
                        Err(e) => {
                            error!("{}", e);
                            FAILURE_COUNT.increment()?;
                        }
                    }
                }
                Err(_) => FAILURE_COUNT.increment()?,
            }
        }
    }

    async fn process_message(&self, message: &TelegramMessage) -> Result<(), Error> {
        if let Some(entry) = API_TOKEN_CONFIG
            .read()
            .await
            .get(message.recipient.as_str())
        {
            if let Some(userid) = entry.telegram_userid {
                let userid = UserId::new(userid);
                if let Some(Some(chatid)) = TELEGRAM_USERIDS.read().await.get(&userid) {
                    self.send_message(*chatid, message.message.as_str()).await?;
                }
            }
        }
        Ok(())
    }

    async fn fill_telegram_user_ids(&self) -> Result<(), Error> {
        loop {
            FAILURE_COUNT.check()?;
            let mut modified = None;
            if let Some(api_tokens_path) = &self.config.api_tokens_path {
                if let Some(new_modified) = fs::metadata(api_tokens_path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                {
                    let old_modified = modified.replace(new_modified);
                    if old_modified.is_none() || modified > old_modified {
                        let config = ApiTokenConfig::new(api_tokens_path).await?;
                        let userid_map = Self::get_userid_chatid_dict(&config);
                        *TELEGRAM_USERIDS.write().await = userid_map;
                        *API_TOKEN_CONFIG.write().await = config;
                    }
                }
            }
            time::sleep(time::Duration::from_secs(1)).await;
        }
    }

    fn get_userid_chatid_dict(api_config: &ApiTokenConfig) -> HashMap<UserId, Option<ChatId>> {
        api_config
            .values()
            .filter_map(|entry| {
                entry
                    .telegram_userid
                    .map(UserId::new)
                    .map(|userid| (userid, entry.telegram_chatid.map(ChatId::new)))
            })
            .collect()
    }

    async fn update_telegram_chat_id(&self, userid: UserId, chatid: ChatId) -> Result<(), Error> {
        let api_tokens_path = self
            .config
            .api_tokens_path
            .as_ref()
            .ok_or_else(|| format_err!("No API_TOKENS_PATH"))?;
        let mut config = ApiTokenConfig::new(api_tokens_path).await?;
        config.add_chatid(userid.into(), chatid.into())?;
        let userid_map = Self::get_userid_chatid_dict(&config);
        *TELEGRAM_USERIDS.write().await = userid_map;
        *API_TOKEN_CONFIG.write().await = config;
        Ok(())
    }
}
