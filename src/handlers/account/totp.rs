use actix_web::{
    get, post,
    web::{self, Data, Json},
    Responder,
};
use google_authenticator::GoogleAuthenticator;
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(secret).service(enable).service(disable);
}

#[get("/secret")]
async fn secret(ctx: Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;

    if user.totp_secret.is_some() {
        return Err(Error::new("Two-factor authentication already enabled"));
    }

    let ga = GoogleAuthenticator::new();

    let secret = ga.create_secret(32);

    let name = urlencoding::encode(&user.email);
    let title = urlencoding::encode("Don't Panic!");
    let url = format!("otpauth://totp/{}?secret={}&issuer={}", name, secret, title);

    Ok(Json(json!({
        "url": url,
        "secret": secret
    })))
}

#[derive(Deserialize, Debug, Validate)]
struct TotpEnable {
    #[validate(length(equal = 32, message = "Invalid secret provided"))]
    secret: String,
    #[validate(length(equal = 6, message = "Invalid code provided"))]
    code: String,
}

#[post("/enable")]
async fn enable(ctx: Data<AppContext<'_>>, id: Identity, input: Json<TotpEnable>) -> Result<impl Responder> {
    let input = input.into_inner();
    input.validate()?;

    let user = id.user(&ctx).await?;

    if user.totp_secret.is_some() {
        return Err(Error::new("Two-factor authentication already enabled"));
    }

    let ga = GoogleAuthenticator::new();

    if !ga.verify_code(&input.secret, &input.code, 30, 0) {
        return Err(Error::field("code", "Invalid code provided".into()));
    }

    let mut user = user.into_active_model();
    user.totp_secret = ActiveValue::set(Some(input.secret));
    user.save(&ctx.db).await?;

    Ok(Json(()))
}

#[post("/disable")]
async fn disable(ctx: Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    let mut user = id.user(&ctx).await?.into_active_model();
    user.totp_secret = ActiveValue::set(None);
    user.save(&ctx.db).await?;

    Ok(Json(()))
}
