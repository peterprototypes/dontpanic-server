use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, Order, QueryOrder, TryIntoModel};
use serde::Deserialize;

use crate::entity::prelude::*;
use crate::entity::project_environments;
use crate::entity::project_report_events;
use crate::entity::project_reports;
use crate::entity::projects;

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
    #[serde(rename(deserialize = "env"))]
    env_ident: Option<String>,
    name: String,
    data: EventData,
}

#[post("/ingress")]
async fn ingress(ctx: web::Data<AppContext<'static>>, event: web::Json<Event>) -> Result<HttpResponse> {
    let event = event.into_inner();

    let maybe_project = Projects::find().filter(projects::Column::ApiKey.eq(&event.api_key)).one(&ctx.db).await?;

    let Some(project) = maybe_project else {
        return Err(Error::new("API key not found or organization disabled"));
    };

    // find environment or create id
    let env_id = if let Some(env_ident) = event.env_ident {
        let maybe_env = ProjectEnvironments::find().filter(project_environments::Column::Name.eq(&env_ident)).one(&ctx.db).await?;

        let crate_env = match maybe_env {
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

        Some(crate_env.project_environment_id)
    } else {
        None
    };

    // find relevant report or create it
    let maybe_report = ProjectReports::find().filter(project_reports::Column::Title.eq(&event.name)).one(&ctx.db).await?;

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
            report_model
        }
        None => {
            report_status = Some(ReportStatus::New);

            // new issue
            project_reports::ActiveModel {
                project_id: ActiveValue::set(project.project_id),
                title: ActiveValue::set(event.name),
                project_environment_id: ActiveValue::set(env_id),
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
        ctx.notifications.send(Notification { status, project, event, report })?;
    }

    Ok(HttpResponse::Ok().finish())
}
