use std::collections::HashMap;

use anyhow::Result;
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::entity::{prelude::*, project_environments, project_report_events, project_reports, project_user_settings, projects, users};
use crate::AppContext;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReportStatus {
    New,
    Regressed,
}

#[derive(Serialize, Debug, Clone)]
pub struct Notification {
    pub status: ReportStatus,
    pub project: projects::Model,
    pub event: project_report_events::Model,
    pub report: project_reports::Model,
    pub environment: Option<project_environments::Model>,
}

pub async fn send(ctx: &AppContext<'_>, notification: &Notification) -> Result<()> {
    let users: Vec<(users::Model, Option<project_user_settings::Model>)> = Users::find()
        .filter(project_user_settings::Column::ProjectId.eq(notification.project.project_id))
        .find_also_related(ProjectUserSettings)
        .all(&ctx.db)
        .await?;

    for (user, maybe_settings) in users {
        if let Some(settings) = maybe_settings {
            if settings.notify_email > 0 {
                if let Err(e) = send_email(ctx, notification, user).await {
                    log::error!("Error sending notification email: {:?}", e);
                }
            }
        }
    }

    if let Err(e) = send_slack(ctx, notification).await {
        log::error!("Error sending slack app message: {:?}", e);
    }

    if let Err(e) = send_slack_webhook(ctx, notification).await {
        log::error!("Error sending slack message via webhook: {:?}", e);
    }

    if let Err(e) = send_webhook(ctx, notification).await {
        log::error!("Error sending report via webhook: {:?}", e);
    }

    Ok(())
}

pub async fn send_slack(_ctx: &AppContext<'_>, notification: &Notification) -> Result<()> {
    let project = &notification.project;

    let Some((token, channel)) = project.slack_bot_token.as_ref().zip(project.slack_channel.as_ref()) else {
        return Ok(());
    };

    let mut title = if notification.status == ReportStatus::New {
        format!("New report on {} received '{}'", notification.project.name, notification.report.title)
    } else {
        format!("Resolved report on {} reappeared: '{}'", notification.project.name, notification.report.title)
    };

    if let Some(environment) = notification.environment.as_ref() {
        title.push_str(&format!(" in {}", environment.name));
    }

    let params = [("token", token), ("channel", channel), ("text", &title)];

    let client = reqwest::Client::new();
    client.post("https://slack.com/api/chat.postMessage").form(&params).send().await?;

    // TODO: log error response

    Ok(())
}

pub async fn send_slack_webhook(_ctx: &AppContext<'_>, notification: &Notification) -> Result<()> {
    let project = &notification.project;

    let Some(webhook) = project.slack_webhook.as_ref() else {
        return Ok(());
    };

    let mut title = if notification.status == ReportStatus::New {
        format!("New report on {} received '{}'", notification.project.name, notification.report.title)
    } else {
        format!("Resolved report on {} reappeared: '{}'", notification.project.name, notification.report.title)
    };

    if let Some(environment) = notification.environment.as_ref() {
        title.push_str(&format!(" in {}", environment.name));
    }

    let mut params = HashMap::new();
    params.insert("username", "Don't Panic".to_string());
    params.insert("icon_url", "https://dontpanic.rs/static/favicon.png".to_string());
    params.insert("text", title);

    let client = reqwest::Client::new();
    client.post(webhook).json(&params).send().await?;

    // TODO: log error response

    Ok(())
}

pub async fn send_webhook(_ctx: &AppContext<'_>, notification: &Notification) -> Result<()> {
    let project = &notification.project;

    let Some(webhook) = project.webhook.as_ref() else {
        return Ok(());
    };

    let event: serde_json::Value = serde_json::from_str(&notification.event.event_data)?;

    let params = json!({
        "status": notification.status,
        "title": notification.report.title,
        "project": notification.project.name,
        "environment": notification.environment.as_ref().map(|e| &e.name),
        "event": event,
    });

    let client = reqwest::Client::new();
    client.post(webhook).json(&params).send().await?;

    Ok(())
}

pub async fn send_email(ctx: &AppContext<'_>, notification: &Notification, user: users::Model) -> Result<()> {
    let template = if notification.status == ReportStatus::New {
        "email/new_report"
    } else {
        "email/regressed_report"
    };

    let mut title = if notification.status == ReportStatus::New {
        format!("New report on {} received '{}'", notification.project.name, notification.report.title)
    } else {
        format!("Resolved report on {} reappeared: '{}'", notification.project.name, notification.report.title)
    };

    if let Some(environment) = notification.environment.as_ref() {
        title.push_str(&format!(" in {}", environment.name));
    }

    let data = serde_json::json!({
        "title": &title,
        "base_url": ctx.config.base_url,
        "scheme": ctx.config.scheme,
        "report": notification.report,
        "project": notification.project,
    });

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(user.email.parse()?)
        .subject(title)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(template, &data)?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    Ok(())
}
