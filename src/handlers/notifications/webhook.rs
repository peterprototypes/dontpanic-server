use actix_web::{
    post,
    web::{self, Data, Json, Path},
    Responder,
};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use validator::Validate;

use crate::{entity::prelude::*, event::EventData, notifications::ReportStatus};

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(save).service(delete).service(test);
}

#[derive(Deserialize, Validate)]
struct WebhookInput {
    #[validate(url(message = "Please enter a valid URL"))]
    webhook_url: String,
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
    project_model.webhook = ActiveValue::set(Some(input.webhook_url));
    project_model.save(&ctx.db).await?;

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
    project_model.webhook = ActiveValue::set(None);
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

    let Some(webhook_url) = project.webhook else {
        return Err(Error::new("Webhook URL is not set"));
    };

    let params = serde_json::json!({
        "status": ReportStatus::New,
        "title": "Called `Option::unwrap()` on a `None` value (Webhook Test)",
        "project": project.name,
        "environment": Option::<String>::None,
        "event": EventData::example(),
    });

    let client = reqwest::Client::new();
    let _res = client.post(webhook_url).json(&params).send().await?;

    Ok(Json(()))
}
