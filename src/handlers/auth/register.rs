use actix_web::{post, web, Responder};
use chrono::prelude::*;
use chrono_tz::Tz;
use lettre::AsyncTransport;
use rand::distr::Alphanumeric;
use rand::Rng;
use sea_orm::{prelude::*, ActiveValue, Condition, TryIntoModel};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::{AppContext, Error, Result};

use crate::entity::organization_invitations;
use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::users;

#[derive(Clone, Debug, Deserialize, Validate)]
struct RegistrationRequest {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: String,
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    password: String,
    name: Option<String>,
    company: Option<String>,
    iana_timezone_name: Option<String>,
    invite_slug: Option<String>,
}

#[post("/register")]
async fn register(ctx: web::Data<AppContext<'static>>, form: web::Json<RegistrationRequest>) -> Result<impl Responder> {
    if !ctx.config.registration_enabled {
        return Err(Error::new("Registration is disabled"));
    }

    form.validate()?;

    let user_search = Users::find()
        .filter(users::Column::Email.eq(&form.email))
        .one(&ctx.db)
        .await?;

    if user_search.is_some() {
        return Err(Error::new("Account already exists. Please login instead."));
    }

    if let Err(e) = create_user(ctx.clone(), form.into_inner()).await {
        return Err(Error::new(e.to_string()));
    }

    Ok(web::Json(json!({
        "require_email_verification": ctx.config.require_email_verification
    })))
}

async fn create_user(ctx: web::Data<AppContext<'_>>, data: RegistrationRequest) -> anyhow::Result<()> {
    let timezone_name = data
        .iana_timezone_name
        .filter(|tz_name| tz_name.parse::<Tz>().is_ok())
        .unwrap_or_else(|| ctx.config.default_user_timezone.to_string());

    let hashed_password = bcrypt::hash(&data.password, bcrypt::DEFAULT_COST)?;

    let name = data.name.filter(|s| !s.is_empty());

    let (email_verification_hash, email_verification_created) = if ctx.config.require_email_verification {
        let hash: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        (Some(hash), Some(Utc::now().naive_utc()))
    } else {
        (None, None)
    };

    let user = users::ActiveModel {
        email: ActiveValue::set(data.email.clone()),
        password: ActiveValue::set(hashed_password.into_bytes()),
        name: ActiveValue::set(name),
        email_verification_hash: ActiveValue::set(email_verification_hash.clone()),
        email_verification_hash_created: ActiveValue::set(email_verification_created),
        iana_timezone_name: ActiveValue::set(timezone_name),
        ..Default::default()
    };

    let user = user.insert(&ctx.db).await?.try_into_model()?;

    // get invitations
    let invitations = OrganizationInvitations::find()
        .filter(
            Condition::any()
                .add(organization_invitations::Column::Email.eq(&data.email))
                .add_option(
                    data.invite_slug
                        .map(|slug| organization_invitations::Column::Slug.eq(slug)),
                ),
        )
        .all(&ctx.db)
        .await?;

    // create organization if user is not invited to any
    if invitations.is_empty() {
        let company = data.company.filter(|s| !s.is_empty());

        let requests_limit = ctx.config.organization_requests_limit;

        let organization = organizations::ActiveModel {
            name: ActiveValue::set(company.unwrap_or(String::from("Default Organization"))),
            requests_limit: ActiveValue::set(requests_limit),
            requests_count_start: ActiveValue::set(requests_limit.map(|_| Utc::now().naive_utc())),
            is_enabled: ActiveValue::set(1),
            ..Default::default()
        };

        let organization = organization.insert(&ctx.db).await?.try_into_model()?;

        let organization_member = organization_users::ActiveModel {
            organization_id: ActiveValue::set(organization.organization_id),
            user_id: ActiveValue::set(user.user_id),
            role: ActiveValue::set("owner".to_string()),
            ..Default::default()
        };

        organization_member.insert(&ctx.db).await?;
    }

    // accept invitations
    for invitation in invitations {
        let organization_member = organization_users::ActiveModel {
            organization_id: ActiveValue::set(invitation.organization_id),
            user_id: ActiveValue::set(user.user_id),
            role: ActiveValue::set(invitation.role.clone()),
            ..Default::default()
        };

        organization_member.insert(&ctx.db).await?;
        invitation.delete(&ctx.db).await?;

        // TODO: send invitation accepted email to the person who invited this user
    }

    if let Some(email_verification_hash) = email_verification_hash {
        // email verification is not disabled

        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(data.email.parse()?)
            .subject("Please confirm your e-mail address")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/confirmation",
                &serde_json::json!({
                    "hash": email_verification_hash,
                    "base_url": ctx.config.base_url,
                    "scheme": ctx.config.scheme,
                    "title": "Please confirm your e-mail address"
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            mailer.send(email).await?;
        } else {
            return Err(Error::new("Email sending is not configured").into());
        }
    }

    Ok(())
}
