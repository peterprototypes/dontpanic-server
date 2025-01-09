use std::hash::{DefaultHasher, Hash, Hasher};

use actix_web::{post, web, HttpResponse};
use chrono::Days;
use chrono::TimeDelta;
use chrono::Utc;
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, JoinType, Order, QueryOrder, QuerySelect, TryIntoModel};
use serde::Deserialize;

use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::project_environments;
use crate::entity::project_report_events;
use crate::entity::project_reports;
use crate::entity::projects;

use crate::entity::users;
use crate::event::EventData;
use crate::notifications::{Notification, ReportStatus};
use crate::{AppContext, Error, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(ingress);
}

#[derive(Deserialize, Debug)]
struct Event {
    #[serde(rename(deserialize = "key"))]
    api_key: String,
    env: Option<String>,
    uid: String,
    data: EventData,
}

#[post("/ingress")]
async fn ingress(ctx: web::Data<AppContext<'static>>, event: web::Json<Event>) -> Result<HttpResponse> {
    let event = event.into_inner();

    let maybe_project = Projects::find()
        .filter(projects::Column::ApiKey.eq(&event.api_key))
        .one(&ctx.db)
        .await?;

    let Some(project) = maybe_project else {
        return Err(Error::new("API key not found or organization disabled"));
    };

    // make sure to handle events sequentially per project
    let _lock = ctx.locked_projects.lock(project.project_id).await;

    // limits check
    // ideally this check should not be in the hot path - it should happen on a timer once an hour and disable all projects in the org
    let org = project
        .find_related(Organizations)
        .one(&ctx.db)
        .await?
        .expect("Each project must have organization");

    if let Some(request_limit) = org.requests_limit {
        let mut request_count = org.requests_count.unwrap_or_default();

        let now = Utc::now().naive_utc();
        let start_date = org.requests_count_start.unwrap_or_default();

        // reset request_count if 30 days have passed and set requests_count_start to today
        if now - start_date > TimeDelta::days(30) {
            request_count = 0;

            let mut row = org.clone().into_active_model();
            row.requests_count_start = ActiveValue::set(Some(now));
            row.save(&ctx.db).await?;
        }

        // send an email one request before the limit is reached
        if request_limit - 1 == request_count {
            let bg_ctx = ctx.clone();
            let bg_org = org.clone();

            actix_web::rt::spawn(async move {
                notify_limit_reached(&bg_ctx, &bg_org).await.expect("Cannot send mail");
            });
        }

        if request_count >= request_limit {
            return Err(Error::new("Organization requests limit exceeded"));
        } else {
            let mut row = org.into_active_model();
            row.requests_count = ActiveValue::set(Some(request_count + 1));
            row.save(&ctx.db).await?;
        }
    }

    // find environment or create id
    let environment = if let Some(env_ident) = event.env {
        let maybe_env = ProjectEnvironments::find()
            .filter(project_environments::Column::Name.eq(&env_ident))
            .one(&ctx.db)
            .await?;

        let environment = match maybe_env {
            Some(env_row) => env_row,
            None => {
                let env_row = project_environments::ActiveModel {
                    project_id: ActiveValue::set(project.project_id),
                    name: ActiveValue::set(env_ident.clone()),
                    ..Default::default()
                };

                env_row.save(&ctx.db).await?.try_into_model()?
            }
        };

        Some(environment)
    } else {
        None
    };

    let env_hash = {
        let mut s = DefaultHasher::new();
        environment
            .as_ref()
            .map(|e| e.name.as_str())
            .unwrap_or_default()
            .hash(&mut s);
        s.finish()
    };

    let uid = format!("p{}-{}-{}", project.project_id, env_hash, event.uid);

    // find relevant report or create it
    let maybe_report = ProjectReports::find()
        .filter(project_reports::Column::Uid.eq(&uid))
        .one(&ctx.db)
        .await?;

    let mut report_status: Option<ReportStatus> = None;

    let report_model = match maybe_report {
        Some(report) => {
            if report.is_resolved > 0 {
                // issue marked as resolved, but reappears again
                report_status = Some(ReportStatus::Regressed);
            }

            let mut report_model = report.into_active_model();
            report_model.last_seen = ActiveValue::set(Utc::now().naive_utc());
            report_model.is_resolved = ActiveValue::set(0);
            report_model.is_seen = ActiveValue::set(0);
            report_model
        }
        None => {
            report_status = Some(ReportStatus::New);

            // new issue
            project_reports::ActiveModel {
                project_id: ActiveValue::set(project.project_id),
                uid: ActiveValue::set(uid),
                title: ActiveValue::set(event.data.title()),
                project_environment_id: ActiveValue::set(environment.as_ref().map(|e| e.project_environment_id)),
                ..Default::default()
            }
        }
    };

    let report = report_model.save(&ctx.db).await?.try_into_model()?;

    // get last event for this report
    let prev_event = ProjectReportEvents::find()
        .filter(project_report_events::Column::ProjectReportId.eq(report.project_report_id))
        .order_by(project_report_events::Column::ProjectReportEventId, Order::Desc)
        .one(&ctx.db)
        .await?;

    // create event
    let event = project_report_events::ActiveModel {
        project_report_id: ActiveValue::set(report.project_report_id),
        prev_event_id: ActiveValue::set(prev_event.as_ref().map(|e| e.project_report_event_id)),
        event_data: ActiveValue::set(serde_json::to_string(&event.data)?),
        ..Default::default()
    };

    let event = event.insert(&ctx.db).await?;

    if let Some(prev_event) = prev_event {
        let mut prev_event = prev_event.into_active_model();
        prev_event.next_event_id = ActiveValue::set(Some(event.project_report_event_id));
        prev_event.save(&ctx.db).await?;
    }

    if let Some(status) = report_status {
        ctx.notifications.send(Notification {
            status,
            project,
            event,
            report,
            environment,
        })?;
    }

    Ok(HttpResponse::Ok().finish())
}

async fn notify_limit_reached(ctx: &AppContext<'_>, org: &organizations::Model) -> Result<()> {
    let owners = Users::find()
        .filter(organization_users::Column::OrganizationId.eq(org.organization_id))
        .filter(organization_users::Column::Role.eq("owner"))
        .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
        .all(&ctx.db)
        .await?;

    let reset_date = org.requests_count_start.map(|date| date + Days::new(30));

    for user in owners {
        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(user.email.parse()?)
            .subject("Monthly Request Limit Reached for Your Organization")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/limit_reached",
                &serde_json::json!({
                    "base_url": ctx.config.base_url,
                    "scheme": ctx.config.scheme,
                    "user": user,
                    "reset_date": reset_date,
                    "title": "Monthly Request Limit Reached for Your Organization"
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            if let Err(e) = mailer.send(email).await {
                log::error!("Error sending mail: {:?}", e);
            }
        }
    }

    Ok(())
}
