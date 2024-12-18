use actix_web::{
    get, post,
    web::{self, Data, Json, Path},
    Responder,
};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::entity::prelude::*;

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(config)
        .service(config_save)
        .service(save)
        .service(delete)
        .service(test);
}

#[derive(Serialize, Deserialize, Debug)]
struct SlackChannels {
    channels: Vec<SlackChannel>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SlackChannel {
    id: String,
    is_member: bool,
    name: String,
}

#[get("/config")]
async fn config(ctx: Data<AppContext<'_>>, path: Path<u32>, id: Identity) -> Result<impl Responder> {
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

    let mut chats = vec![];

    if let Some(bot_token) = project.slack_bot_token.as_deref() {
        let params = [
            ("token", bot_token),
            ("limit", "1000"),
            ("types", "public_channel,private_channel"),
        ];

        let client = reqwest::Client::new();
        let mut result: SlackChannels = client
            .post("https://slack.com/api/conversations.list")
            .form(&params)
            .send()
            .await?
            .json()
            .await?;

        result.channels.retain(|channel| channel.is_member);

        chats = result.channels;
    }

    let redirect_uri = format!(
        "{}://{}/reports/notifications?project_id={}",
        ctx.config.scheme, ctx.config.base_url, project.project_id
    );

    // chats = vec![];

    Ok(Json(json!({
        "project_id": project.project_id,
        "slack_chats": chats,
        "slack_redirect_uri": redirect_uri,
        "slack_client_id": ctx.config.slack_client_id,
    })))
}

#[derive(Deserialize, Debug)]
struct SlackAuthParams {
    code: String,
}

#[derive(Deserialize, Debug)]
struct SlackAccessResponse {
    ok: bool,
    access_token: Option<String>,
    error: Option<String>,
}

#[post("/config")]
async fn config_save(
    ctx: Data<AppContext<'_>>,
    path: Path<u32>,
    id: Identity,
    input: Json<SlackAuthParams>,
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

    let code = input.into_inner().code;

    let redirect_uri = format!(
        "{}://{}/reports/notifications?project_id={}",
        ctx.config.scheme, ctx.config.base_url, project.project_id
    );

    // let redirect_uri = format!("https://localhost:8080/reports/notifications?project_id=10");

    let client_id = ctx
        .config
        .slack_client_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Slack client id not found"))?;

    let client_secret = ctx
        .config
        .slack_client_secret
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Slack client secret not found"))?;

    let params = [
        ("code", urlencoding::encode(&code).to_string()),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
    ];

    let client = reqwest::Client::new();
    let result: SlackAccessResponse = client
        .post("https://slack.com/api/oauth.v2.access")
        .form(&params)
        .send()
        .await?
        .json()
        .await?;

    if !result.ok {
        return Err(Error::Internal(anyhow::anyhow!("Error from slack: {:?}", result.error)));
    }

    let mut project_model = project.into_active_model();
    project_model.slack_bot_token = ActiveValue::set(result.access_token);
    project_model.save(&ctx.db).await?;

    Ok(Json(()))
}

#[derive(Deserialize, Validate)]
struct SlackAppInput {
    #[validate(length(min = 1, message = "Slack channel is required"))]
    slack_channel: String,
}

#[post("/save")]
async fn save(
    ctx: Data<AppContext<'_>>,
    id: Identity,
    path: Path<u32>,
    input: Json<SlackAppInput>,
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
    project_model.slack_channel = ActiveValue::set(Some(input.slack_channel));
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
    project_model.slack_bot_token = ActiveValue::set(None);
    project_model.slack_channel = ActiveValue::set(None);
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

    let Some((token, channel)) = project.slack_bot_token.zip(project.slack_channel) else {
        return Err(Error::new("Slack App not configured"));
    };

    let params = [
        ("token", token),
        ("channel", channel),
        (
            "text",
            "Slack is working! I'll post here when your app panic!()s".into(),
        ),
    ];

    let client = reqwest::Client::new();
    client
        .post("https://slack.com/api/chat.postMessage")
        .form(&params)
        .send()
        .await?;

    Ok(Json(()))
}
