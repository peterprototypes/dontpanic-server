use actix_web::{
    get, post,
    web::{self, Data, Json, Path},
    Responder,
};
use migration::IntoCondition;
use sea_orm::{prelude::*, QuerySelect};
use sea_orm::{ActiveValue, FromQueryResult, JoinType};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::entity::organization_users;
use crate::entity::prelude::*;
use crate::entity::project_user_settings;
use crate::entity::users;

use crate::{AppContext, Error, Identity, Result};

mod slack_app;
mod slack_webhook;
mod teams_webhook;
mod webhook;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(per_user_notifications)
        .service(per_user_save)
        .service(get_project)
        .service(web::scope("/{project_id}/slack-app").configure(slack_app::routes))
        .service(web::scope("/{project_id}/slack-webhook").configure(slack_webhook::routes))
        .service(web::scope("/{project_id}/teams-webhook").configure(teams_webhook::routes))
        .service(web::scope("/{project_id}/webhook").configure(webhook::routes));
}

#[derive(FromQueryResult, Serialize)]
struct NotificationsTable {
    user_id: u32,
    email: String,
    role: String,
    name: Option<String>,
    notify_email: Option<bool>,
    notify_pushover: Option<bool>,
}

#[get("/per-user/{project_id}")]
async fn per_user_notifications(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();

    let project = Projects::find_by_id(project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _ = user
        .role(&ctx.db, project.organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    let settings: Vec<NotificationsTable> = Users::find()
        .select_only()
        .column(users::Column::UserId)
        .column(users::Column::Email)
        .column(users::Column::Name)
        .column(organization_users::Column::Role)
        .column(project_user_settings::Column::NotifyEmail)
        .column(project_user_settings::Column::NotifyPushover)
        .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
        .join(
            JoinType::LeftJoin,
            users::Relation::ProjectUserSettings
                .def()
                .on_condition(move |_left, right| {
                    Expr::col((right, project_user_settings::Column::ProjectId))
                        .eq(project_id)
                        .into_condition()
                }),
        )
        .filter(organization_users::Column::OrganizationId.eq(project.organization_id))
        .into_model()
        .all(&ctx.db)
        .await?;

    Ok(Json(json!({
        "organization_id": project.organization_id,
        "settings": settings,
    })))
}

#[derive(Serialize, Deserialize, Debug)]
struct UserNotificationSettings {
    user_id: u32,
    notify_email: Option<bool>,
    notify_pushover: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserNotificationsForm {
    settings: Vec<UserNotificationSettings>,
}

#[post("/per-user/{project_id}")]
async fn per_user_save(
    ctx: Data<AppContext<'_>>,
    id: Identity,
    path: Path<u32>,
    input: Json<UserNotificationsForm>,
) -> Result<impl Responder> {
    let project_id = path.into_inner();

    let project = Projects::find_by_id(project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _ = user
        .role(&ctx.db, project.organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    ProjectUserSettings::delete_many()
        .filter(project_user_settings::Column::ProjectId.eq(project_id))
        .exec(&ctx.db)
        .await?;

    for user_settings in input.into_inner().settings {
        // make sure member is part of org
        let org_member_search = OrganizationUsers::find_by_id((user_settings.user_id, project.organization_id))
            .one(&ctx.db)
            .await?;

        if org_member_search.is_none() {
            continue;
        }

        let project_member = project_user_settings::ActiveModel {
            project_id: ActiveValue::set(project_id),
            user_id: ActiveValue::set(user_settings.user_id),
            notify_email: ActiveValue::set(user_settings.notify_email.unwrap_or_default() as i8),
            notify_pushover: ActiveValue::set(user_settings.notify_pushover.unwrap_or_default() as i8),
        };

        project_member.insert(&ctx.db).await?;
    }

    Ok(Json(()))
}

#[get("/project/{project_id}")]
async fn get_project(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();

    let project = Projects::find_by_id(project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _ = user
        .role(&ctx.db, project.organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    let environments = project.find_related(ProjectEnvironments).all(&ctx.db).await?;

    let mut res = serde_json::json!(project);
    res["environments"] = serde_json::json!(environments);

    Ok(Json(res))
}
