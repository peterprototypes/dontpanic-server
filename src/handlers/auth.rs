use actix_session::Session;
use actix_web::{get, web, Responder};
use serde::{Deserialize, Serialize};

use crate::Result;

mod change_email;
mod login;
mod register;
mod request_password_reset;
mod resend_verification_email;
mod reset_password;
mod verify_email;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailChangePayload {
    pub id: u32,
    pub new_email: String,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register::register)
        .service(login::login)
        .service(logout)
        .service(verify_email::verify_email)
        .service(change_email::change_email)
        .service(resend_verification_email::resend_verification_email)
        .service(request_password_reset::request_password_reset)
        .service(reset_password::reset_password);
}

#[get("/logout")]
async fn logout(session: Session) -> Result<impl Responder> {
    session.remove("uid");
    Ok(web::Json(()))
}
