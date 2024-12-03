use actix_session::Session;
use actix_web::{get, web, Responder};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, TryIntoModel};

use crate::{AppContext, Error, Result};

use crate::entity::prelude::*;
use crate::entity::users;

#[get("/verify-email/{hash}")]
async fn verify_email(
    ctx: web::Data<AppContext<'_>>,
    path: web::Path<String>,
    session: Session,
) -> Result<impl Responder> {
    let hash = path.into_inner();

    let user = Users::find()
        .filter(users::Column::EmailVerificationHash.eq(hash))
        .one(&ctx.db)
        .await?
        .ok_or(Error::new("Email verification request not found or expired."))?;

    let mut user = user.into_active_model();
    user.email_verification_hash = ActiveValue::set(None);
    user.email_verification_hash_created = ActiveValue::set(None);
    let user = user.save(&ctx.db).await?.try_into_model()?;

    //device used to register is trusted
    session.insert(format!("seen_{}", user.user_id), true)?;

    Ok(web::Json(()))
}
