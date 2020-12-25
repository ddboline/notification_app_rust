use actix_web::{error::ResponseError, HttpResponse};
use anyhow::Error as AnyhowError;
use serde_json::Error as SerdeJsonError;
use stack_string::StackString;
use thiserror::Error;
use tokio::io::Error as TokioIoError;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Anyhow Error {0}")]
    AnyhowError(#[from] AnyhowError),
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("BadRequest: {}", _0)]
    BadRequest(StackString),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("TokioIoError: {}", _0)]
    TokioIoError(#[from] TokioIoError),
    #[error("SerdeJsonError {}", _0)]
    SerdeJsonError(#[from] SerdeJsonError),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Self::Unauthorized => HttpResponse::Unauthorized().json("Not Authorized"),
            Self::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            _ => {
                HttpResponse::InternalServerError().json("Internal Server Error, Please try later")
            }
        }
    }
}
