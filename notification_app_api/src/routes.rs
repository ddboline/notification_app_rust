use actix_web::{
    http::StatusCode,
    web::{Data, Json},
    HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use notification_app_lib::config::TelegramMessage;

use crate::{app::AppState, errors::ServiceError as Error};

pub type HttpResult = Result<HttpResponse, Error>;

fn form_http_response(body: String) -> HttpResult {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body))
}

pub async fn notify_telegram(
    payload: Json<TelegramMessage>,
    data: Data<AppState>,
    credentials: BearerAuth,
) -> HttpResult {
    if data.api_tokens.contains(credentials.token()) {
        let mesg = payload.into_inner();
        data.queue.push(mesg);
        form_http_response("".to_string())
    } else {
        Err(Error::Unauthorized)
    }
}
