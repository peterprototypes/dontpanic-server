use std::collections::HashMap;

use actix_web::{
    post,
    web::{self, Data, Json, Path},
    Responder,
};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use validator::Validate;

use crate::entity::prelude::*;

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(save).service(delete).service(test);
}

#[derive(Deserialize, Validate)]
struct EnvironmentWebhookInput {
    project_environment_id: u32,
    slack_webhook: Option<String>,
}

#[derive(Deserialize, Validate)]
struct WebhookInput {
    #[validate(url(message = "Please enter a valid URL"))]
    webhook_url: String,
    environments: Vec<EnvironmentWebhookInput>,
}

#[post("/save")]
async fn save(
    ctx: Data<AppContext<'_>>,
    id: Identity,
    path: Path<u32>,
    input: Json<WebhookInput>,
) -> Result<impl Responder> {
    input.validate()?;
    let input = input.into_inner();

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

    let mut project_model = project.into_active_model();
    project_model.slack_webhook = ActiveValue::set(Some(input.webhook_url));
    project_model.save(&ctx.db).await?;

    for env_input in input.environments {
        let Some(env) = ProjectEnvironments::find_by_id(env_input.project_environment_id)
            .one(&ctx.db)
            .await?
        else {
            continue;
        };

        let mut env = env.into_active_model();
        env.slack_webhook = ActiveValue::set(env_input.slack_webhook);
        env.save(&ctx.db).await?;
    }

    Ok(Json(()))
}

#[post("/delete")]
async fn delete(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
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

    let mut project_model = project.into_active_model();
    project_model.slack_webhook = ActiveValue::set(None);
    project_model.save(&ctx.db).await?;

    Ok(Json(()))
}

#[post("/test")]
async fn test(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
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

    let Some(webhook_url) = project.slack_webhook else {
        return Err(Error::new("Slack Webhook URL is not set"));
    };

    let mut params = HashMap::new();
    params.insert(
        "text",
        format!(
            "Slack is working! I'll post here when project {} panic!()s",
            project.name
        ),
    );

    let client = reqwest::Client::new();
    let res = client.post(webhook_url).json(&params).send().await?;

    log::info!("Slack test response: {:?}", res.text().await);

    Ok(Json(()))
}
