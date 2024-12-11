use actix_web::Responder;
use actix_web::{delete, get, post, route, web};
use chrono::{Days, Utc};
use lettre::AsyncTransport;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use sea_orm::{prelude::*, ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QuerySelect, TryIntoModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use validator::ValidateArgs;
use validator::ValidationError;
use validator::ValidationErrors;

use crate::entity::organization_invitations;
use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::project_user_settings;

use crate::entity::users;

use crate::{AppContext, Error, Identity, Result};

mod projects;
use projects::OrganizationProject;

mod members;
use members::OrganizationMember;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list)
        .service(create)
        .service(web::scope("/{organization_id}/projects").configure(projects::routes))
        .service(web::scope("/{organization_id}/members").configure(members::routes));

    // cfg.service(organization)
    //     .service(org_create)
    //     .service(org_delete)
    //     .service(org_settings)
    //     .service(org_notifications)
    //     .service(org_members)
    //     .service(org_invite)
    //     .service(org_invite_delete)
    //     .service(org_invite_resend)
    //     .service(org_member_edit)
    //     .service(org_member_delete);
}

#[derive(Serialize, Debug)]
struct Organization {
    organization_id: u32,
    name: String,
    requests_limit: Option<u32>,
    requests_count: Option<u32>,
    requests_count_start: Option<DateTime>,
    is_enabled: i8,
    created: DateTime,
    projects: Vec<OrganizationProject>,
    members: Vec<OrganizationMember>,
}

#[get("")]
async fn list(ctx: web::Data<AppContext<'_>>, identity: Identity) -> Result<impl Responder> {
    let user = identity.user(&ctx).await?;

    let orgs_and_projects: Vec<(organizations::Model, Vec<crate::entity::projects::Model>)> = Organizations::find()
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .find_with_related(Projects)
        .all(&ctx.db)
        .await?;

    let mut response = vec![];

    for (org, projects) in orgs_and_projects {
        let projects: Vec<OrganizationProject> = projects.into_iter().map(OrganizationProject::from).collect();

        let members: Vec<OrganizationMember> = Users::find()
            .column_as(organization_users::Column::Role, "role")
            .column_as(organization_users::Column::Created, "date_added")
            .column_as(organization_users::Column::OrganizationId, "organization_id")
            .filter(organization_users::Column::OrganizationId.eq(org.organization_id))
            .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
            .into_model::<OrganizationMember>()
            .all(&ctx.db)
            .await?;

        let org = Organization {
            organization_id: org.organization_id,
            name: org.name,
            requests_limit: org.requests_limit,
            requests_count: org.requests_count,
            requests_count_start: org.requests_count_start,
            is_enabled: org.is_enabled,
            created: org.created,
            projects,
            members,
        };

        response.push(org);
    }

    Ok(web::Json(response))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateInput {
    #[validate(length(min = 1, max = 80, message = "Organization name is required"))]
    name: String,
}

#[post("")]
async fn create(ctx: web::Data<AppContext<'_>>, id: Identity, input: web::Json<CreateInput>) -> Result<impl Responder> {
    input.validate()?;

    let name = input.name.trim().to_string();

    let maybe_org = Organizations::find()
        .filter(organizations::Column::Name.eq(&name))
        .filter(organization_users::Column::UserId.eq(id.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?;

    if maybe_org.is_some() {
        return Err(Error::field(
            "name",
            "An organization with the same name already exists.".into(),
        ));
    }

    let requests_limit = ctx.config.organization_requests_limit;

    let org = organizations::ActiveModel {
        name: ActiveValue::set(name),
        requests_limit: ActiveValue::set(requests_limit),
        requests_count_start: ActiveValue::set(requests_limit.map(|_| Utc::now().naive_utc())),
        is_enabled: ActiveValue::set(1),
        ..Default::default()
    };

    let org = org.insert(&ctx.db).await?.try_into_model()?;

    let organization_member = organization_users::ActiveModel {
        organization_id: ActiveValue::set(org.organization_id),
        user_id: ActiveValue::set(id.user_id),
        role: ActiveValue::set("owner".to_string()),
        ..Default::default()
    };

    organization_member.insert(&ctx.db).await?;

    Ok(web::Json(json!({
        "organization_id": org.organization_id,
    })))
}

// #[delete("/organization/{organization_id}")]
// async fn org_delete(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
//     let mut view = ViewModel::default();

//     let org_id = path.into_inner();
//     view.set("org_id", org_id);

//     let user = identity.user(&ctx).await?;
//     let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

//     if user_role != "owner" {
//         return Err(Error::new("Only owners can delete an organization"));
//     }

//     Organizations::delete(&ctx.db, org_id).await?;

//     view.redirect("/reports", true);

//     Ok(view)
// }

// #[derive(Clone, Debug, Serialize, Deserialize, Validate)]
// struct OrgSettingsFrom {
//     #[validate(length(min = 1, max = 80, message = "Organization name is required"))]
//     name: String,
// }

// #[route("/organization/{organization_id}/settings", method = "GET", method = "POST")]
// async fn org_settings(
//     ctx: web::Data<AppContext<'_>>,
//     identity: Identity,
//     path: web::Path<u32>,
//     form: Option<web::Form<OrgSettingsFrom>>,
// ) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("organizations/settings");

//     let org_id = path.into_inner();
//     let form = form.map(|f| f.into_inner());

//     let user = identity.user(&ctx).await?;
//     view.set("user", &user);

//     let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;
//     view.set("role", &user_role);

//     view.set("org_id", org_id);

//     let mut org = Organizations::find_by_id(org_id)
//         .filter(organization_users::Column::UserId.eq(identity.user_id))
//         .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
//         .one(&ctx.db)
//         .await?
//         .ok_or(Error::NotFound)?;

//     view.set(
//         "form",
//         form.clone()
//             .unwrap_or_else(|| OrgSettingsFrom { name: org.name.clone() }),
//     );

//     if let Some(fields) = form {
//         if let Err(errors) = fields.validate() {
//             view.set("errors", &errors);
//             return Ok(view);
//         }

//         let org_search = Organizations::find()
//             .filter(organizations::Column::Name.eq(&fields.name))
//             .filter(organizations::Column::OrganizationId.ne(org_id))
//             .one(&ctx.db)
//             .await?;

//         if org_search.is_some() {
//             let mut errors = ValidationErrors::new();
//             errors.add(
//                 "name",
//                 ValidationError::new("exists").with_message("An organization with the same name already exists".into()),
//             );
//             view.set("errors", errors);
//             return Ok(view);
//         }

//         let mut org_model = org.into_active_model();
//         org_model.name = ActiveValue::set(fields.name);
//         org = org_model.save(&ctx.db).await?.try_into_model()?;

//         view.message("Organization information updated");
//         view.set("saved", true);
//     }

//     let reset_date = org.requests_count_start.map(|date| date + Days::new(30));
//     view.set("limit_reset_date", reset_date);

//     view.set("organization", org);

//     Ok(view)
// }

// #[get("/organization/{organization_id}/notifications")]
// async fn org_notifications(
//     _ctx: web::Data<AppContext<'_>>,
//     _identity: Identity,
//     path: web::Path<u32>,
// ) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("organizations/notifications");

//     let organization_id = path.into_inner();
//     view.set("org_id", organization_id);

//     Ok(view)
// }

// #[derive(Serialize, Deserialize, Validate)]
// #[validate(context = String)]
// struct InviteForm {
//     #[validate(
//         email(message = "A valid email address is required"),
//         length(max = 320, message = "Must be less than 320 chars")
//     )]
//     email: String,
//     #[validate(custom(function = "validate_role_choice", use_context))]
//     role: String,
// }

// #[route("/organization/{organization_id}/invite", method = "GET", method = "POST")]
// async fn org_invite(
//     ctx: web::Data<AppContext<'_>>,
//     identity: Identity,
//     path: web::Path<u32>,
//     form: Option<web::Form<InviteForm>>,
// ) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("organizations/invite");

//     view.set("form", &form);

//     let org_id = path.into_inner();
//     view.set("org_id", org_id);

//     let user = identity.user(&ctx).await?;
//     let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;
//     view.set("role", &user_role);

//     let org = Organizations::find_by_id(org_id)
//         .one(&ctx.db)
//         .await?
//         .ok_or(Error::NotFound)?;

//     if let Some(fields) = form.map(|f| f.into_inner()) {
//         if let Err(errors) = fields.validate_with_args(&user_role) {
//             view.set("errors", &errors);
//             return Ok(view);
//         }

//         // check if invite exists
//         let maybe_invite = OrganizationInvitations::find()
//             .filter(organization_invitations::Column::Email.eq(&fields.email))
//             .filter(organization_invitations::Column::OrganizationId.eq(org_id))
//             .one(&ctx.db)
//             .await?;

//         if let Some(invitation) = maybe_invite {
//             let mut errors = ValidationErrors::new();
//             errors.add(
//                 "email",
//                 ValidationError::new("exists").with_message("Email already invited".into()),
//             );
//             view.set("errors", errors);

//             view.set("existing_invitation_id", invitation.organization_invitation_id);

//             return Ok(view);
//         }

//         let maybe_user = Users::find()
//             .filter(users::Column::Email.eq(&fields.email))
//             .one(&ctx.db)
//             .await?;

//         if let Some(user) = maybe_user {
//             let maybe_user = OrganizationUsers::find_by_id((user.user_id, org_id))
//                 .one(&ctx.db)
//                 .await?;

//             if maybe_user.is_some() {
//                 let mut errors = ValidationErrors::new();
//                 errors.add(
//                     "email",
//                     ValidationError::new("exists").with_message("User is already a member".into()),
//                 );
//                 view.set("errors", errors);
//                 return Ok(view);
//             }

//             let org_member = organization_users::ActiveModel {
//                 organization_id: ActiveValue::set(org_id),
//                 user_id: ActiveValue::set(user.user_id),
//                 role: ActiveValue::set(fields.role),
//                 ..Default::default()
//             };

//             org_member.insert(&ctx.db).await?;

//             let title = format!("You have been added to the {} organization in Don't Panic", org.name);

//             let email = lettre::Message::builder()
//                 .from(ctx.config.email_from.clone().into())
//                 .to(user.email.parse()?)
//                 .subject(title.clone())
//                 .header(lettre::message::header::ContentType::TEXT_HTML)
//                 .body(ctx.hb.render(
//                     "email/org_member_added",
//                     &serde_json::json!({
//                         "base_url": ctx.config.base_url,
//                         "scheme": ctx.config.scheme,
//                         "organization": org,
//                         "added_by": user
//                     }),
//                 )?)?;

//             if let Some(mailer) = ctx.mailer.as_ref() {
//                 mailer.send(email).await?;
//             }

//             view.message("Member added");
//         } else {
//             let org_invitation = organization_invitations::ActiveModel {
//                 organization_id: ActiveValue::set(org_id),
//                 email: ActiveValue::set(fields.email.clone()),
//                 role: ActiveValue::set(fields.role),
//                 ..Default::default()
//             };

//             org_invitation.insert(&ctx.db).await?;

//             let title = format!("You have been invited to the {} organization in Don't Panic", org.name);

//             let email = lettre::Message::builder()
//                 .from(ctx.config.email_from.clone().into())
//                 .to(fields.email.parse()?)
//                 .subject(title.clone())
//                 .header(lettre::message::header::ContentType::TEXT_HTML)
//                 .body(ctx.hb.render(
//                     "email/org_invitation",
//                     &serde_json::json!({
//                         "base_url": ctx.config.base_url,
//                         "scheme": ctx.config.scheme,
//                         "organization": org,
//                         "added_by": user
//                     }),
//                 )?)?;

//             if let Some(mailer) = ctx.mailer.as_ref() {
//                 mailer.send(email).await?;
//             }

//             view.message("Invitation sent");
//         }

//         view.redirect(format!("/organization/{}?tab=members", org_id), false);
//     }

//     Ok(view)
// }

// #[delete("/organization/{organization_id}/invite/{organization_invitation_id}")]
// async fn org_invite_delete(
//     ctx: web::Data<AppContext<'_>>,
//     identity: Identity,
//     path: web::Path<(u32, u32)>,
// ) -> Result<ViewModel> {
//     let mut view = ViewModel::default();

//     let (org_id, org_invitation_id) = path.into_inner();

//     let user = identity.user(&ctx).await?;
//     let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

//     if user_role == "admin" || user_role == "owner" {
//         let invitation = OrganizationInvitations::find_by_id(org_invitation_id)
//             .filter(organization_invitations::Column::OrganizationId.eq(org_id))
//             .one(&ctx.db)
//             .await?
//             .ok_or(Error::NotFound)?;
//         invitation.delete(&ctx.db).await?;

//         view.message("Invitation deleted");
//     }

//     view.redirect(format!("/organization/{}?tab=members", org_id), false);

//     Ok(view)
// }

// #[post("/organization/{organization_id}/resend-invite/{organization_invitation_id}")]
// async fn org_invite_resend(
//     ctx: web::Data<AppContext<'_>>,
//     identity: Identity,
//     path: web::Path<(u32, u32)>,
// ) -> Result<ViewModel> {
//     let mut view = ViewModel::default();

//     let (org_id, org_invitation_id) = path.into_inner();

//     let org = Organizations::find_by_id(org_id)
//         .one(&ctx.db)
//         .await?
//         .ok_or(Error::NotFound)?;

//     let user = identity.user(&ctx).await?;
//     let _user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

//     let invitation = OrganizationInvitations::find_by_id(org_invitation_id)
//         .filter(organization_invitations::Column::OrganizationId.eq(org_id))
//         .one(&ctx.db)
//         .await?
//         .ok_or(Error::NotFound)?;

//     let title = format!("You have been invited to the {} organization in Don't Panic", org.name);

//     let email = lettre::Message::builder()
//         .from(ctx.config.email_from.clone().into())
//         .to(invitation.email.parse()?)
//         .subject(title.clone())
//         .header(lettre::message::header::ContentType::TEXT_HTML)
//         .body(ctx.hb.render(
//             "email/org_invitation",
//             &serde_json::json!({
//                 "base_url": ctx.config.base_url,
//                 "scheme": ctx.config.scheme,
//                 "organization": org,
//                 "added_by": user
//             }),
//         )?)?;

//     if let Some(mailer) = ctx.mailer.as_ref() {
//         mailer.send(email).await?;
//     }

//     view.message("Invitation sent");

//     Ok(view)
// }
