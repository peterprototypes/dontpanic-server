use std::collections::HashMap;

use anyhow::Result;
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entity::{prelude::*, project_report_events, project_reports, project_user_settings, projects, users};
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

    Ok(())
}

pub async fn send_slack(_ctx: &AppContext<'_>, notification: &Notification) -> Result<()> {
    let project = &notification.project;

    let Some((token, channel)) = project.slack_bot_token.as_ref().zip(project.slack_channel.as_ref()) else {
        return Ok(());
    };

    let title = if notification.status == ReportStatus::New {
        format!("New report on {} received '{}'", notification.project.name, notification.report.title)
    } else {
        format!("Resolved report on {} reappeared: '{}'", notification.project.name, notification.report.title)
    };

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

    let title = if notification.status == ReportStatus::New {
        format!("New report on {} received '{}'", notification.project.name, notification.report.title)
    } else {
        format!("Resolved report on {} reappeared: '{}'", notification.project.name, notification.report.title)
    };

    let mut params = HashMap::new();
    params.insert("username", "Don't Panic".to_string());
    params.insert("icon_url", "https://dontpanic.rs/static/favicon.png".to_string());
    params.insert("text", title);

    let client = reqwest::Client::new();
    client.post(webhook).json(&params).send().await?;

    // TODO: log error response

    Ok(())
}

pub async fn send_email(ctx: &AppContext<'_>, notification: &Notification, user: users::Model) -> Result<()> {
    let template = if notification.status == ReportStatus::New {
        "email/new_report"
    } else {
        "email/regressed_report"
    };

    let title = if notification.status == ReportStatus::New {
        format!("New report on {} received '{}'", notification.project.name, notification.report.title)
    } else {
        format!("Resolved report on {} reappeared: '{}'", notification.project.name, notification.report.title)
    };

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
