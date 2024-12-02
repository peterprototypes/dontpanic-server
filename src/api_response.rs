use std::collections::HashMap;

use actix_web::{
    body::BoxBody,
    http::{header::ContentType, StatusCode},
    HttpRequest, HttpResponse, Responder,
};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct ApiError {
    pub r#type: String,
    pub message: String,
}

#[derive(Serialize, Debug, Clone)]
pub enum ApiResponse<T> {
    Data(T),
    Error(String),
    FieldErrors(HashMap<String, ApiError>),
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self::Data(data)
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self::Error(error.into())
    }

    pub fn errors(errors: HashMap<String, ApiError>) -> Self {
        Self::FieldErrors(errors)
    }
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        let status = match self {
            ApiResponse::Data(_) => StatusCode::OK,
            ApiResponse::Error(_) => StatusCode::BAD_REQUEST,
            ApiResponse::FieldErrors(_) => StatusCode::BAD_REQUEST,
        };

        HttpResponse::build(status).content_type(ContentType::json()).body(body)
    }
}
