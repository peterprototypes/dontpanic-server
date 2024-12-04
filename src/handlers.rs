use actix_web::{web, HttpResponse};

use crate::Error;

pub mod account;
pub mod auth;
pub mod ingress;
pub mod notifications;
pub mod organizations;
pub mod reports;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(auth::routes);
    cfg.configure(organizations::routes);

    cfg.default_service(web::route().to(|| async { Err::<HttpResponse, _>(Error::new("Not Found")) }));
}
