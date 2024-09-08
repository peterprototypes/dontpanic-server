use std::collections::HashMap;

use actix_web::{get, post, route, web};
use migration::IntoCondition;
use sea_orm::{prelude::*, IntoActiveModel, QuerySelect, TryIntoModel};
use sea_orm::{ActiveValue, FromQueryResult, JoinType};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_qs::actix::QsForm;
use validator::Validate;

use crate::entity::users;
use crate::entity::{organization_users, prelude::*, project_user_settings};

use crate::event::EventData;
use crate::notifications::ReportStatus;
use crate::AppContext;
use crate::Error;
use crate::Identity;
use crate::Result;
use crate::ViewModel;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(notifications)
        .service(notifications_save)
        .service(slack_auth)
        .service(slack_config)
        .service(slack_test)
        .service(slack_webhook)
        .service(slack_test_webhook)
        .service(webhook)
        .service(test_webhook);
}

#[derive(FromQueryResult, Serialize)]
struct NotificationsTable {
    user_id: u32,
    email: String,
    role: String,
    name: Option<String>,
    notify_email: Option<bool>,
}

#[get("/notifications/setup/{project_id}")]
async fn notifications(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("notifications/setup");

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;
    view.set("user", user);

    let members: Vec<NotificationsTable> = Users::find()
        .select_only()
        .column(users::Column::UserId)
        .column(users::Column::Email)
        .column(users::Column::Name)
        .column(organization_users::Column::Role)
        .column(project_user_settings::Column::NotifyEmail)
        .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
        .join(
            JoinType::LeftJoin,
            users::Relation::ProjectUserSettings
                .def()
                .on_condition(move |_left, right| Expr::col((right, project_user_settings::Column::ProjectId)).eq(project_id).into_condition()),
        )
        .filter(organization_users::Column::OrganizationId.eq(project.organization_id))
        .into_model()
        .all(&ctx.db)
        .await?;

    view.set("members", members);
    view.set("project", project);
    view.set("project_id", project_id);

    Ok(view)
}

#[derive(Deserialize, Debug)]
struct NotificationSettings {
    user_ids: Option<Vec<u32>>,
}

#[post("/notifications/setup/{project_id}")]
async fn notifications_save(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>, form: QsForm<NotificationSettings>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    ProjectUserSettings::delete_many()
        .filter(project_user_settings::Column::ProjectId.eq(project_id))
        .exec(&ctx.db)
        .await?;

    if let Some(user_ids) = form.into_inner().user_ids {
        for user_id in user_ids {
            // make sure member is part of org
            let org_member_search = OrganizationUsers::find_by_id((user_id, project.organization_id)).one(&ctx.db).await?;

            if org_member_search.is_none() {
                continue;
            }

            let project_member = project_user_settings::ActiveModel {
                project_id: ActiveValue::set(project_id),
                user_id: ActiveValue::set(user_id),
                notify_email: ActiveValue::set(1),
            };

            project_member.insert(&ctx.db).await?;
        }
    }

    view.message("Notification settings updated.");

    Ok(view)
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

#[derive(Serialize, Deserialize, Debug)]
struct SlackConfigForm {
    channel: String,
}

#[route("/notifications/slack-config/{project_id}", method = "GET", method = "POST")]
async fn slack_config(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>, form: Option<web::Form<SlackConfigForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("notifications/slack_config");

    view.set("form", &form);

    let project_id = path.into_inner();
    let mut project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    view.set(
        "slack_redirect_uri",
        format!("{}://{}/notifications/slack-auth/{}", ctx.config.scheme, ctx.config.base_url, project.project_id),
    );

    view.set("slack_client_id", &ctx.config.slack_client_id);

    if let Some(bot_token) = project.slack_bot_token.as_deref() {
        let params = [("token", bot_token), ("limit", "1000"), ("types", "public_channel,private_channel")];

        let client = reqwest::Client::new();
        let mut result: SlackChannels = client.post("https://slack.com/api/conversations.list").form(&params).send().await?.json().await?;

        result.channels.retain(|channel| channel.is_member);

        view.set("slack_chats", result.channels);

        if let Some(fields) = form.map(|f| f.into_inner()) {
            let mut project_model = project.into_active_model();
            project_model.slack_channel = ActiveValue::set(Some(fields.channel));
            project = project_model.save(&ctx.db).await?.try_into_model()?;

            view.message("Slack channel set");
        }
    }

    view.set("project", &project);

    Ok(view)
}

#[post("/notifications/slack-test/{project_id}")]
async fn slack_test(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    let Some((token, channel)) = project.slack_bot_token.zip(project.slack_channel) else {
        view.message("Slack not configured");
        return Ok(view);
    };

    let params = [
        ("token", token),
        ("channel", channel),
        ("text", "Slack is working! I'll post here when your app panic!()s".into()),
    ];

    let client = reqwest::Client::new();
    client.post("https://slack.com/api/chat.postMessage").form(&params).send().await?;

    view.message("Message sent");

    Ok(view)
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

#[get("/notifications/slack-auth/{project_id}")]
async fn slack_auth(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>, query: web::Query<SlackAuthParams>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    // retrieve the verification code and send it to oauth.v2.access to obtain an access_token

    let redirect_uri = format!("{}://{}/notifications/slack-auth/{}", ctx.config.scheme, ctx.config.base_url, project.project_id);

    let client_id = ctx.config.slack_client_id.clone().ok_or_else(|| anyhow::anyhow!("Slack client id not found"))?;
    let client_secret = ctx.config.slack_client_secret.clone().ok_or_else(|| anyhow::anyhow!("Slack client secret not found"))?;

    let params = [
        ("code", urlencoding::encode(&query.code).to_string()),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
    ];

    let client = reqwest::Client::new();
    let result: SlackAccessResponse = client.post("https://slack.com/api/oauth.v2.access").form(&params).send().await?.json().await?;

    if !result.ok {
        return Err(Error::Internal(anyhow::anyhow!("Error from slack: {:?}", result.error)));
    }

    let mut project_model = project.into_active_model();
    project_model.slack_bot_token = ActiveValue::set(result.access_token);
    project_model.save(&ctx.db).await?;

    view.redirect(format!("/notifications/setup/{}", project_id), true);

    Ok(view)
}

#[derive(Serialize, Deserialize, Debug, Validate)]
struct SlackWebhookForm {
    #[serde(deserialize_with = "empty_string_as_none")]
    #[validate(url(message = "Please enter a valid URL"))]
    webhook_url: Option<String>,
}

#[route("/notifications/slack-webhook/{project_id}", method = "GET", method = "POST")]
async fn slack_webhook(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>, form: Option<web::Form<SlackWebhookForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("notifications/slack_webhook");

    view.set("form", &form);

    let project_id = path.into_inner();
    let mut project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    if let Some(fields) = form.map(|f| f.into_inner()) {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        if fields.webhook_url.is_some() {
            view.message("Slack webhook set");
        } else {
            view.message("Slack webhook removed");
        }

        let mut project_model = project.into_active_model();
        project_model.slack_webhook = ActiveValue::set(fields.webhook_url);
        project = project_model.save(&ctx.db).await?.try_into_model()?;
    } else {
        view.set("form", SlackWebhookForm {
            webhook_url: project.slack_webhook.clone(),
        });
    }

    view.set("project", &project);

    Ok(view)
}

fn empty_string_as_none<'de, D, T>(de: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_ref().map(String::as_str);
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

#[post("/notifications/slack-test-webhook/{project_id}")]
async fn slack_test_webhook(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    let Some(webhook_url) = project.slack_webhook else {
        view.message("Slack not configured");
        return Ok(view);
    };

    let mut params = HashMap::new();
    params.insert("text", "Slack is working! I'll post here when your app panic!()s");

    let client = reqwest::Client::new();
    let res = client.post(webhook_url).json(&params).send().await?;

    log::info!("Slack test response: {:?}", res.text().await);

    view.message("Message sent");

    Ok(view)
}

#[derive(Serialize, Deserialize, Debug, Validate)]
struct WebhookForm {
    #[serde(deserialize_with = "empty_string_as_none")]
    #[validate(url(message = "Please enter a valid URL"))]
    webhook_url: Option<String>,
}

#[route("/notifications/webhook/{project_id}", method = "GET", method = "POST")]
async fn webhook(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>, form: Option<web::Form<WebhookForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("notifications/webhook");

    view.set("form", &form);

    let project_id = path.into_inner();
    let mut project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    if let Some(fields) = form.map(|f| f.into_inner()) {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        if fields.webhook_url.is_some() {
            view.message("Slack webhook set");
        } else {
            view.message("Slack webhook removed");
        }

        let mut project_model = project.into_active_model();
        project_model.webhook = ActiveValue::set(fields.webhook_url);
        project = project_model.save(&ctx.db).await?.try_into_model()?;
    } else {
        view.set("form", SlackWebhookForm {
            webhook_url: project.webhook.clone(),
        });
    }

    view.set("project", &project);

    Ok(view)
}

#[post("/notifications/test-webhook/{project_id}")]
async fn test_webhook(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id).one(&ctx.db).await?.ok_or(Error::NotFound)?;

    let user = identity.user(&ctx).await?;
    let _ = user.role(&ctx.db, project.organization_id).await?.ok_or(Error::LoginRequired)?;

    let Some(webhook_url) = project.webhook else {
        view.message("Webhook not configured");
        return Ok(view);
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

    view.message("Webhook called");

    Ok(view)
}
