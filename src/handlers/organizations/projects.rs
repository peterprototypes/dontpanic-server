use actix_web::{
    get, post, web,
    web::{Data, Json, Path},
    Responder,
};
use rand::{distributions::Alphanumeric, prelude::*};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, JoinType, QuerySelect, QueryTrait, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::project_user_settings;
use crate::entity::projects;

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list).service(manage).service(get_single).service(delete);
}

#[derive(Serialize, Debug)]
pub struct OrganizationProject {
    project_id: u32,
    organization_id: u32,
    name: String,
    api_key: String,
    created: DateTime,
}

impl From<projects::Model> for OrganizationProject {
    fn from(project: projects::Model) -> Self {
        Self {
            project_id: project.project_id,
            organization_id: project.organization_id,
            name: project.name,
            api_key: project.api_key,
            created: project.created,
        }
    }
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

    let projects: Vec<OrganizationProject> = organization
        .find_related(Projects)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(OrganizationProject::from)
        .collect();

    Ok(Json(projects))
}

#[derive(Debug, Deserialize, Validate)]
struct ProjectInput {
    project_id: Option<u32>,
    #[validate(length(min = 1, max = 80, message = "Project name is required"))]
    name: String,
}

#[post("")]
async fn manage(
    ctx: Data<AppContext<'_>>,
    path: Path<u32>,
    id: Identity,
    input: Json<ProjectInput>,
) -> Result<impl Responder> {
    input.validate()?;
    let input = input.into_inner();

    let organization_id = path.into_inner();
    let user = id.user(&ctx).await?;

    let organization = Organizations::find_by_id(organization_id)
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let project_search = Projects::find()
        .filter(projects::Column::Name.eq(&input.name))
        .filter(projects::Column::OrganizationId.eq(organization.organization_id))
        .apply_if(input.project_id, |q, project_id| {
            q.filter(projects::Column::ProjectId.ne(project_id))
        })
        .one(&ctx.db)
        .await?;

    if project_search.is_some() {
        return Err(Error::field(
            "name",
            "A project with the same name already exists".into(),
        ));
    }

    let mut project = if let Some(project_id) = input.project_id {
        Projects::find_by_id(project_id)
            .filter(projects::Column::OrganizationId.eq(organization.organization_id))
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?
            .into_active_model()
    } else {
        let api_key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        projects::ActiveModel {
            organization_id: ActiveValue::set(organization.organization_id),
            api_key: ActiveValue::set(api_key),
            ..Default::default()
        }
    };

    let is_new = project.project_id.is_not_set();

    project.name = ActiveValue::set(input.name);

    let project = project.save(&ctx.db).await?.try_into_model()?;

    if is_new {
        let project_user_settings = project_user_settings::ActiveModel {
            project_id: ActiveValue::set(project.project_id),
            user_id: ActiveValue::set(id.user_id),
            notify_email: ActiveValue::set(1),
            notify_pushover: ActiveValue::set(if user.pushover_user_key.is_some() { 1 } else { 0 }),
        };

        project_user_settings.insert(&ctx.db).await?;
    }

    Ok(Json(OrganizationProject::from(project)))
}

#[get("/{project_id}")]
async fn get_single(ctx: Data<AppContext<'_>>, path: Path<(u32, u32)>, id: Identity) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;

    let (organization_id, project_id) = path.into_inner();

    let organization = Organizations::find_by_id(organization_id)
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let project = Projects::find_by_id(project_id)
        .filter(projects::Column::OrganizationId.eq(organization.organization_id))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(Json(OrganizationProject::from(project)))
}

#[post("/delete/{project_id}")]
async fn delete(ctx: web::Data<AppContext<'_>>, path: web::Path<(u32, u32)>, id: Identity) -> Result<impl Responder> {
    let (organization_id, project_id) = path.into_inner();

    let user = id.user(&ctx).await?;
    let user_role = user.role(&ctx.db, organization_id).await?.ok_or(Error::LoginRequired)?;

    if user_role == "admin" || user_role == "owner" {
        let project = Projects::find_by_id(project_id)
            .filter(projects::Column::OrganizationId.eq(organization_id))
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;

        project.delete(&ctx.db).await?;
    } else {
        return Err(Error::new(
            "You do not have permission to delete projects in this organization",
        ));
    }

    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use actix_web::test;
    use serde_json::Value;

    #[actix_web::test]
    async fn test_crud() {
        let (app, sess) = crate::test_app_with_auth().await.unwrap();

        // create
        let req = test::TestRequest::post()
            .uri("/api/organizations/1/projects")
            .cookie(sess.clone())
            .set_json(serde_json::json!({
                "name": "Test Project",
            }))
            .to_request();

        let resp: Value = test::call_and_read_body_json(&app, req).await;
        let project_id = resp["project_id"].as_u64().unwrap();

        // get single
        let req = test::TestRequest::get()
            .uri(&format!("/api/organizations/1/projects/{}", project_id))
            .cookie(sess.clone())
            .to_request();

        let resp: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp["name"], "Test Project");
        assert_eq!(resp["project_id"], project_id);

        // edit
        let req = test::TestRequest::post()
            .uri("/api/organizations/1/projects")
            .cookie(sess.clone())
            .set_json(serde_json::json!({
                "project_id": project_id,
                "name": "Test Project Updated",
            }))
            .to_request();

        test::call_service(&app, req).await;

        // list
        let req = test::TestRequest::get()
            .uri("/api/organizations/1/projects")
            .cookie(sess.clone())
            .to_request();

        let resp: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp[0]["name"], "Test Project Updated");
        assert_eq!(resp[0]["project_id"], project_id);

        //delete
        let req = test::TestRequest::post()
            .uri(&format!("/api/organizations/1/projects/delete/{}", project_id))
            .cookie(sess.clone())
            .to_request();

        test::call_service(&app, req).await;

        // check deleted
        let req = test::TestRequest::get()
            .uri("/api/organizations/1/projects")
            .cookie(sess.clone())
            .to_request();

        let resp: Value = test::call_and_read_body_json(&app, req).await;
        assert!(resp.as_array().unwrap().is_empty());
    }
}
