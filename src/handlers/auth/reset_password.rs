use actix_web::{post, web, Responder};
use anyhow::anyhow;
use chrono::{prelude::*, TimeDelta};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use validator::Validate;

use crate::{AppContext, Error, Result};

use crate::entity::prelude::*;
use crate::entity::users;

#[derive(Clone, Debug, Deserialize, Validate)]
struct PasswordResetForm {
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    new_password: String,
    #[validate(
        must_match(other = "new_password", message = "Password do not match"),
        length(min = 8, message = "Must be at least 8 characters long")
    )]
    new_password_repeat: String,
}

#[post("/password-reset/{hash}")]
async fn reset_password(
    ctx: web::Data<AppContext<'_>>,
    path: web::Path<String>,
    form: web::Json<PasswordResetForm>,
) -> Result<impl Responder> {
    form.validate()?;

    let hash = path.into_inner();

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

    let hashed_password = bcrypt::hash(&form.new_password, bcrypt::DEFAULT_COST)?;

    let mut user = user.into_active_model();
    user.password = ActiveValue::set(hashed_password.into_bytes());
    user.password_reset_hash = ActiveValue::set(None);
    user.password_reset_hash_created = ActiveValue::set(None);
    user.save(&ctx.db).await?;

    Ok(web::Json(()))
}
