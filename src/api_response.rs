use std::collections::HashMap;

use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum ApiResponse<T> {
    Data(T),
    Error(String),
    FieldErrors(HashMap<String, String>),
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self::Data(data)
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self::Error(error.into())
    }

    pub fn field_errors(errors: HashMap<String, String>) -> Self {
        Self::FieldErrors(errors)
    }
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok().content_type(ContentType::json()).body(body)
    }
}
