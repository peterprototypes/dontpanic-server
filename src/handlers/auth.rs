use actix_session::Session;
use actix_web::{get, web};
use actix_web::{HttpResponse, Responder};

use crate::Error;
use crate::Result;

mod login;
mod register;
mod request_password_reset;
mod resend_verification_email;
mod reset_password;
mod user;
mod verify_email;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register::register)
        .service(login::login)
        .service(user::user)
        .service(logout)
        .service(verify_email::verify_email)
        .service(resend_verification_email::resend_verification_email)
        .service(request_password_reset::request_password_reset)
        .service(reset_password::reset_password)
        .default_service(web::route().to(|| async { Err::<HttpResponse, _>(Error::new("Not Found")) }));
}

#[get("/logout")]
async fn logout(session: Session) -> Result<impl Responder> {
    session.remove("uid");
    Ok(web::Json(()))
}
