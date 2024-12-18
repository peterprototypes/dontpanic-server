use actix_web::{get, post, web, Responder};
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entity::organization_users;
use crate::entity::prelude::*;
use crate::entity::users;

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get)
        .service(update)
        .service(delete)
        .service(update_password);
}

#[derive(Clone, Debug, Serialize, Validate)]
struct AccountResponse {
    user_id: u32,
    email: String,
    name: Option<String>,
    iana_timezone_name: String,
    created: DateTime,
}

impl From<users::Model> for AccountResponse {
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

#[get("")]
async fn get(ctx: web::Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;
    Ok(web::Json(AccountResponse::from(user)))
}

#[derive(Serialize, Deserialize, Clone)]
struct InfoInput {
    name: Option<String>,
}

#[post("")]
async fn update(ctx: web::Data<AppContext<'_>>, id: Identity, input: web::Json<InfoInput>) -> Result<impl Responder> {
    let mut user = id.user(&ctx).await?.into_active_model();
    user.name = ActiveValue::set(input.into_inner().name.filter(|s| !s.is_empty()));
    let user = user.save(&ctx.db).await?.try_into_model()?;

    Ok(web::Json(AccountResponse::from(user)))
}

#[post("/delete")]
async fn delete(ctx: web::Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;

    let user_owned_organizations = user
        .find_related(OrganizationUsers)
        .filter(organization_users::Column::Role.eq("owner"))
        .all(&ctx.db)
        .await?;

    for user_org in user_owned_organizations {
        let org_id = user_org.organization_id;

        let owner_count = OrganizationUsers::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::Role.eq("owner"))
            .count(&ctx.db)
            .await?;

        if owner_count == 1 {
            return Err(Error::new(
                "Cannot delete account because you are the only owner of an organization.",
            ));
        }
    }

    OrganizationUsers::delete_many()
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .exec(&ctx.db)
        .await?;

    user.delete(&ctx.db).await?;
    id.logout();

    Ok(web::Json(()))
}

#[derive(Clone, Deserialize, Validate)]
struct PasswordUpdate {
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    old_password: String,
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    new_password: String,
    #[validate(
        must_match(other = "new_password", message = "Password do not match"),
        length(min = 8, message = "Must be at least 8 characters long")
    )]
    new_password_repeat: String,
}

#[post("/update-password")]
async fn update_password(
    ctx: web::Data<AppContext<'_>>,
    id: Identity,
    input: web::Json<PasswordUpdate>,
) -> Result<impl Responder> {
    input.validate()?;

    let user = id.user(&ctx).await?;
    let password_hash = std::str::from_utf8(&user.password)?;

    if !bcrypt::verify(&input.old_password, password_hash)? {
        return Err(Error::field("old_password", "Password is incorrect".into()));
    }

    let hashed_password = bcrypt::hash(&input.new_password, bcrypt::DEFAULT_COST)?;

    let mut user_model = user.into_active_model();
    user_model.password = ActiveValue::set(hashed_password.into_bytes());
    user_model.save(&ctx.db).await?;

    Ok(web::Json(()))
}
