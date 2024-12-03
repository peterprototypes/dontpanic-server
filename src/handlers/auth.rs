use actix_session::Session;
use actix_web::{http, HttpRequest, HttpResponse};
use actix_web::{route, web};
use anyhow::anyhow;
use chrono::{TimeDelta, Utc};
use lettre::AsyncTransport;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entity::prelude::*;
use crate::entity::users;

use crate::Result;
use crate::{AppContext, Error, ViewModel};

mod login;
mod register;
mod resend_verification_email;
mod user;
mod verify_email;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register::register)
        .service(login::login)
        .service(user::user)
        .service(logout)
        .service(verify_email::verify_email)
        .service(resend_verification_email::resend_verification_email)
        .service(request_password_reset)
        .service(reset_password)
        .default_service(web::route().to(|| async { Err::<HttpResponse, _>(Error::new("Not Found")) }));
}

#[route("/logout", method = "GET")]
async fn logout(session: Session) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    session.remove("uid");

    view.redirect("/login", true);

    Ok(view)
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct PasswordResetRequestForm {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: String,
}

#[route("/auth/password-reset-request", method = "GET", method = "POST")]
async fn request_password_reset(
    ctx: web::Data<AppContext<'static>>,
    req: HttpRequest,
    form: Option<web::Form<PasswordResetRequestForm>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template_and_layout("auth/password_reset_request", "layout_auth");

    view.set("form", &form);

    let Some(fields) = form else {
        return Ok(view);
    };

    if let Err(errors) = fields.validate() {
        view.set("errors", &errors);
        return Ok(view);
    }

    let pw_reset_ctx = ctx.clone();
    let email = fields.email.clone();

    // try for constant time response, regardless if user exists or not
    actix_web::rt::spawn(async move {
        if let Err(e) = password_reset_request_in_bg(pw_reset_ctx, req, email).await {
            log::error!("Password reset request error: {}", e);
        }
    });

    view.set("success", true);

    Ok(view)
}

async fn password_reset_request_in_bg(ctx: web::Data<AppContext<'_>>, req: HttpRequest, email: String) -> Result<()> {
    let user = Users::find()
        .filter(users::Column::Email.eq(&email))
        .one(&ctx.db)
        .await?;

    let Some(user) = user else { return Ok(()) };

    let reset_hash: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let mut user = user.into_active_model();
    user.password_reset_hash = ActiveValue::set(Some(reset_hash.clone()));
    user.password_reset_hash_created = ActiveValue::set(Some(Utc::now().naive_utc()));
    let user = user.save(&ctx.db).await?.try_into_model()?;

    let (scheme, ip) = {
        let conn_info = req.connection_info();
        (
            conn_info.scheme().to_string(),
            conn_info.realip_remote_addr().map(|s| s.to_string()),
        )
    };

    let user_agent = if let Some(value) = req.headers().get(http::header::USER_AGENT) {
        value.to_str()?.to_owned()
    } else {
        "Unknown".into()
    };

    let title = format!("Password reset for {} account", ctx.config.base_url);

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(user.email.parse()?)
        .subject(title.clone())
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(
            "email/password_reset",
            &serde_json::json!({
                "hash": reset_hash,
                "base_url": ctx.config.base_url,
                "scheme": scheme,
                "ip": ip,
                "user_agent": user_agent,
                "title": title
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct PasswordResetForm {
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    new_password: String,
    #[validate(
        must_match(other = "new_password", message = "Password do not match"),
        length(min = 8, message = "Must be at least 8 characters long")
    )]
    new_password_repeat: String,
}

#[route("/auth/password-reset/{hash}", method = "GET", method = "POST")]
async fn reset_password(
    ctx: web::Data<AppContext<'_>>,
    path: web::Path<String>,
    form: Option<web::Form<PasswordResetForm>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template_and_layout("auth/password_reset", "layout_auth");

    let hash = path.into_inner();
    view.set("hash", &hash);

    let user = Users::find()
        .filter(users::Column::PasswordResetHash.eq(hash))
        .one(&ctx.db)
        .await?
        .ok_or(Error::new(
            "Password reset link expired. Go to forgotten password and try again.",
        ))?;

    let now = Utc::now();

    let Some(hash_created) = user.password_reset_hash_created else {
        return Err(Error::Internal(anyhow!(
            "Password reset hash present, but without a creation date"
        )));
    };

    if now - hash_created.and_utc() > TimeDelta::hours(1) {
        return Err(Error::new("Password reset request not found or expired."));
    }

    // verified

    view.set("form", &form);

    let Some(fields) = form else {
        return Ok(view);
    };

    if let Err(errors) = fields.validate() {
        view.set("errors", &errors);
        return Ok(view);
    }

    // valid

    let hashed_password = bcrypt::hash(&fields.new_password, bcrypt::DEFAULT_COST)?;

    let mut user = user.into_active_model();
    user.password = ActiveValue::set(hashed_password.into_bytes());
    user.password_reset_hash = ActiveValue::set(None);
    user.password_reset_hash_created = ActiveValue::set(None);
    user.save(&ctx.db).await?;

    view.set("success", true);

    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;

    #[actix_web::test]
    async fn test_login() {
        let ctx = crate::AppContext::testing().await.unwrap();
        let app = test::init_service(App::new().app_data(web::Data::new(ctx)).configure(routes)).await;

        let req = test::TestRequest::get()
            .uri("/login")
            .set_json(json!({
                "email": "testing@dontpanic.rs",
                "password": "password"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let res: serde_json::Value = test::read_body_json(resp).await;
        assert!(res.is_object());
    }
}
