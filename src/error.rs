use std::collections::HashMap;
use std::fmt;

use actix_session::SessionGetError;
use actix_web::{
    http::{header::ToStrError, StatusCode},
    HttpResponse,
};
use serde_json::json;

use crate::ApiResponse;

#[derive(Debug)]
pub enum Error {
    NotFound,
    LoginRequired,
    UserMessage(String),
    FieldErrors(HashMap<String, String>),
    Internal(anyhow::Error),
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self::UserMessage(message.into())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Not Found"),
            Self::UserMessage(msg) => write!(f, "{}", msg),
            Self::LoginRequired => write!(f, "Unauthorized"),
            Self::FieldErrors(_) => write!(f, "Bad Request"),
            Self::Internal(_) => write!(f, "An internal error occurred. Please try again later."),
        }
    }
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let response = match self {
            Self::NotFound => ApiResponse::<()>::error("Not Found".to_string()),
            Self::UserMessage(msg) => ApiResponse::error(msg.clone()),
            Self::LoginRequired => ApiResponse::error("Unauthorized".to_string()),
            Self::FieldErrors(errors) => ApiResponse::field_errors(errors.clone()),
            Self::Internal(_) => ApiResponse::error("An internal error occurred. Please try again later.".to_string()),
        };

        let res = HttpResponse::build(self.status_code()).json(response);

        if let Self::Internal(e) = self {
            log::error!("{:?}", e);
        }

        res
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::LoginRequired => StatusCode::UNAUTHORIZED,
            Self::UserMessage(_) => StatusCode::BAD_REQUEST,
            Self::FieldErrors(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(value: bcrypt::BcryptError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Internal(value.into())
    }
}

impl From<handlebars::RenderError> for Error {
    fn from(value: handlebars::RenderError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<actix_session::SessionInsertError> for Error {
    fn from(value: actix_session::SessionInsertError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<ToStrError> for Error {
    fn from(value: ToStrError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<SessionGetError> for Error {
    fn from(value: SessionGetError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<lettre::transport::smtp::Error> for Error {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<lettre::error::Error> for Error {
    fn from(value: lettre::error::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<lettre::address::AddressError> for Error {
    fn from(value: lettre::address::AddressError) -> Self {
        Self::Internal(value.into())
    }
}

impl<T: Send + Sync + 'static> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Internal(value.into())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(value: validator::ValidationErrors) -> Self {
        let field_errors = value
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let errors = errors.iter().filter_map(|e| e.message.clone()).map(|e| e.clone()).collect::<Vec<_>>().join(", ");

                (field.to_string(), errors)
            })
            .collect();

        Self::FieldErrors(field_errors)
    }
}
