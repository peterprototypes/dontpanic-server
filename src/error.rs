use std::fmt;

use actix_htmx::Htmx;
use actix_session::SessionGetError;
use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    http::{
        header::{ContentType, ToStrError, LOCATION},
        StatusCode,
    },
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web, HttpMessage, HttpResponse,
};

use crate::AppContext;

#[derive(Debug)]
pub enum Error {
    NotFound,
    LoginRequired,
    UserMessage(String),
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
            Self::Internal(_) => write!(f, "An internal error occurred. Please try again later."),
        }
    }
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let mut res = HttpResponse::build(self.status_code());
        res.insert_header(ContentType::html());

        if let Self::LoginRequired = self {
            res.insert_header((LOCATION, "/login"));
        }

        if let Self::Internal(e) = self {
            log::error!("{:?}", e);
        }

        res.body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::LoginRequired => StatusCode::TEMPORARY_REDIRECT,
            Self::UserMessage(_) => StatusCode::BAD_REQUEST,
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

// Custom error handlers, to return HTML responses when an error occurs.
pub fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, not_found_handler)
        .default_handler(default_error_handler)
}

fn not_found_handler<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(res.into_parts().0, response.map_into_left_body())))
}

fn default_error_handler<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<BoxBody>> {
    let msg = res
        .response()
        .error()
        .map(|e| e.to_string())
        .unwrap_or_else(|| String::from("An error occurred. Please try again later."));

    let response = get_error_response(&res, &msg);
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(res.into_parts().0, response.map_into_left_body())))
}

fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse<BoxBody> {
    let request = res.request();

    if request.content_type() == ContentType::json().to_string() {
        return HttpResponse::build(res.status())
            .content_type(ContentType::json())
            .body(serde_json::json!({"error": error}).to_string());
    }

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |err: &str| HttpResponse::build(res.status()).content_type(ContentType::plaintext()).body(err.to_string());

    let ctx = request.app_data::<web::Data<AppContext<'_>>>();

    match ctx {
        Some(ctx) => {
            let is_htmx = request.extensions().get::<Htmx>().map(|htmx| htmx.is_htmx).unwrap_or_default();

            let data = serde_json::json!({
                "error": error,
                "status_code": res.status().as_str(),
                "layout": "layout_auth",
                "is_htmx": is_htmx
            });

            let body = ctx.hb.render("error", &data);

            match body {
                Ok(body) => HttpResponse::build(res.status()).content_type(ContentType::html()).body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
