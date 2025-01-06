use actix_web::cookie::{Cookie, CookieJar, Key};
use actix_web::web::{self, Json, Query};
use actix_web::{get, Responder};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;

use crate::{AppContext, Error, Result};

use crate::entity::prelude::*;
use crate::entity::users;
use crate::handlers::auth::EmailChangePayload;

#[derive(Debug, Deserialize)]
struct ChangeEmailQuery {
    payload: String,
}

#[get("/change-email")]
async fn change_email(ctx: web::Data<AppContext<'_>>, query: Query<ChangeEmailQuery>) -> Result<impl Responder> {
    let payload = query.into_inner().payload;

    let key = Key::from(&ctx.config.cookie_secret);

    let mut jar = CookieJar::new();
    jar.add_original(Cookie::new("payload", payload.clone()));

    let Some(res) = jar.private(&key).get("payload") else {
        return Err(Error::new("Invalid email change request."));
    };

    let payload: EmailChangePayload = serde_json::from_str(res.value())?;

    // check if user with the new email exists
    let user_exists = Users::find()
        .filter(users::Column::Email.eq(&payload.new_email))
        .one(&ctx.db)
        .await?
        .is_some();

    if user_exists {
        return Err(Error::new("Another account with the same email is already registered"));
    }

    let user = Users::find_by_id(payload.id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::new("Email change request not found or expired."))?;

    let mut user = user.into_active_model();
    user.email = ActiveValue::set(payload.new_email);
    user.save(&ctx.db).await?;

    Ok(Json(()))
}
