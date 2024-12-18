use actix_web::Responder;
use actix_web::{
    get, post,
    web::{self, Data, Json, Path},
};
use chrono::{Days, Utc};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, JoinType, QuerySelect, TryIntoModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;

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
        .service(web::scope("/{organization_id}/members").configure(members::routes))
        .service(delete)
        .service(edit);
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

        let _todo_reset_date = org.requests_count_start.map(|date| date + Days::new(30));

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
async fn create(ctx: Data<AppContext<'_>>, id: Identity, input: Json<CreateInput>) -> Result<impl Responder> {
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

#[post("/{organization_id}")]
async fn edit(
    ctx: Data<AppContext<'_>>,
    id: Identity,
    input: Json<CreateInput>,
    path: Path<u32>,
) -> Result<impl Responder> {
    let input = input.into_inner();
    input.validate()?;

    let organization_id = path.into_inner();
    let user = id.user(&ctx).await?;

    let user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    if user_role != "owner" {
        return Err(Error::new("Only owners can manage an organization"));
    }

    let org = Organizations::find_by_id(organization_id)
        .filter(organization_users::Column::UserId.eq(id.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let org_search = Organizations::find()
        .filter(organizations::Column::Name.eq(&input.name))
        .filter(organizations::Column::OrganizationId.ne(organization_id))
        .one(&ctx.db)
        .await?;

    if org_search.is_some() {
        return Err(Error::field(
            "name",
            "An organization with the same name already exists.".into(),
        ));
    }

    let mut org_model = org.into_active_model();
    org_model.name = ActiveValue::set(input.name);
    org_model.save(&ctx.db).await?;

    Ok(Json(()))
}

#[post("/{organization_id}/delete")]
async fn delete(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
    let organization_id = path.into_inner();

    let user = id.user(&ctx).await?;
    let user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    if user_role != "owner" {
        return Err(Error::new("Only owners can delete an organization"));
    }

    Organizations::delete(&ctx.db, organization_id).await?;

    Ok(Json(()))
}
