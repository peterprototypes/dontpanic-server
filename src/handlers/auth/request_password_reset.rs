use actix_web::{http, post, web, HttpRequest, Responder};
use chrono::prelude::*;
use lettre::AsyncTransport;
use rand::distr::Alphanumeric;
use rand::prelude::*;
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, TryIntoModel};
use serde::Deserialize;
use validator::Validate;

use crate::{AppContext, Error, Identity, Result};

use crate::entity::prelude::*;
use crate::entity::users;

#[derive(Clone, Debug, Deserialize, Validate)]
struct PasswordResetRequest {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: Option<String>,
}

#[post("/request-password-reset")]
pub async fn request_password_reset(
    ctx: web::Data<AppContext<'static>>,
    req: HttpRequest,
    form: web::Json<PasswordResetRequest>,
    id: Option<Identity>,
) -> Result<impl Responder> {
    form.validate()?;

    let email = if let Some(email) = form.email.clone() {
        email
    } else if let Some(id) = id {
        id.user(&ctx).await?.email.clone()
    } else {
        return Err(Error::field("email", "Email is required".into()));
    };

    // try for constant time response, regardless if user exists or not
    let pw_reset_ctx = ctx.clone();

    actix_web::rt::spawn(async move {
        if let Err(e) = password_reset_request_in_bg(pw_reset_ctx, req, email).await {
            log::error!("Password reset request error: {}", e);
        }
    });

    Ok(web::Json(()))
}

async fn password_reset_request_in_bg(ctx: web::Data<AppContext<'_>>, req: HttpRequest, email: String) -> Result<()> {
    let user = Users::find()
        .filter(users::Column::Email.eq(&email))
        .one(&ctx.db)
        .await?;

    let Some(user) = user else { return Ok(()) };

    let reset_hash: String = rand::rng()
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
    } else {
        return Err(Error::new("Email sending is not configured"));
    }

    Ok(())
}
