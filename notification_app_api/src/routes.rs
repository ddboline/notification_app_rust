use actix_web::{
    http::StatusCode,
    web::{Data, Json},
    HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use tokio::{io::AsyncWriteExt, net::UnixStream};

use notification_app_lib::config::TelegramMessage;

use crate::{
    app::{AppState, CONFIG},
    errors::ServiceError as Error,
};

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
        let mut sock = UnixStream::connect(&CONFIG.unix_socket).await?;
        let data = serde_json::to_string(&mesg)?;
        sock.write_all(data.as_bytes()).await?;
        form_http_response(data)
    } else {
        Err(Error::Unauthorized)
    }
}
