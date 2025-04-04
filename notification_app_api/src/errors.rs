use anyhow::Error as AnyhowError;
use axum::{
    extract::Json,
    http::{header::ToStrError, StatusCode},
    response::{IntoResponse, Response},
};
use log::error;
use serde::Serialize;
use serde_json::Error as SerdeJsonError;
use serde_yml::Error as YamlError;
use stack_string::{format_sstr, StackString};
use std::{fmt::Debug, net::AddrParseError};
use thiserror::Error;
use tokio::task::JoinError;
use utoipa::{
    openapi::{ContentBuilder, ResponseBuilder, ResponsesBuilder},
    IntoResponses, PartialSchema, ToSchema,
};

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("JoinError {0}")]
    JoinError(#[from] JoinError),
    #[error("ioError {0}")]
    IoError(#[from] std::io::Error),
    #[error("AddrParseError {0}")]
    AddrParseError(#[from] AddrParseError),
    #[error("ToStrError {0}")]
    ToStrError(#[from] ToStrError),
    #[error("Anyhow Error {0}")]
    AnyhowError(#[from] AnyhowError),
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("BadRequest: {0}")]
    BadRequest(StackString),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("SerdeJsonError {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error("YamlError {0}")]
    YamlError(#[from] YamlError),
}

#[derive(Serialize, ToSchema)]
struct ErrorMessage {
    message: StackString,
}

impl axum::response::IntoResponse for ErrorMessage {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorMessage {
                    message: format_sstr!("Not authorized"),
                },
            )
                .into_response(),
            Self::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, ErrorMessage { message }).into_response()
            }
            e => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorMessage {
                    message: format_sstr!("Internal Server Error: {e}"),
                },
            )
                .into_response(),
        }
    }
}

impl IntoResponses for ServiceError {
    fn responses() -> std::collections::BTreeMap<
        String,
        utoipa::openapi::RefOr<utoipa::openapi::response::Response>,
    > {
        let error_message_content = ContentBuilder::new()
            .schema(Some(ErrorMessage::schema()))
            .build();
        ResponsesBuilder::new()
            .response(
                StatusCode::UNAUTHORIZED.as_str(),
                ResponseBuilder::new()
                    .description("Not Authorized")
                    .content(
                        "text/html",
                        ContentBuilder::new().schema(Some(String::schema())).build(),
                    ),
            )
            .response(
                StatusCode::BAD_REQUEST.as_str(),
                ResponseBuilder::new()
                    .description("Bad Request")
                    .content("application/json", error_message_content.clone()),
            )
            .response(
                StatusCode::INTERNAL_SERVER_ERROR.as_str(),
                ResponseBuilder::new()
                    .description("Internal Server Error")
                    .content("application/json", error_message_content.clone()),
            )
            .build()
            .into()
    }
}

#[cfg(test)]
mod test {
    use axum::http::header::ToStrError;
    use std::{fmt::Error as FmtError, net::AddrParseError};
    use tokio::{task::JoinError, time::error::Elapsed};

    use crate::errors::ServiceError as Error;

    #[test]
    fn test_error_size() {
        println!("JoinError {}", std::mem::size_of::<JoinError>());
        println!("Elapsed {}", std::mem::size_of::<Elapsed>());
        println!("FmtError {}", std::mem::size_of::<FmtError>());
        println!("ToStrError {}", std::mem::size_of::<ToStrError>());
        println!("AddrParseError {}", std::mem::size_of::<AddrParseError>());
        println!("JoinError {}", std::mem::size_of::<JoinError>());

        assert_eq!(std::mem::size_of::<Error>(), 40);
    }
}
