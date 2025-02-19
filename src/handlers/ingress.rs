use std::hash::{DefaultHasher, Hash, Hasher};

use actix_web::{post, web, HttpResponse};
use chrono::prelude::*;
use lettre::AsyncTransport;
use sea_orm::prelude::*;
use sea_orm::sea_query;
use sea_orm::{ActiveValue, IntoActiveModel, JoinType, QueryOrder, QuerySelect, TryIntoModel};
use serde::{Deserialize, Serialize};

use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::project_environments;
use crate::entity::project_report_events;
use crate::entity::project_report_stats;
use crate::entity::project_reports;
use crate::entity::projects;
use crate::entity::{organization_stats, organization_users};

use crate::entity::users;
use crate::notifications::{Notification, ReportStatus};
use crate::{AppContext, Error, Result};

// To preserve backwards compatibility with any client version,
// only new, optional fields should be added to these structures

#[derive(Serialize, Deserialize, Debug)]
struct EventFileLocation {
    #[serde(rename = "f")]
    file: String,
    #[serde(rename = "l")]
    line: u32,
    #[serde(rename = "c")]
    column: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LogEvent {
    #[serde(rename = "ts")]
    timestamp: u64,
    #[serde(rename = "lvl")]
    level: u8,
    #[serde(rename = "msg")]
    message: String,
    #[serde(rename = "mod")]
    module: Option<String>,
    #[serde(rename = "f")]
    file: Option<String>,
    #[serde(rename = "l")]
    line: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventData {
    title: String,
    #[serde(rename = "loc")]
    location: Option<EventFileLocation>,
    #[serde(rename = "ver")]
    version: Option<String>,
    os: String,
    arch: String,
    #[serde(rename = "tid")]
    thread_id: Option<String>,
    #[serde(rename = "tname")]
    thread_name: Option<String>,
    #[serde(rename = "trace")]
    backtrace: String,
    #[serde(rename = "log")]
    log_messages: Vec<LogEvent>,
}

#[derive(Deserialize, Debug)]
struct Event {
    key: String,
    env: Option<String>,
    data: EventData,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(ingress);
}

#[post("/ingress")]
async fn ingress(ctx: web::Data<AppContext<'static>>, event: web::Json<Event>) -> Result<HttpResponse> {
    let event = event.into_inner();

    let maybe_project = Projects::find()
        .filter(projects::Column::ApiKey.eq(&event.key))
        .one(&ctx.db)
        .await?;

    let Some(project) = maybe_project else {
        return Err(Error::new("API key not found or organization disabled"));
    };

    // make sure to handle events sequentially per project
    let _lock = ctx.locked_projects.lock(project.project_id).await;

    // limits check
    let org = project
        .find_related(Organizations)
        .one(&ctx.db)
        .await?
        .expect("Each project must have organization");

    if org.is_enabled == 0 {
        return Err(Error::new("Organization requests limit exceeded"));
    }

    if let Some(request_limit) = org.requests_limit {
        let request_count = org.requests_count.unwrap_or_default();

        // send an email when 90% of the limit is reached
        if request_limit * 9 / 10 == request_count {
            let bg_ctx = ctx.clone();
            let bg_org = org.clone();

            actix_web::rt::spawn(async move {
                if let Err(e) = notify_limit_approaching(&bg_ctx, &bg_org).await {
                    log::error!("Error sending limit reached notification: {:?}", e);
                }
            });
        }

        if request_count >= request_limit {
            return Err(Error::new("Organization requests limit exceeded"));
        } else {
            let mut row = org.clone().into_active_model();
            row.requests_count = ActiveValue::set(Some(request_count + 1));
            row.save(&ctx.db).await?;
        }
    }

    record_org_stat(&ctx.db, org.organization_id, "event", "total_count").await?;

    // find environment or create it
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

    let environment_hash = {
        let mut s = DefaultHasher::new();
        environment
            .as_ref()
            .map(|e| e.name.as_str())
            .unwrap_or_default()
            .hash(&mut s);
        s.finish()
    };

    // Enforce event title limit. Varchar column limit counts in characters, not bytes
    let title_upto = event
        .data
        .title
        .char_indices()
        .enumerate()
        .map(|(char_idx, (byte_idx, _))| (char_idx, byte_idx))
        .find(|(i, _)| *i >= 496);

    let event_title = if let Some((_, byte_idx)) = title_upto {
        let mut title = event.data.title.clone();
        title.truncate(byte_idx);
        title.push_str("...");
        title
    } else {
        event.data.title.clone()
    };

    let event_uid = if let Some(location) = event.data.location.as_ref() {
        format!("{}-{}-{:?}", location.file, location.line, location.column)
    } else {
        event_title.clone()
    };

    let uid = format!("p{}-{}-{}", project.project_id, environment_hash, event_uid);

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

            record_org_stat(
                &ctx.db,
                org.organization_id,
                "new_project_report",
                &project.project_id.to_string(),
            )
            .await?;

            // new issue
            project_reports::ActiveModel {
                project_id: ActiveValue::set(project.project_id),
                uid: ActiveValue::set(uid),
                title: ActiveValue::set(event_title),
                project_environment_id: ActiveValue::set(environment.as_ref().map(|e| e.project_environment_id)),
                ..Default::default()
            }
        }
    };

    let report = report_model.save(&ctx.db).await?.try_into_model()?;

    // fill log messages from latest to oldest and limit to 65 000 characters
    let mut log_messages: Vec<String> = Vec::new();
    let mut log_messages_size = 2;

    for log_message in event.data.log_messages.into_iter().rev() {
        let message_serialized = serde_json::to_string(&log_message)?;
        log_messages_size += message_serialized.len() + 1;

        if log_messages_size > 65000 {
            break;
        }

        log_messages.push(message_serialized);
    }

    log_messages.reverse();

    // enforce log message memory limit
    // let log_messages: Vec<LogEvent> = event
    //     .data
    //     .log_messages
    //     .into_iter()
    //     .take(100)
    //     .map(|mut log_event| {
    //         let encoded = serde_json::to_string(&log_event.message).unwrap();
    //         let upto = encoded.char_indices().map(|(i, _)| i).find(|i| *i > 500);

    //         if let Some(upto) = upto {
    //             log_event.message.truncate(upto);
    //             log_event.message.push_str("...");
    //         }

    //         log_event
    //     })
    //     .collect::<Vec<_>>();

    // enforce backtrace limit of 10 000 characters, considering urf-8 codepoints and avoiding panics
    let backtrace = event.data.backtrace.chars().take(10000).collect::<String>();

    // create event
    let event_model = project_report_events::ActiveModel {
        project_report_id: ActiveValue::set(report.project_report_id),
        backtrace: ActiveValue::set(Some(backtrace)),
        log: ActiveValue::set(Some(format!("[{}]", log_messages.join(",")))),
        ..Default::default()
    };

    let event_row = event_model.insert(&ctx.db).await?;

    // retain only last 5 events for this report (this may be in a separate task)
    let events = ProjectReportEvents::find()
        .filter(project_report_events::Column::ProjectReportId.eq(report.project_report_id))
        .order_by_desc(project_report_events::Column::ProjectReportEventId)
        .all(&ctx.db)
        .await?;

    let events_to_del = events
        .iter()
        .skip(5)
        .map(|e| e.project_report_event_id)
        .collect::<Vec<_>>();

    if !events_to_del.is_empty() {
        ProjectReportEvents::delete_many()
            .filter(project_report_events::Column::ProjectReportEventId.is_in(events_to_del))
            .exec(&ctx.db)
            .await?;
    }

    let is_new_report = matches!(report_status, Some(ReportStatus::New) | Some(ReportStatus::Regressed));

    // Increment counters
    record_report_stat(&ctx.db, report.project_report_id, "event", "total_count", is_new_report).await?;
    record_report_stat(&ctx.db, report.project_report_id, "os", &event.data.os, is_new_report).await?;
    record_report_stat(
        &ctx.db,
        report.project_report_id,
        "arch",
        &event.data.arch,
        is_new_report,
    )
    .await?;

    if let Some(version) = event.data.version.as_ref() {
        record_report_stat(&ctx.db, report.project_report_id, "version", version, is_new_report).await?;
    }

    let res = ctx.notifications.send(Notification {
        status: report_status,
        project,
        event: event_row,
        report,
        environment,
    });

    if let Err(e) = res {
        log::error!("Error sending notification: {:?}", e);
    }

    Ok(HttpResponse::Ok().finish())
}

async fn notify_limit_approaching(ctx: &AppContext<'_>, org: &organizations::Model) -> Result<()> {
    let owners = Users::find()
        .filter(organization_users::Column::OrganizationId.eq(org.organization_id))
        .filter(organization_users::Column::Role.eq("owner"))
        .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
        .all(&ctx.db)
        .await?;

    for user in owners {
        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(user.email.parse()?)
            .subject("Don't Panic: Your Organization's API Quota is Running Low")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/limit_approaching",
                &serde_json::json!({
                    "base_url": ctx.config.base_url,
                    "scheme": ctx.config.scheme,
                    "user": user,
                    "org": org,
                    "title": "Don't Panic: Your Organization's API Quota is Running Low"
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

async fn record_report_stat(
    db: &DatabaseConnection,
    report_id: u32,
    category: &str,
    name: &str,
    is_new_report: bool,
) -> Result<()> {
    let current_hour = Utc::now()
        .date_naive()
        .and_hms_opt(Utc::now().hour(), 0, 0)
        .expect("valid time")
        .time();

    let stat = project_report_stats::ActiveModel {
        project_report_id: ActiveValue::set(report_id),
        category: ActiveValue::set(category.into()),
        name: ActiveValue::set(name.into()),
        count: ActiveValue::set(1),
        date: ActiveValue::set(Utc::now().date_naive().and_time(current_hour)),
        // first event is considered a spike
        spiking: ActiveValue::set(is_new_report as i8),
        ..Default::default()
    };

    ProjectReportStats::insert(stat)
        .on_conflict(
            sea_query::OnConflict::columns([
                project_report_stats::Column::ProjectReportId,
                project_report_stats::Column::Category,
                project_report_stats::Column::Name,
                project_report_stats::Column::Date,
            ])
            .value(
                project_report_stats::Column::Count,
                Expr::col(project_report_stats::Column::Count).add(1),
            )
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

async fn record_org_stat(db: &DatabaseConnection, organization_id: u32, category: &str, name: &str) -> Result<()> {
    let stat = organization_stats::ActiveModel {
        organization_id: ActiveValue::set(organization_id),
        category: ActiveValue::set(category.into()),
        name: ActiveValue::set(name.into()),
        count: ActiveValue::set(1),
        date: ActiveValue::set(Utc::now().date_naive().and_time(NaiveTime::default())),
        ..Default::default()
    };

    OrganizationStats::insert(stat)
        .on_conflict(
            sea_query::OnConflict::columns([
                organization_stats::Column::OrganizationId,
                organization_stats::Column::Category,
                organization_stats::Column::Name,
                organization_stats::Column::Date,
            ])
            .value(
                organization_stats::Column::Count,
                Expr::col(organization_stats::Column::Count).add(1),
            )
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_ingress_endpoint() {
        let (app, sess) = crate::test_app_with_auth().await.unwrap();

        // create
        let req = test::TestRequest::post()
            .uri("/api/organizations/1/projects")
            .cookie(sess.clone())
            .set_json(serde_json::json!({
                "name": "Test Project",
            }))
            .to_request();

        let res: Value = test::call_and_read_body_json(&app, req).await;
        let project_id = res["project_id"].as_u64().unwrap();
        let api_key = res["api_key"].as_str().unwrap();

        // test bad request
        let req = test::TestRequest::post()
            .uri("/ingress")
            .cookie(sess.clone())
            .set_json(serde_json::json!({
                "key": api_key,
            }))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // test good request
        let req = test::TestRequest::post()
            .uri("/ingress")
            .set_json(serde_json::json!({
                "key": api_key,
                "env": "production",
                "data": {
                    "title": "Test Error",
                    "trace": "backtrace",
                    "log": [
                        {
                            "msg": "Error message",
                            "lvl": 3,
                            "ts": 1738255164
                        }
                    ],
                    "os": "linux",
                    "arch": "x86_64",
                    "ver": "1.0.0",
                    "loc": {
                        "f": "main.rs",
                        "l": 10,
                        "c": 5
                    }
                }
            }))
            .to_request();

        let res = test::call_service(&app, req).await;
        // let body = test::read_body(res).await;
        assert_eq!(res.status(), StatusCode::OK);

        // test getting reports
        let req = test::TestRequest::get()
            .uri(&format!("/api/reports?project_id={}", project_id))
            .cookie(sess.clone())
            .to_request();

        let res: Value = test::call_and_read_body_json(&app, req).await;
        let report_id = res["reports"][0]["report"]["project_report_id"].as_u64().unwrap();

        assert_eq!(res["reports"][0]["report"]["title"], "Test Error");
        assert_eq!(res["reports"][0]["env"]["name"], "production");

        // get getting single report
        let req = test::TestRequest::get()
            .uri(&format!("/api/reports/{}", report_id))
            .cookie(sess.clone())
            .to_request();

        let res: Value = test::call_and_read_body_json(&app, req).await;
        let obj = res.as_object().unwrap();

        assert!(obj.contains_key("project"));
        assert!(obj.contains_key("report"));
        assert!(obj.contains_key("env"));
        assert!(obj.contains_key("org"));
        assert!(obj.contains_key("daily_events"));
        assert!(obj.contains_key("os_dataset"));
        assert!(obj.contains_key("os_names"));
        assert!(obj.contains_key("version_dataset"));
        assert!(obj.contains_key("version_names"));
        assert!(obj.contains_key("last_event"));
    }

    // #[actix_web::test]
    // async fn test_ingress_limits() {
    //     let (app, sess) = crate::test_app_with_auth().await.unwrap();

    //     // create
    //     let req = test::TestRequest::post()
    //         .uri("/api/organizations/1/projects")
    //         .cookie(sess.clone())
    //         .set_json(serde_json::json!({
    //             "name": "Test Project",
    //         }))
    //         .to_request();

    //     let res: Value = test::call_and_read_body_json(&app, req).await;
    //     let project_id = res["project_id"].as_u64().unwrap();
    //     let api_key = res["api_key"].as_str().unwrap();

    //     let mut log_messages = vec![];

    //     for i in 0..200 {
    //         let mut message = String::new();

    //         for j in 0..650 {
    //             message.push_str(&format!("{}\" ", j));
    //         }

    //         log_messages.push(serde_json::json!({
    //             "msg": message,
    //             "lvl": 3,
    //             "ts": 1738255164
    //         }));
    //     }

    //     // test good request
    //     let req = test::TestRequest::post()
    //         .uri("/ingress")
    //         .set_json(serde_json::json!({
    //             "key": api_key,
    //             "env": "production",
    //             "data": {
    //                 "title": "Test Error",
    //                 "trace": "backtrace",
    //                 "log": log_messages,
    //                 "os": "linux",
    //                 "arch": "x86_64",
    //                 "ver": "1.0.0",
    //                 "loc": {
    //                     "f": "main.rs",
    //                     "l": 10,
    //                     "c": 5
    //                 }
    //             }
    //         }))
    //         .to_request();

    //     let res = test::call_service(&app, req).await;
    //     // let body = test::read_body(res).await;
    //     assert_eq!(res.status(), StatusCode::OK);

    //     // test getting reports
    //     let req = test::TestRequest::get()
    //         .uri(&format!("/api/reports?project_id={}", project_id))
    //         .cookie(sess.clone())
    //         .to_request();

    //     let res: Value = test::call_and_read_body_json(&app, req).await;
    //     let report_id = res["reports"][0]["report"]["project_report_id"].as_u64().unwrap();

    //     assert_eq!(res["reports"][0]["report"]["title"], "Test Error");
    //     assert_eq!(res["reports"][0]["env"]["name"], "production");
    // }
}
