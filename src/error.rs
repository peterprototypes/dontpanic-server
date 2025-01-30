use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use actix_session::SessionGetError;
use actix_web::{
    http::{header::ToStrError, StatusCode},
    HttpResponse,
};
use serde::Serialize;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Debug, Clone)]
pub struct ErrorMessage {
    r#type: Option<String>,
    message: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Error {
    NotFound,
    LoginRequired,
    User(ErrorMessage),
    Fields(HashMap<String, ErrorMessage>),
    #[serde(skip)]
    Internal(anyhow::Error),
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self::User(ErrorMessage {
            r#type: None,
            message: message.into(),
        })
    }

    pub fn new_with_type(r#type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::User(ErrorMessage {
            r#type: Some(r#type.into()),
            message: message.into(),
        })
    }

    pub fn field(name: &'static str, message: Cow<'static, str>) -> Self {
        let mut errors = ValidationErrors::new();

        errors.add(name, ValidationError::new("server_validation").with_message(message));

        Self::from(errors)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Not Found"),
            Self::User(msg) => write!(f, "{}", msg.message),
            Self::LoginRequired => write!(f, "Unauthorized"),
            Self::Fields(_) => write!(f, "Bad Request"),
            Self::Internal(_) => write!(f, "An internal error occurred. Please try again later."),
        }
    }
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        // log error if it is internal
        if let Self::Internal(e) = self {
            log::error!("{:?}", e);

            #[cfg(test)]
            eprintln!("Internal error during testing: {:#?} - {:?}", e, e);
        }

        let mut builder = HttpResponse::build(self.status_code());

        // since Error::Internal(anyhow::Error) cannot be serialized we need to transform it to UserMessage
        if let Self::Internal(_) = self {
            return builder.json(Self::User(ErrorMessage {
                r#type: Some("internal_server_error".into()),
                message: self.to_string(),
            }));
        }

        if let Self::NotFound = self {
            return builder.json(Self::User(ErrorMessage {
                r#type: Some("not_found".into()),
                message: "The requested resource was not found on the server.".into(),
            }));
        }

        builder.json(self)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::LoginRequired => StatusCode::UNAUTHORIZED,
            Self::User(_) => StatusCode::BAD_REQUEST,
            Self::Fields(_) => StatusCode::BAD_REQUEST,
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
                let message = errors
                    .iter()
                    .filter_map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                (
                    field.to_string(),
                    ErrorMessage {
                        r#type: Some("server".to_string()),
                        message,
                    },
                )
            })
            .collect();

        Self::Fields(field_errors)
    }
}
