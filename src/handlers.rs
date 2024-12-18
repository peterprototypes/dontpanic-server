use actix_web::{web, HttpResponse};

use crate::Error;

pub mod account;
pub mod auth;
pub mod ingress;
pub mod notifications;
pub mod organizations;
pub mod reports;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::routes));
    cfg.service(web::scope("/organizations").configure(organizations::routes));
    cfg.service(web::scope("/account").configure(account::routes));
    cfg.service(web::scope("/reports").configure(reports::routes));
    cfg.service(web::scope("/notifications").configure(notifications::routes));

    cfg.default_service(web::route().to(|| async { Err::<HttpResponse, _>(Error::new("Not Found")) }));
}
