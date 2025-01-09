use actix_web::{
    post,
    web::{self, Data, Json, Path},
    Responder,
};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::entity::prelude::*;

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
    project_model.teams_webhook = ActiveValue::set(Some(input.webhook_url));
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
    project_model.teams_webhook = ActiveValue::set(None);
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

    let Some(webhook_url) = project.teams_webhook else {
        return Err(Error::new("Teams Webhook URL is not set"));
    };

    let params = json!({
        "type": "message",
        "attachments": [
            {
                "contentType": "application/vnd.microsoft.card.adaptive",
                "content": {
                    "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                    "type": "AdaptiveCard",
                    "version": "1.0",
                    "body": [
                        {
                            "type": "ColumnSet",
                            "columns": [
                                {
                                    "type": "Column",
                                    "width": "auto",
                                    "items": [
                                        {
                                            "type": "Image",
                                            "url": "https://dontpanic.rs/static/favicon.png",
                                            "size": "small",
                                        }
                                    ]
                                },
                                {
                                    "type": "Column",
                                    "width": "stretch",
                                    "verticalContentAlignment": "center",
                                    "items": [
                                        {
                                            "type": "TextBlock",
                                            "text": "Don't Panic MS Teams integration is working",
                                            "size": "medium",
                                            "weight": "bolder",
                                            "style": "heading",
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "type": "TextBlock",
                            "text": format!(
                                "You'll get notified in this channel when '{}' experiences errors or panics.",
                                project.name
                            ),
                            "wrap": "true",
                        }
                    ]
                }
            }
        ]
    });

    let client = reqwest::Client::new();
    let res = client.post(webhook_url).json(&params).send().await?;

    log::info!("MS Teams test response: {:?}", res.text().await);

    Ok(Json(()))
}
