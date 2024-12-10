use actix_web::{
    get, post, web,
    web::{Data, Json, Path},
    Responder,
};
use lettre::AsyncTransport;
use sea_orm::{prelude::*, ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QuerySelect, TryIntoModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::{Validate, ValidateArgs, ValidationError};

use crate::entity::organization_invitations;
use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::users;

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list)
        .service(manage)
        .service(delete)
        .service(invite)
        .service(delete_invite)
        .service(resend_invite);
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct OrganizationMember {
    user_id: u32,
    email: String,
    name: Option<String>,
    role: String,
    date_added: DateTime,
}

#[get("")]
async fn list(ctx: web::Data<AppContext<'_>>, path: Path<u32>, id: Identity) -> Result<impl Responder> {
    let organization_id = path.into_inner();

    let user = id.user(&ctx).await?;

    let organization = Organizations::find_by_id(organization_id)
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let members: Vec<OrganizationMember> = Users::find()
        .column_as(organization_users::Column::Role, "role")
        .column_as(organization_users::Column::Created, "date_added")
        .filter(organization_users::Column::OrganizationId.eq(organization.organization_id))
        .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
        .into_model::<OrganizationMember>()
        .all(&ctx.db)
        .await?;

    let invites = OrganizationInvitations::find()
        .filter(organization_invitations::Column::OrganizationId.eq(organization_id))
        .all(&ctx.db)
        .await?;

    Ok(Json(json!({
        "members": members,
        "invitations": invites,
    })))
}

#[derive(Debug, Deserialize, Validate)]
#[validate(context = String)]
struct MemberInput {
    #[validate(custom(function = "validate_role_choice", use_context))]
    role: String,
}

#[post("/{user_id}")]
async fn manage(
    ctx: Data<AppContext<'_>>,
    path: Path<(u32, u32)>,
    id: Identity,
    input: Json<MemberInput>,
) -> Result<impl Responder> {
    let input = input.into_inner();

    let (organization_id, user_id) = path.into_inner();

    let current_user = id.user(&ctx).await?;
    let user_role = current_user
        .role(&ctx.db, organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    let member = Users::find_by_id(user_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;
    let org_member = OrganizationUsers::find_by_id((member.user_id, organization_id))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    input.validate_with_args(&user_role)?;

    let mut org_member_model = org_member.into_active_model();
    org_member_model.role = ActiveValue::set(input.role);
    let org_member = org_member_model.save(&ctx.db).await?.try_into_model()?;

    Ok(Json(OrganizationMember {
        user_id: member.user_id,
        email: member.email,
        name: member.name,
        role: org_member.role,
        date_added: org_member.created,
    }))
}

#[post("/delete/{user_id}")]
async fn delete(ctx: web::Data<AppContext<'_>>, path: web::Path<(u32, u32)>, id: Identity) -> Result<impl Responder> {
    let (organization_id, user_id) = path.into_inner();

    let user = id.user(&ctx).await?;
    let user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    if user_role == "admin" || user_role == "owner" {
        let org_member = OrganizationUsers::find_by_id((user_id, organization_id))
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;

        // only owners can delete other owners
        if org_member.role == "owner" && user_role == "admin" {
            return Err(Error::new("Permission denied"));
        }

        if org_member.user_id == user.user_id {
            return Err(Error::new("You cannot delete yourself"));
        }

        org_member.delete(&ctx.db).await?;
    }

    Ok(Json(()))
}

#[derive(Debug, Deserialize, Validate)]
#[validate(context = String)]
struct InviteInput {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: String,
    #[validate(custom(function = "validate_role_choice", use_context))]
    role: String,
}

#[post("/invite")]
async fn invite(
    ctx: web::Data<AppContext<'_>>,
    path: web::Path<u32>,
    id: Identity,
    input: Json<InviteInput>,
) -> Result<impl Responder> {
    let input = input.into_inner();
    let organization_id = path.into_inner();
    let user = id.user(&ctx).await?;
    let user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    let organization = Organizations::find_by_id(organization_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    input.validate_with_args(&user_role)?;

    // check if invite exists
    let maybe_invite = OrganizationInvitations::find()
        .filter(organization_invitations::Column::Email.eq(&input.email))
        .filter(organization_invitations::Column::OrganizationId.eq(organization_id))
        .one(&ctx.db)
        .await?;

    if maybe_invite.is_some() {
        return Err(Error::field("email", "Email already invited".into()));
    }

    let maybe_user = Users::find()
        .filter(users::Column::Email.eq(&input.email))
        .one(&ctx.db)
        .await?;

    if let Some(user) = maybe_user {
        let maybe_user = OrganizationUsers::find_by_id((user.user_id, organization_id))
            .one(&ctx.db)
            .await?;

        if maybe_user.is_some() {
            return Err(Error::field(
                "email",
                "User is already a member of this organization".into(),
            ));
        }

        let org_member = organization_users::ActiveModel {
            organization_id: ActiveValue::set(organization_id),
            user_id: ActiveValue::set(user.user_id),
            role: ActiveValue::set(input.role),
            ..Default::default()
        };

        org_member.insert(&ctx.db).await?;

        let title = format!(
            "You have been added to the {} organization in Don't Panic",
            organization.name
        );

        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(user.email.parse()?)
            .subject(title.clone())
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/org_member_added",
                &serde_json::json!({
                    "base_url": ctx.config.base_url,
                    "scheme": ctx.config.scheme,
                    "organization": organization,
                    "added_by": user
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            mailer.send(email).await?;
        }
    } else {
        let org_invitation = organization_invitations::ActiveModel {
            organization_id: ActiveValue::set(organization_id),
            email: ActiveValue::set(input.email.clone()),
            role: ActiveValue::set(input.role),
            ..Default::default()
        };

        org_invitation.insert(&ctx.db).await?;

        let title = format!(
            "You have been invited to the {} organization in Don't Panic",
            organization.name
        );

        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(input.email.parse()?)
            .subject(title.clone())
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/org_invitation",
                &serde_json::json!({
                    "base_url": ctx.config.base_url,
                    "scheme": ctx.config.scheme,
                    "organization": organization,
                    "added_by": user
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            mailer.send(email).await?;
        }
    }

    Ok(Json(()))
}

#[post("/delete-invite/{invite_id}")]
async fn delete_invite(
    ctx: web::Data<AppContext<'_>>,
    path: web::Path<(u32, u32)>,
    id: Identity,
) -> Result<impl Responder> {
    let (organization_id, org_invitation_id) = path.into_inner();

    let user = id.user(&ctx).await?;
    let user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    if user_role == "admin" || user_role == "owner" {
        let invitation = OrganizationInvitations::find_by_id(org_invitation_id)
            .filter(organization_invitations::Column::OrganizationId.eq(organization_id))
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;

        invitation.delete(&ctx.db).await?;
    } else {
        return Err(Error::new("Only admins and owners can delete invites"));
    }

    Ok(Json(()))
}

#[post("/resend-invite/{email}")]
async fn resend_invite(
    ctx: web::Data<AppContext<'_>>,
    path: web::Path<(u32, String)>,
    id: Identity,
) -> Result<impl Responder> {
    let (organization_id, email) = path.into_inner();

    let organization = Organizations::find_by_id(organization_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    let invitation = OrganizationInvitations::find()
        .filter(organization_invitations::Column::OrganizationId.eq(organization_id))
        .filter(organization_invitations::Column::Email.eq(email))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let title = format!(
        "You have been invited to the {} organization in Don't Panic",
        organization.name
    );

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(invitation.email.parse()?)
        .subject(title.clone())
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(
            "email/org_invitation",
            &serde_json::json!({
                "base_url": ctx.config.base_url,
                "scheme": ctx.config.scheme,
                "organization": organization,
                "added_by": user
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    Ok(Json(()))
}

fn validate_role_choice(role: &str, user_role: &String) -> std::result::Result<(), ValidationError> {
    if user_role == "member" {
        return Err(ValidationError::new("forbidden").with_message("Only admins and owners can invite members".into()));
    }

    match user_role.as_ref() {
        "member" => {
            Err(ValidationError::new("forbidden").with_message("Only admins and owners can invite members".into()))
        }
        "admin" => match role {
            "member" | "admin" => Ok(()),
            "owner" => Err(ValidationError::new("forbidden").with_message("Only owners invite owners".into())),
            _ => Err(ValidationError::new("forbidden").with_message("Unknown role".into())),
        },
        "owner" => match role {
            "member" | "admin" | "owner" => Ok(()),
            _ => Err(ValidationError::new("forbidden").with_message("Unknown role".into())),
        },
        _ => Err(ValidationError::new("forbidden").with_message("Unknown role".into())),
    }
}