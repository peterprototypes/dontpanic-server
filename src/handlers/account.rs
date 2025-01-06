use actix_web::{
    get, post,
    web::{self, Data, Json},
    Responder,
};
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entity::organization_users;
use crate::entity::prelude::*;
use crate::entity::users;

use crate::handlers::auth::EmailChangePayload;
use crate::{AppContext, Error, Identity, Result};

mod totp;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get)
        .service(update)
        .service(update_email)
        .service(delete)
        .service(update_password)
        .service(web::scope("/totp").configure(totp::routes));
}

#[derive(Clone, Debug, Serialize, Validate)]
struct AccountResponse {
    user_id: u32,
    email: String,
    name: Option<String>,
    iana_timezone_name: String,
    totp_enabled: bool,
    created: DateTime,
}

impl From<users::Model> for AccountResponse {
    fn from(value: users::Model) -> Self {
        Self {
            user_id: value.user_id,
            email: value.email,
            name: value.name,
            iana_timezone_name: value.iana_timezone_name,
            totp_enabled: value.totp_secret.is_some(),
            created: value.created,
        }
    }
}

#[get("")]
async fn get(ctx: Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;
    Ok(Json(AccountResponse::from(user)))
}

#[derive(Serialize, Deserialize, Clone)]
struct InfoInput {
    name: Option<String>,
}

#[post("")]
async fn update(ctx: Data<AppContext<'_>>, id: Identity, input: Json<InfoInput>) -> Result<impl Responder> {
    let mut user = id.user(&ctx).await?.into_active_model();
    user.name = ActiveValue::set(input.into_inner().name.filter(|s| !s.is_empty()));
    let user = user.save(&ctx.db).await?.try_into_model()?;

    Ok(Json(AccountResponse::from(user)))
}

#[derive(Clone, Deserialize, Validate)]
struct EmailUpdate {
    #[validate(email(message = "Invalid email address"))]
    new_email: String,
}

#[post("/update-email")]
async fn update_email(ctx: Data<AppContext<'_>>, id: Identity, input: Json<EmailUpdate>) -> Result<impl Responder> {
    input.validate()?;

    let user = id.user(&ctx).await?;

    // Instead of saving the new email in a special database field or passing it as a clear text param,
    // we will store it in an encoded cookie. This way in the confirmation endpoint we can be
    // sure that it was the email the user entered here. This can be done with a JWT too, but
    // why add the extra dependency if we can do it with a cookie.

    let payload = EmailChangePayload {
        id: user.user_id,
        new_email: input.new_email.clone(),
    };

    let key = actix_web::cookie::Key::from(&ctx.config.cookie_secret);
    let cookie = actix_web::cookie::Cookie::new("payload", serde_json::to_string(&payload)?);
    let mut jar = actix_web::cookie::CookieJar::new();
    jar.private_mut(&key).add(cookie);
    let cookie = jar.delta().next().unwrap();

    let encoded = &cookie.encoded().to_string();

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(input.new_email.parse()?)
        .subject("Security Alert: Email Change Requested")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(
            "email/change-email",
            &serde_json::json!({
                "title": "Security Alert: Email Change Requested",
                "payload": encoded,
                "base_url": ctx.config.base_url,
                "scheme": ctx.config.scheme,
                "old_email": user.email,
                "new_email": input.new_email,
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        if let Err(e) = mailer.send(email).await {
            log::error!("Error sending email change request email: {:?}", e);
        }
    } else {
        return Err(Error::new("Email sending is not configured"));
    }

    Ok(Json(()))
}

#[post("/delete")]
async fn delete(ctx: Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
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

    Ok(Json(()))
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
    ctx: Data<AppContext<'_>>,
    id: Identity,
    input: Json<PasswordUpdate>,
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

    Ok(Json(()))
}
