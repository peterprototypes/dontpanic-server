use actix_htmx::Htmx;
use actix_web::{
    body::BoxBody,
    http::{
        header::{ContentType, LOCATION},
        StatusCode,
    },
    web::Data,
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::Serialize;
use serde_json::{json, to_value, value::Serializer, Value};

use crate::AppContext;

pub struct ViewModel {
    template_name: String,
    redirect_to: Option<(String, bool)>,
    toast_message: Option<String>,
    data: Value,
}

impl Default for ViewModel {
    fn default() -> Self {
        Self {
            template_name: String::new(),
            redirect_to: None,
            toast_message: None,
            data: json!({
                "layout": "layout"
            }),
        }
    }
}

impl ViewModel {
    pub fn with_template(template_name: impl Into<String>) -> Self {
        Self {
            template_name: template_name.into(),
            redirect_to: None,
            toast_message: None,
            data: json!({
                "layout": "layout"
            }),
        }
    }

    pub fn with_template_and_layout(template_name: impl Into<String>, layout_name: impl Into<String>) -> Self {
        Self {
            template_name: template_name.into(),
            redirect_to: None,
            toast_message: None,
            data: json!({
                "layout": layout_name.into()
            }),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Serialize) {
        let data = self.data.as_object_mut().unwrap();

        if let Ok(v) = value.serialize(Serializer) {
            data.insert(key.into(), v);
        }
    }

    pub fn redirect(&mut self, to: impl Into<String>, refresh: bool) {
        self.redirect_to = Some((to.into(), refresh));
    }

    pub fn message(&mut self, message: impl Into<String>) {
        self.toast_message = Some(message.into());
    }
}

impl Responder for ViewModel {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let extensions = req.extensions();

        let mut response = HttpResponse::Ok();

        response.content_type(ContentType::html());

        if let Some(toast_message) = self.toast_message {
            response.append_header(("X-toast-message", toast_message));
        }

        if let Some((redirect_url, refresh)) = self.redirect_to {
            if let Some(htmx) = extensions.get::<Htmx>() {
                if htmx.is_htmx {
                    if refresh {
                        htmx.redirect(redirect_url);
                    } else {
                        htmx.redirect_with_swap(
                            serde_json::json!({
                                "path": redirect_url,
                                "target": "main"
                            })
                            .to_string(),
                        );
                    }

                    return response.finish();
                }
            }

            return HttpResponse::TemporaryRedirect().insert_header((LOCATION, redirect_url)).finish();
        }

        if self.template_name.is_empty() {
            return response.finish();
        }

        let mut data = self.data;

        if let Some(htmx) = extensions.get::<Htmx>() {
            if let Ok(is_htmx) = to_value(htmx.is_htmx) {
                data.as_object_mut().unwrap().insert("is_htmx".to_string(), is_htmx);
            }
        }

        data.as_object_mut().unwrap().insert("version".to_string(), std::env!("CARGO_PKG_VERSION").into());

        let Some(ctx) = req.app_data::<Data<AppContext>>() else {
            response.status(StatusCode::INTERNAL_SERVER_ERROR);
            return response.body("Missing AppContext in Actix app data");
        };

        match ctx.hb.render(&self.template_name, &data) {
            Ok(body) => response.body(body),
            Err(e) => HttpResponse::from_error(crate::Error::from(e)),
        }
    }
}
