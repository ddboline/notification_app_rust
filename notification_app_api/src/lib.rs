#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::unused_async)]

pub mod app;
pub mod errors;
pub mod routes;

use rweb::Schema;
use serde::{Deserialize, Serialize};
use stack_string::StackString;

use notification_app_lib::config::TelegramMessage;

#[derive(Serialize, Deserialize, Default, Debug, Schema)]
pub struct TelegramMessageWrapper {
    pub recipient: StackString,
    pub message: StackString,
}

impl From<TelegramMessage> for TelegramMessageWrapper {
    fn from(item: TelegramMessage) -> Self {
        Self {
            recipient: item.recipient,
            message: item.message,
        }
    }
}

impl From<TelegramMessageWrapper> for TelegramMessage {
    fn from(item: TelegramMessageWrapper) -> Self {
        Self {
            recipient: item.recipient,
            message: item.message,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
