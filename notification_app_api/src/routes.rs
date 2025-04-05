use axum::{
    extract::{FromRequestParts, Json, State},
    http::{header::AUTHORIZATION, request::Parts},
};
use stack_string::StackString;
use std::{str::FromStr, sync::Arc};
use utoipa::{OpenApi, PartialSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_helper::{html_response::HtmlResponse as HtmlBase, UtoipaResponse};

use crate::{app::AppState, errors::ServiceError as Error, TelegramMessageWrapper};

type WarpResult<T> = Result<T, Error>;

#[derive(UtoipaResponse)]
#[response(description = "Send Notification", status = "CREATED")]
#[rustfmt::skip]
struct NotifyResponse(HtmlBase::<&'static str>);

#[utoipa::path(post, path = "/notify", responses(NotifyResponse, Error))]
async fn notify_telegram(
    data: State<Arc<AppState>>,
    credentials: BearerAuth,
    payload: Json<TelegramMessageWrapper>,
) -> WarpResult<NotifyResponse> {
    if data.api_tokens.contains(credentials.token()) {
        let Json(payload) = payload;
        data.queue.push(payload.into());
        Ok(HtmlBase::new("message sent").into())
    } else {
        Err(Error::Unauthorized)
    }
}

pub fn notify_telegram_router(app: &AppState) -> OpenApiRouter {
    let app = Arc::new(app.clone());

    OpenApiRouter::new()
        .routes(routes!(notify_telegram))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Notification API",
        description = "Simple Notification Service",
    ),
    components(schemas(TelegramMessageWrapper))
)]
pub struct ApiDoc;

struct BearerAuth(StackString);

impl BearerAuth {
    #[must_use]
    fn token(&self) -> &str {
        &self.0
    }
}

impl FromStr for BearerAuth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace();
        if iter.next().unwrap_or("") == "Bearer" {
            if let Some(auth) = iter.next() {
                return Ok(Self(auth.into()));
            }
        }
        Err(Error::BadRequest("Invalid Bearer Header".into()))
    }
}

impl<S> FromRequestParts<S> for BearerAuth
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| Error::Unauthorized)?
            .to_str()?
            .parse()
    }
}
