use anyhow::Result;
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::entity::{
    prelude::*, project_environments, project_report_events, project_reports, project_user_settings, projects, users,
};
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

    let report_url = format!(
        "{}://{}/reports/view/{}",
        ctx.config.scheme, ctx.config.base_url, notification.report.project_report_id
    );

    for (user, maybe_settings) in users {
        if let Some(settings) = maybe_settings {
            if settings.notify_email > 0 {
                if let Err(e) = send_email(ctx, notification, &user, &report_url).await {
                    log::error!("Error sending notification email: {:?}", e);
                }
            }

            if settings.notify_pushover > 0 {
                if let Err(e) = send_pushover(ctx, notification, &user, &report_url).await {
                    log::error!("Error sending pushover notification: {:?}", e);
                }
            }
        }
    }

    if let Err(e) = send_slack(ctx, notification, &report_url).await {
        log::error!("Error sending slack app message: {:?}", e);
    }

    if let Err(e) = send_slack_webhook(ctx, notification, &report_url).await {
        log::error!("Error sending slack message via webhook: {:?}", e);
    }

    if let Err(e) = send_webhook(ctx, notification, &report_url).await {
        log::error!("Error sending report via webhook: {:?}", e);
    }

    Ok(())
}

pub async fn send_slack(_ctx: &AppContext<'_>, notification: &Notification, report_url: &str) -> Result<()> {
    let project = &notification.project;

    let Some((token, channel)) = project.slack_bot_token.as_deref().zip(project.slack_channel.as_deref()) else {
        return Ok(());
    };

    let mut params = get_slack_blocks(notification, report_url);
    params["token"] = token.into();
    params["channel"] = channel.into();

    let client = reqwest::Client::new();
    client
        .post("https://slack.com/api/chat.postMessage")
        .form(&params)
        .send()
        .await?;

    // TODO: log error response

    Ok(())
}

pub async fn send_slack_webhook(_ctx: &AppContext<'_>, notification: &Notification, report_url: &str) -> Result<()> {
    let project = &notification.project;

    let Some(webhook) = project.slack_webhook.as_ref() else {
        return Ok(());
    };

    let mut params = get_slack_blocks(notification, report_url);
    params["username"] = "Don't Panic".into();
    params["icon_url"] = "https://dontpanic.rs/static/favicon.png".into();

    let client = reqwest::Client::new();
    client.post(webhook).json(&params).send().await?;

    // TODO: log error response

    Ok(())
}

fn get_slack_blocks(notification: &Notification, report_url: &str) -> serde_json::Value {
    let mut title = if notification.status == ReportStatus::New {
        format!(
            ":boom: New report on {} received {}",
            notification.project.name, notification.report.title
        )
    } else {
        format!(
            "Resolved report on {} reappeared: {}",
            notification.project.name, notification.report.title
        )
    };

    let mut markdown = if notification.status == ReportStatus::New {
        format!(
            ":boom: New report on *{}* received {}",
            notification.project.name, notification.report.title
        )
    } else {
        format!(
            "Resolved report on *{}* reappeared: {}",
            notification.project.name, notification.report.title
        )
    };

    if let Some(environment) = notification.environment.as_ref() {
        title.push_str(&format!(" in {}", environment.name));
        markdown.push_str(&format!(" in *{}*", environment.name));
    }

    json!({
        "text": title,
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": markdown
                }
            },
            {
                "type": "actions",
                "elements": [
                    {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "View in Don't Panic"
                        },
                        "url": report_url
                    }
                ]
            }
        ]
    })
}

pub async fn send_webhook(_ctx: &AppContext<'_>, notification: &Notification, report_url: &str) -> Result<()> {
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
        "url": report_url,
    });

    let client = reqwest::Client::new();
    client.post(webhook).json(&params).send().await?;

    Ok(())
}

pub async fn send_email(
    ctx: &AppContext<'_>,
    notification: &Notification,
    user: &users::Model,
    report_url: &str,
) -> Result<()> {
    let template = if notification.status == ReportStatus::New {
        "email/new_report"
    } else {
        "email/regressed_report"
    };

    let mut title = if notification.status == ReportStatus::New {
        format!(
            "New report on {} received '{}'",
            notification.project.name, notification.report.title
        )
    } else {
        format!(
            "Resolved report on {} reappeared: '{}'",
            notification.project.name, notification.report.title
        )
    };

    if let Some(environment) = notification.environment.as_ref() {
        title.push_str(&format!(" in {}", environment.name));
    }

    let data = serde_json::json!({
        "title": &title,
        "report": notification.report,
        "project": notification.project,
        "report_url": report_url,
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

pub async fn send_pushover(
    ctx: &AppContext<'_>,
    notification: &Notification,
    user: &users::Model,
    report_url: &str,
) -> Result<()> {
    let Some((token, user_key)) = ctx
        .config
        .pushover_app_token
        .as_deref()
        .zip(user.pushover_user_key.as_deref())
    else {
        return Ok(());
    };

    let mut message = if notification.status == ReportStatus::New {
        format!(
            "New report on {} received '{}'",
            notification.project.name, notification.report.title
        )
    } else {
        format!(
            "Resolved report on {} reappeared: '{}'",
            notification.project.name, notification.report.title
        )
    };

    if let Some(environment) = notification.environment.as_ref() {
        message.push_str(&format!(" in {}", environment.name));
    }

    let client = reqwest::Client::new();

    let res = client
        .post("https://api.pushover.net/1/messages.json")
        .form(&[
            ("token", token),
            ("user", user_key),
            ("message", &message),
            ("url", report_url),
        ])
        .send()
        .await?;

    if !res.status().is_success() {
        let body = res.text().await?;
        log::error!("Error sending pushover notification: {}", body);
    }

    Ok(())
}
