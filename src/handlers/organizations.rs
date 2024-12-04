use actix_web::Responder;
use actix_web::{delete, get, post, route, web};
use chrono::{Days, Utc};
use lettre::AsyncTransport;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, JoinType, QuerySelect, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator::ValidateArgs;
use validator::ValidationError;
use validator::ValidationErrors;

use crate::entity::organization_invitations;
use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::project_user_settings;
use crate::entity::projects;

use crate::entity::users;
use crate::Identity;
use crate::Result;
use crate::ViewModel;
use crate::{AppContext, Error};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/organizations").service(list));

    // cfg.service(organization)
    //     .service(org_create)
    //     .service(org_delete)
    //     .service(org_settings)
    //     .service(org_notifications)
    //     .service(org_members)
    //     .service(org_projects)
    //     .service(org_project_create)
    //     .service(org_project_edit)
    //     .service(org_project_delete)
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
    projects: Vec<projects::Model>,
}

impl Organization {
    fn from_model(org: organizations::Model, projects: Vec<projects::Model>) -> Self {
        Self {
            organization_id: org.organization_id,
            name: org.name,
            requests_limit: org.requests_limit,
            requests_count: org.requests_count,
            requests_count_start: org.requests_count_start,
            is_enabled: org.is_enabled,
            created: org.created,
            projects,
        }
    }
}

#[get("")]
async fn list(ctx: web::Data<AppContext<'_>>, identity: Identity) -> Result<impl Responder> {
    let user = identity.user(&ctx).await?;

    let organizations: Vec<Organization> = Organizations::find()
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .find_with_related(Projects)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|(org, projects)| Organization::from_model(org, projects))
        .collect();

    Ok(web::Json(organizations))
}

#[derive(Deserialize)]
struct OrganizationQuery {
    tab: Option<String>,
}

#[get("/organization/{organization_id}")]
async fn organization(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<u32>,
    query: web::Query<OrganizationQuery>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/view");

    let user = identity.user(&ctx).await?;
    view.set("user", user);

    let organization_id = path.into_inner();
    view.set("org_id", organization_id);

    view.set(
        "active_tab",
        match query.tab.as_deref() {
            Some("projects") => "projects",
            Some("settings") => "settings",
            Some("members") => "members",
            Some("notifications") => "notifications",
            _ => "projects",
        },
    );

    Ok(view)
}

#[derive(Serialize, Deserialize, Validate)]
struct NewOrganizationForm {
    #[validate(length(min = 1, max = 80, message = "Organization name is required"))]
    name: String,
}

#[route("/create-organization", method = "GET", method = "POST")]
async fn org_create(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    form: Option<web::Form<NewOrganizationForm>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/create");

    let user = identity.user(&ctx).await?;
    view.set("user", user);

    view.set("form", &form);

    if let Some(fields) = form.map(|f| f.into_inner()) {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let name = fields.name.trim().to_string();

        let maybe_org = Organizations::find()
            .filter(organizations::Column::Name.eq(&name))
            .filter(organization_users::Column::UserId.eq(identity.user_id))
            .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
            .one(&ctx.db)
            .await?;

        if maybe_org.is_some() {
            let mut errors = ValidationErrors::new();
            errors.add(
                "name",
                ValidationError::new("exists")
                    .with_message("An organization with the same name already exists.".into()),
            );
            view.set("errors", errors);
            return Ok(view);
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
            user_id: ActiveValue::set(identity.user_id),
            role: ActiveValue::set("owner".to_string()),
            ..Default::default()
        };

        organization_member.insert(&ctx.db).await?;

        view.redirect(format!("/organization/{}?tab=projects", org.organization_id), true);
    }

    Ok(view)
}

#[delete("/organization/{organization_id}")]
async fn org_delete(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let org_id = path.into_inner();
    view.set("org_id", org_id);

    let user = identity.user(&ctx).await?;
    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

    if user_role != "owner" {
        return Err(Error::new("Only owners can delete an organization"));
    }

    Organizations::delete(&ctx.db, org_id).await?;

    view.redirect("/reports", true);

    Ok(view)
}

#[get("/organization/{organization_id}/projects")]
async fn org_projects(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/projects");

    let user = identity.user(&ctx).await?;
    view.set("user", user);

    let org_id = path.into_inner();
    view.set("org_id", org_id);

    let org = Organizations::find_by_id(org_id)
        .filter(organization_users::Column::UserId.eq(identity.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let projects = org.find_related(Projects).all(&ctx.db).await?;
    view.set("projects", projects);
    view.set("organization", org);

    Ok(view)
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct ProjectFrom {
    #[validate(length(min = 1, max = 80, message = "Project name is required"))]
    project_name: String,
}

#[route("/organization/{organization_id}/add-project", method = "GET", method = "POST")]
async fn org_project_create(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<u32>,
    form: Option<web::Form<ProjectFrom>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/project_create");

    view.set("form", &form);

    let org_id = path.into_inner();
    view.set("org_id", org_id);

    let org = Organizations::find_by_id(org_id)
        .filter(organization_users::Column::UserId.eq(identity.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    view.set("organization", org);

    if let Some(fields) = form.map(|f| f.into_inner()) {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let project_search = Projects::find()
            .filter(projects::Column::Name.eq(&fields.project_name))
            .filter(projects::Column::OrganizationId.eq(org_id))
            .one(&ctx.db)
            .await?;

        if project_search.is_some() {
            let mut errors = ValidationErrors::new();
            errors.add(
                "project_name",
                ValidationError::new("exists").with_message("A project with the same name already exists".into()),
            );
            view.set("errors", errors);
            return Ok(view);
        }

        let api_key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let project = projects::ActiveModel {
            organization_id: ActiveValue::set(org_id),
            name: ActiveValue::set(fields.project_name),
            api_key: ActiveValue::set(api_key),
            ..Default::default()
        };

        let project = project.save(&ctx.db).await?.try_into_model()?;

        let project_user_settings = project_user_settings::ActiveModel {
            project_id: ActiveValue::set(project.project_id),
            user_id: ActiveValue::set(identity.user_id),
            notify_email: ActiveValue::set(1),
        };

        project_user_settings.insert(&ctx.db).await?;

        view.redirect(format!("/organization/{}?tab=projects", org_id), true);
    }

    Ok(view)
}

#[route(
    "/organization/{organization_id}/project-edit/{project_id}",
    method = "GET",
    method = "POST"
)]
async fn org_project_edit(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<(u32, u32)>,
    form: Option<web::Form<ProjectFrom>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/project_edit");

    let form = form.map(|f| f.into_inner());

    let (org_id, project_id) = path.into_inner();

    let user = identity.user(&ctx).await?;
    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;
    view.set("role", &user_role);

    let mut project = Projects::find_by_id(project_id)
        .filter(projects::Column::OrganizationId.eq(org_id))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    view.set(
        "form",
        form.clone().unwrap_or_else(|| ProjectFrom {
            project_name: project.name.clone(),
        }),
    );

    if let Some(fields) = form {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let project_search = Projects::find()
            .filter(projects::Column::Name.eq(&fields.project_name))
            .filter(projects::Column::ProjectId.ne(project_id))
            .one(&ctx.db)
            .await?;

        if project_search.is_some() {
            let mut errors = ValidationErrors::new();
            errors.add(
                "project_name",
                ValidationError::new("exists").with_message("A project with the same name already exists".into()),
            );
            view.set("errors", errors);
            return Ok(view);
        }

        let mut project_model = project.into_active_model();
        project_model.name = ActiveValue::set(fields.project_name);
        project = project_model.save(&ctx.db).await?.try_into_model()?;

        view.message("Project information updated");
        view.set("saved", true);
    }

    view.set("project", project);
    view.set("org_id", org_id);

    Ok(view)
}

#[delete("/organization/{organization_id}/project-edit/{project_id}")]
async fn org_project_delete(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<(u32, u32)>,
) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let (org_id, project_id) = path.into_inner();

    let user = identity.user(&ctx).await?;
    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

    if user_role == "admin" || user_role == "owner" {
        let project = Projects::find_by_id(project_id)
            .filter(projects::Column::OrganizationId.eq(org_id))
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;

        project.delete(&ctx.db).await?;

        view.message("Project deleted");
    }

    view.redirect(format!("/organization/{}?tab=projects", org_id), true);

    Ok(view)
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct OrgSettingsFrom {
    #[validate(length(min = 1, max = 80, message = "Organization name is required"))]
    name: String,
}

#[route("/organization/{organization_id}/settings", method = "GET", method = "POST")]
async fn org_settings(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<u32>,
    form: Option<web::Form<OrgSettingsFrom>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/settings");

    let org_id = path.into_inner();
    let form = form.map(|f| f.into_inner());

    let user = identity.user(&ctx).await?;
    view.set("user", &user);

    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;
    view.set("role", &user_role);

    view.set("org_id", org_id);

    let mut org = Organizations::find_by_id(org_id)
        .filter(organization_users::Column::UserId.eq(identity.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    view.set(
        "form",
        form.clone()
            .unwrap_or_else(|| OrgSettingsFrom { name: org.name.clone() }),
    );

    if let Some(fields) = form {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let org_search = Organizations::find()
            .filter(organizations::Column::Name.eq(&fields.name))
            .filter(organizations::Column::OrganizationId.ne(org_id))
            .one(&ctx.db)
            .await?;

        if org_search.is_some() {
            let mut errors = ValidationErrors::new();
            errors.add(
                "name",
                ValidationError::new("exists").with_message("An organization with the same name already exists".into()),
            );
            view.set("errors", errors);
            return Ok(view);
        }

        let mut org_model = org.into_active_model();
        org_model.name = ActiveValue::set(fields.name);
        org = org_model.save(&ctx.db).await?.try_into_model()?;

        view.message("Organization information updated");
        view.set("saved", true);
    }

    let reset_date = org.requests_count_start.map(|date| date + Days::new(30));
    view.set("limit_reset_date", reset_date);

    view.set("organization", org);

    Ok(view)
}

#[get("/organization/{organization_id}/notifications")]
async fn org_notifications(
    _ctx: web::Data<AppContext<'_>>,
    _identity: Identity,
    path: web::Path<u32>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/notifications");

    let organization_id = path.into_inner();
    view.set("org_id", organization_id);

    Ok(view)
}

#[derive(Serialize)]
struct OrganizationMember {
    user: Option<users::Model>,
    user_org: organization_users::Model,
}

#[get("/organization/{organization_id}/members")]
async fn org_members(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/members");

    let user = identity.user(&ctx).await?;

    let org_id = path.into_inner();
    view.set("org_id", org_id);

    let org = Organizations::find_by_id(org_id)
        .filter(organization_users::Column::UserId.eq(identity.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let org_members: Vec<OrganizationMember> = OrganizationUsers::find()
        .filter(organization_users::Column::OrganizationId.eq(org_id))
        .find_also_related(Users)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|(user_org, user)| OrganizationMember { user_org, user })
        .collect();

    let org_invites = OrganizationInvitations::find()
        .filter(organization_invitations::Column::OrganizationId.eq(org_id))
        .all(&ctx.db)
        .await?;

    view.set("invitations", org_invites);
    view.set("members", org_members);
    view.set("organization", org);
    view.set("user", user);

    Ok(view)
}

#[derive(Serialize, Deserialize, Validate)]
#[validate(context = String)]
struct InviteForm {
    #[validate(
        email(message = "A valid email address is required"),
        length(max = 320, message = "Must be less than 320 chars")
    )]
    email: String,
    #[validate(custom(function = "validate_role_choice", use_context))]
    role: String,
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

#[route("/organization/{organization_id}/invite", method = "GET", method = "POST")]
async fn org_invite(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<u32>,
    form: Option<web::Form<InviteForm>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/invite");

    view.set("form", &form);

    let org_id = path.into_inner();
    view.set("org_id", org_id);

    let user = identity.user(&ctx).await?;
    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;
    view.set("role", &user_role);

    let org = Organizations::find_by_id(org_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    if let Some(fields) = form.map(|f| f.into_inner()) {
        if let Err(errors) = fields.validate_with_args(&user_role) {
            view.set("errors", &errors);
            return Ok(view);
        }

        // check if invite exists
        let maybe_invite = OrganizationInvitations::find()
            .filter(organization_invitations::Column::Email.eq(&fields.email))
            .filter(organization_invitations::Column::OrganizationId.eq(org_id))
            .one(&ctx.db)
            .await?;

        if let Some(invitation) = maybe_invite {
            let mut errors = ValidationErrors::new();
            errors.add(
                "email",
                ValidationError::new("exists").with_message("Email already invited".into()),
            );
            view.set("errors", errors);

            view.set("existing_invitation_id", invitation.organization_invitation_id);

            return Ok(view);
        }

        let maybe_user = Users::find()
            .filter(users::Column::Email.eq(&fields.email))
            .one(&ctx.db)
            .await?;

        if let Some(user) = maybe_user {
            let maybe_user = OrganizationUsers::find_by_id((user.user_id, org_id))
                .one(&ctx.db)
                .await?;

            if maybe_user.is_some() {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "email",
                    ValidationError::new("exists").with_message("User is already a member".into()),
                );
                view.set("errors", errors);
                return Ok(view);
            }

            let org_member = organization_users::ActiveModel {
                organization_id: ActiveValue::set(org_id),
                user_id: ActiveValue::set(user.user_id),
                role: ActiveValue::set(fields.role),
                ..Default::default()
            };

            org_member.insert(&ctx.db).await?;

            let title = format!("You have been added to the {} organization in Don't Panic", org.name);

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
                        "organization": org,
                        "added_by": user
                    }),
                )?)?;

            if let Some(mailer) = ctx.mailer.as_ref() {
                mailer.send(email).await?;
            }

            view.message("Member added");
        } else {
            let org_invitation = organization_invitations::ActiveModel {
                organization_id: ActiveValue::set(org_id),
                email: ActiveValue::set(fields.email.clone()),
                role: ActiveValue::set(fields.role),
                ..Default::default()
            };

            org_invitation.insert(&ctx.db).await?;

            let title = format!("You have been invited to the {} organization in Don't Panic", org.name);

            let email = lettre::Message::builder()
                .from(ctx.config.email_from.clone().into())
                .to(fields.email.parse()?)
                .subject(title.clone())
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(ctx.hb.render(
                    "email/org_invitation",
                    &serde_json::json!({
                        "base_url": ctx.config.base_url,
                        "scheme": ctx.config.scheme,
                        "organization": org,
                        "added_by": user
                    }),
                )?)?;

            if let Some(mailer) = ctx.mailer.as_ref() {
                mailer.send(email).await?;
            }

            view.message("Invitation sent");
        }

        view.redirect(format!("/organization/{}?tab=members", org_id), false);
    }

    Ok(view)
}

#[delete("/organization/{organization_id}/invite/{organization_invitation_id}")]
async fn org_invite_delete(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<(u32, u32)>,
) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let (org_id, org_invitation_id) = path.into_inner();

    let user = identity.user(&ctx).await?;
    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

    if user_role == "admin" || user_role == "owner" {
        let invitation = OrganizationInvitations::find_by_id(org_invitation_id)
            .filter(organization_invitations::Column::OrganizationId.eq(org_id))
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;
        invitation.delete(&ctx.db).await?;

        view.message("Invitation deleted");
    }

    view.redirect(format!("/organization/{}?tab=members", org_id), false);

    Ok(view)
}

#[post("/organization/{organization_id}/resend-invite/{organization_invitation_id}")]
async fn org_invite_resend(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<(u32, u32)>,
) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let (org_id, org_invitation_id) = path.into_inner();

    let org = Organizations::find_by_id(org_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

    let invitation = OrganizationInvitations::find_by_id(org_invitation_id)
        .filter(organization_invitations::Column::OrganizationId.eq(org_id))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let title = format!("You have been invited to the {} organization in Don't Panic", org.name);

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
                "organization": org,
                "added_by": user
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    view.message("Invitation sent");

    Ok(view)
}

#[delete("/organization/{organization_id}/member/{user_id}")]
async fn org_member_delete(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<(u32, u32)>,
) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let (org_id, user_id) = path.into_inner();

    view.redirect(format!("/organization/{}?tab=members", org_id), false);

    let user = identity.user(&ctx).await?;
    let user_role = user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;

    if user_role == "admin" || user_role == "owner" {
        let org_member = OrganizationUsers::find_by_id((user_id, org_id))
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

    Ok(view)
}

#[derive(Serialize, Deserialize, Validate)]
#[validate(context = String)]
struct EditForm {
    #[validate(custom(function = "validate_role_choice", use_context))]
    role: String,
}

#[route(
    "/organization/{organization_id}/member-edit/{user_id}",
    method = "GET",
    method = "POST"
)]
async fn org_member_edit(
    ctx: web::Data<AppContext<'_>>,
    identity: Identity,
    path: web::Path<(u32, u32)>,
    form: Option<web::Form<EditForm>>,
) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("organizations/member_edit");

    view.set("form", &form);

    let (org_id, user_id) = path.into_inner();
    view.set("org_id", org_id);

    let current_user = identity.user(&ctx).await?;
    let user_role = current_user.role(&ctx.db, org_id).await?.ok_or(Error::LoginRequired)?;
    view.set("role", &user_role);

    let member = Users::find_by_id(user_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;
    let org_member = OrganizationUsers::find_by_id((member.user_id, org_id))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    view.set("member", &member);
    view.set("org_member", &org_member);

    if let Some(fields) = form.map(|f| f.into_inner()) {
        if let Err(errors) = fields.validate_with_args(&user_role) {
            view.set("errors", &errors);
            return Ok(view);
        }

        let mut org_member_model = org_member.into_active_model();
        org_member_model.role = ActiveValue::set(fields.role);
        org_member_model.save(&ctx.db).await?.try_into_model()?;

        view.redirect(format!("/organization/{}?tab=members", org_id), false);
    }

    Ok(view)
}
