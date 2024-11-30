use actix_web::web;

pub mod account;
pub mod auth;
pub mod ingress;
pub mod menu;
pub mod notifications;
pub mod organizations;
pub mod reports;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::routes));
}
