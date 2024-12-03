use actix_web::{get, web, Responder};
use sea_orm::prelude::*;
use serde::Serialize;
use validator::Validate;

use crate::{AppContext, Error, Identity, Result};

use crate::entity::prelude::*;
use crate::entity::users;

#[derive(Clone, Debug, Serialize, Validate)]
struct UserResponse {
    user_id: u32,
    email: String,
    name: Option<String>,
    iana_timezone_name: String,
    created: DateTime,
}

impl From<users::Model> for UserResponse {
    fn from(value: users::Model) -> Self {
        Self {
            user_id: value.user_id,
            email: value.email,
            name: value.name,
            iana_timezone_name: value.iana_timezone_name,
            created: value.created,
        }
    }
}

#[get("/user")]
pub async fn user(ctx: web::Data<AppContext<'_>>, identity: Identity) -> Result<impl Responder> {
    let user = Users::find_by_id(identity.user_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::LoginRequired)?;

    Ok(web::Json(UserResponse::from(user)))
}
