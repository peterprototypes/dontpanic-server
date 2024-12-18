use actix_web::{post, web, HttpRequest, Responder};
use anyhow::anyhow;
use chrono::{prelude::*, TimeDelta};
use lettre::AsyncTransport;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use validator::Validate;

use crate::{AppContext, Result};

use crate::entity::prelude::*;
use crate::entity::users;

#[derive(Clone, Debug, Deserialize, Validate)]
struct ResendVerificationRequest {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: String,
}

#[post("/resend-verification-email")]
async fn resend_verification_email(
    ctx: web::Data<AppContext<'static>>,
    req: HttpRequest,
    form: web::Json<ResendVerificationRequest>,
) -> Result<impl Responder> {
    // try for constant time response, regardless if user exists or not
    actix_web::rt::spawn(async move {
        if let Err(e) = resend_verification_email_in_bg(ctx, req, form).await {
            log::warn!("Resend verification email error: {}", e);
        }
    });

    Ok(web::Json(()))
}

async fn resend_verification_email_in_bg(
    ctx: web::Data<AppContext<'_>>,
    req: HttpRequest,
    form: web::Json<ResendVerificationRequest>,
) -> anyhow::Result<()> {
    let user = Users::find()
        .filter(users::Column::Email.eq(&form.email))
        .one(&ctx.db)
        .await?
        .ok_or(anyhow!("User not found"))?;

    if user.email_verification_hash.is_none() {
        anyhow::bail!("Email is already verified");
    }

    if !ctx.config.require_email_verification {
        anyhow::bail!("Email verification is not required");
    }

    let Some(hash_created) = user.email_verification_hash_created else {
        anyhow::bail!("User {} has no email_verification_hash_created date", user.user_id);
    };

    if Utc::now() - hash_created.and_utc() < TimeDelta::seconds(50) {
        anyhow::bail!("Attempt to resent email verification without respecting time limit");
    }

    let hash: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let created = Utc::now().naive_utc();

    let mut user = user.into_active_model();
    user.email_verification_hash = ActiveValue::set(Some(hash.clone()));
    user.email_verification_hash_created = ActiveValue::set(Some(created));
    user.save(&ctx.db).await?;

    let scheme = {
        let conn_info = req.connection_info();
        conn_info.scheme().to_string()
    };

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(form.email.parse()?)
        .subject("Please confirm your e-mail address")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(
            "email/confirmation",
            &serde_json::json!({
                "hash": hash,
                "base_url": ctx.config.base_url,
                "scheme": scheme,
                "title": "Please confirm your e-mail address"
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    Ok(())
}
