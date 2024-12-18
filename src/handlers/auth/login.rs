use actix_session::Session;
use actix_web::{http, post, web, HttpRequest, Responder};
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use serde::Deserialize;
use validator::Validate;

use crate::{AppContext, Error, Result};

use crate::entity::prelude::*;
use crate::entity::users;

#[derive(Clone, Debug, Deserialize, Validate)]
struct LoginRequest {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: String,
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    password: String,
}

#[post("/login")]
pub async fn login(
    ctx: web::Data<AppContext<'_>>,
    req: HttpRequest,
    session: Session,
    form: web::Json<LoginRequest>,
) -> Result<impl Responder> {
    form.validate()?;

    let user = Users::find()
        .filter(users::Column::Email.eq(&form.email))
        .one(&ctx.db)
        .await?;

    let Some(user) = user else {
        let _ = bcrypt::hash("I want this else branch to take as much time", bcrypt::DEFAULT_COST);

        return Err(Error::new("Login failed; Invalid email or password."));
    };

    let password_hash = std::str::from_utf8(&user.password)?;

    if !bcrypt::verify(&form.password, password_hash)? {
        return Err(Error::new("Login failed; Invalid email or password."));
    }

    if ctx.config.require_email_verification && user.email_verification_hash.is_some() {
        return Err(Error::new_with_type(
            "email_unverified",
            "Your email is not yet verified.",
        ));
    }

    session.insert("uid", user.user_id)?;

    if session.get::<bool>(&format!("seen_{}", user.user_id))?.is_none() {
        let ip = {
            let conn_info = req.connection_info();
            conn_info.realip_remote_addr().map(|s| s.to_string())
        };

        let user_agent = if let Some(value) = req.headers().get(http::header::USER_AGENT) {
            value.to_str()?.to_owned()
        } else {
            "Unknown".into()
        };

        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(user.email.parse()?)
            .subject("Security Alert: Sign-in from a new device")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/new_login",
                &serde_json::json!({
                    "ip": ip,
                    "user_agent": user_agent,
                    "title": "Security Alert: Sign-in from a new device"
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            if let Err(e) = mailer.send(email).await {
                log::error!("Error sending sign-in from a new device email: {:?}", e);
            }
        }
    }

    session.insert(format!("seen_{}", user.user_id), true)?;

    Ok(web::Json(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;

    #[actix_web::test]
    async fn test_login() {
        let ctx = crate::AppContext::testing().await.unwrap();
        let app = test::init_service(App::new().app_data(web::Data::new(ctx)).service(login)).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({
                "email": "testing@dontpanic.rs",
                "password": "password"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // let res: serde_json::Value = test::read_body_json(resp).await;
        // assert!(res.is_object());
    }
}
