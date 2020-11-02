use actix_web::{error::ResponseError, HttpResponse};
use anyhow::Error as AnyhowError;
use stack_string::StackString;
use thiserror::Error;

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
