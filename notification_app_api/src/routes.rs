use rweb::{post, Json, Rejection, Reply};
use stack_string::StackString;
use std::str::FromStr;

use notification_app_lib::config::TelegramMessage;

use crate::{app::AppState, errors::ServiceError as Error};

pub type WarpResult<T> = Result<T, Rejection>;

#[post("/notify")]
pub async fn notify_telegram(
    payload: Json<TelegramMessage>,
    #[data] data: AppState,
    #[header = "authorization"] credentials: BearerAuth,
) -> WarpResult<impl Reply> {
    if data.api_tokens.contains(credentials.token()) {
        let mesg = payload.into_inner();
        data.queue.push(mesg);
        Ok(rweb::reply::html(""))
    } else {
        Err(Error::Unauthorized.into())
    }
}

struct BearerAuth(StackString);

impl BearerAuth {
    pub fn token(&self) -> &str {
        &self.0
    }
}

impl FromStr for BearerAuth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();
        if iter.next().unwrap_or_else(|| "") == "Bearer" {
            if let Some(auth) = iter.next() {
                return Ok(Self(auth.into()));
            }
        }
        Err(Error::BadRequest("Invalid Bearer Header".into()))
    }
}
