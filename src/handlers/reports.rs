use std::collections::HashMap;

use actix_web::{
    delete, get, post,
    web::{self, Data, Json, Query},
    Responder,
};
use anyhow::anyhow;
use chrono::prelude::*;
use sea_orm::{
    prelude::*, ActiveValue, Condition, IntoActiveModel, JoinType, Order, QueryOrder, QuerySelect, QueryTrait,
};
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsForm;

use crate::entity::{
    organization_users, organizations, prelude::*, project_environments, project_report_events, project_reports,
    projects,
};

use crate::event::EventData;
use crate::{AppContext, Error, Identity, Result, ViewModel};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list).service(delete).service(resolve);
    // cfg.service(reports)
    //     .service(report_view)
    //     .service(report_event)
    //     .service(reports_delete)
    //     .service(reports_resolve);
}

#[derive(Serialize)]
struct ReportSummary {
    report: project_reports::Model,
    project: Option<projects::Model>,
    env: Option<project_environments::Model>,
}

#[derive(Serialize)]
struct ListResult {
    reports: Vec<ReportSummary>,
    next: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Cursor {
    project_report_id: u32,
    seen: i8,
    last_seen: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
struct ReportsQuery {
    cursor: Option<String>,
    project_id: Option<u32>,
    term: Option<String>,
    resolved: Option<u32>,
}

#[get("")]
async fn list(ctx: Data<AppContext<'_>>, id: Identity, q: Query<ReportsQuery>) -> Result<impl Responder> {
    let q = q.into_inner();

    let resolved = q.resolved.unwrap_or_default();
    let cursor = q.cursor.as_ref().and_then(|v| serde_json::from_str::<Cursor>(v).ok());

    let reports_and_envs = ProjectReports::find()
        .filter(organization_users::Column::UserId.eq(id.user_id))
        .filter(project_reports::Column::IsResolved.eq(resolved))
        .apply_if(q.project_id, |query, v| query.filter(projects::Column::ProjectId.eq(v)))
        .apply_if(q.term.as_ref().filter(|v| !v.is_empty()), |query, v| {
            query.filter(
                Condition::any()
                    .add(project_reports::Column::Title.contains(v))
                    .add(project_environments::Column::Name.contains(v)),
            )
        })
        .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
        .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .order_by(project_reports::Column::IsSeen, Order::Asc)
        .order_by(project_reports::Column::LastSeen, Order::Desc)
        .order_by(project_reports::Column::ProjectReportId, Order::Desc)
        .find_also_related(ProjectEnvironments)
        .apply_if(cursor, |query, cursor| {
            query.filter(
                Condition::any()
                    .add(project_reports::Column::IsSeen.gt(cursor.seen))
                    .add(
                        Condition::all()
                            .add(project_reports::Column::IsSeen.eq(cursor.seen))
                            .add(project_reports::Column::LastSeen.lt(cursor.last_seen)),
                    )
                    .add(
                        Condition::all()
                            .add(project_reports::Column::IsSeen.eq(cursor.seen))
                            .add(project_reports::Column::LastSeen.eq(cursor.last_seen))
                            .add(project_reports::Column::ProjectReportId.lt(cursor.project_report_id)),
                    ),
            )
        })
        .limit(11)
        .all(&ctx.db)
        .await?;

    let mut reports: Vec<ReportSummary> = vec![];

    let mut next = None;

    let count = reports_and_envs.len();

    for (report, env) in reports_and_envs.into_iter().take(10) {
        let project = if q.project_id.is_none() {
            report.find_related(Projects).one(&ctx.db).await?
        } else {
            None
        };

        next = Some(serde_json::to_string(&Cursor {
            project_report_id: report.project_report_id,
            seen: report.is_seen,
            last_seen: report.last_seen,
        })?);

        reports.push(ReportSummary { report, env, project });
    }

    if count < 11 {
        next = None;
    }

    Ok(Json(ListResult { reports, next }))
}

#[post("/delete")]
async fn delete(ctx: Data<AppContext<'_>>, id: Identity, report_ids: Json<Vec<u32>>) -> Result<impl Responder> {
    let report_ids = report_ids.into_inner();

    // make sure the user owns those reports
    let owned_reports: Vec<u32> = ProjectReports::find()
        .select_only()
        .column(project_reports::Column::ProjectReportId)
        .filter(project_reports::Column::ProjectReportId.is_in(report_ids))
        .filter(organization_users::Column::UserId.eq(id.user_id))
        .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
        .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .into_tuple()
        .all(&ctx.db)
        .await?;

    let res = ProjectReports::delete_many()
        .filter(project_reports::Column::ProjectReportId.is_in(owned_reports))
        .exec(&ctx.db)
        .await?;

    Ok(Json(serde_json::json!({
        "deleted": res.rows_affected,
    })))
}

#[post("/resolve")]
async fn resolve(ctx: Data<AppContext<'_>>, id: Identity, report_ids: Json<Vec<u32>>) -> Result<impl Responder> {
    let report_ids = report_ids.into_inner();

    // make sure the user owns those reports
    let owned_reports: Vec<u32> = ProjectReports::find()
        .select_only()
        .column(project_reports::Column::ProjectReportId)
        .filter(project_reports::Column::ProjectReportId.is_in(report_ids))
        .filter(organization_users::Column::UserId.eq(id.user_id))
        .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
        .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .into_tuple()
        .all(&ctx.db)
        .await?;

    let res = ProjectReports::update_many()
        .col_expr(project_reports::Column::IsResolved, Expr::value(1))
        .filter(project_reports::Column::ProjectReportId.is_in(owned_reports))
        .exec(&ctx.db)
        .await?;

    Ok(Json(serde_json::json!({
        "deleted": res.rows_affected,
    })))
}

// #[derive(Serialize)]
// struct ReportSummary {
//     report: project_reports::Model,
//     project: Option<projects::Model>,
//     env: Option<project_environments::Model>,
// }

// #[derive(Deserialize)]
// struct ReportsQuery {
//     page: Option<u64>,
//     project_id: Option<u32>,
//     term: Option<String>,
//     resolved: Option<u32>,
// }

// #[get("/reports")]
// async fn reports(ctx: web::Data<AppContext<'_>>, identity: Identity, query: web::Query<ReportsQuery>) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("reports/list");

//     let resolved = query.resolved.unwrap_or_default();

//     view.set("project_id", query.project_id);
//     view.set("term", query.term.clone());
//     view.set("resolved", resolved);

//     let mut page = query.page.unwrap_or(1);
//     let reports_per_page = 10;

//     let user = Users::find_by_id(identity.user_id).one(&ctx.db).await?.ok_or(Error::LoginRequired)?;
//     view.set("user", user);

//     let reports_paginator = ProjectReports::find()
//         .filter(organization_users::Column::UserId.eq(identity.user_id))
//         .filter(project_reports::Column::IsResolved.eq(resolved))
//         .apply_if(query.project_id, |query, v| query.filter(projects::Column::ProjectId.eq(v)))
//         .apply_if(query.term.as_ref().filter(|v| !v.is_empty()), |query, v| {
//             query.filter(
//                 Condition::any()
//                     .add(project_reports::Column::Title.contains(v))
//                     .add(project_environments::Column::Name.contains(v)),
//             )
//         })
//         .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
//         .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
//         .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
//         .order_by(project_reports::Column::IsSeen, Order::Asc)
//         .order_by(project_reports::Column::LastSeen, Order::Desc)
//         .order_by(project_reports::Column::ProjectReportId, Order::Desc)
//         .find_also_related(ProjectEnvironments)
//         .paginate(&ctx.db, reports_per_page);

//     let num_pages = reports_paginator.num_pages().await?;

//     if page > num_pages {
//         page = 1;
//     }

//     let mut reports: Vec<ReportSummary> = vec![];

//     for (report, env) in reports_paginator.fetch_page(page - 1).await? {
//         let project = if query.project_id.is_none() {
//             report.find_related(Projects).one(&ctx.db).await?
//         } else {
//             None
//         };

//         reports.push(ReportSummary { report, env, project });
//     }

//     view.set("reports", reports);
//     view.set("page", page);

//     if page > 1 {
//         view.set("prev_page", page - 1);
//     }

//     if page < num_pages {
//         view.set("next_page", page + 1);
//     }

//     Ok(view)
// }

// #[derive(Serialize)]
// struct DayEvents {
//     events_count: usize,
//     date: NaiveDate,
// }

// #[derive(Serialize)]
// struct WeekEvents {
//     month_label: Option<String>,
//     days: Vec<DayEvents>,
// }

// #[derive(Deserialize)]
// struct ReportViewQuery {
//     event_id: Option<u32>,
//     back_url: Option<String>,
// }

// #[get("/reports/view/{project_report_id}")]
// async fn report_view(ctx: web::Data<AppContext<'_>>, identity: Identity, project_report_id: web::Path<u32>, query: web::Query<ReportViewQuery>) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("reports/view");

//     let user = Users::find_by_id(identity.user_id).one(&ctx.db).await?.ok_or(Error::LoginRequired)?;
//     view.set("user", user);

//     view.set("event_id", query.event_id.unwrap_or_default());

//     let report_id = project_report_id.into_inner();

//     let report = ProjectReports::find_by_id(report_id)
//         .filter(organization_users::Column::UserId.eq(identity.user_id))
//         .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
//         .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
//         .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
//         .one(&ctx.db)
//         .await?
//         .ok_or(Error::NotFound)?;

//     if report.is_seen == 0 {
//         let mut report_model = report.clone().into_active_model();
//         report_model.is_seen = ActiveValue::set(1);
//         report_model.save(&ctx.db).await?;
//     }

//     view.set("back_url", query.back_url.clone().unwrap_or_else(|| format!("/reports?project_id={}", report.project_id)));

//     let env = report.find_related(ProjectEnvironments).one(&ctx.db).await?;
//     let project = report.find_related(Projects).one(&ctx.db).await?.ok_or(anyhow!("Report project not found"))?;
//     let org = project.find_related(Organizations).one(&ctx.db).await?;

//     view.set("report", report);
//     view.set("env", env);
//     view.set("project", project);
//     view.set("org", org);

//     // last year stats
//     let stats: HashMap<String, i64> = ProjectReportEvents::find()
//         .select_only()
//         .column_as(Expr::cust("substring(created,1,10)"), "date_created")
//         .column_as(project_report_events::Column::ProjectReportEventId.count(), "count")
//         .filter(project_report_events::Column::ProjectReportId.eq(report_id))
//         .group_by(Expr::cust("substring(created,1,10)"))
//         .into_tuple()
//         .all(&ctx.db)
//         .await?
//         .into_iter()
//         .collect();

//     let today = Utc::now().date_naive();

//     let mut current_week = vec![];
//     let mut weeks = vec![];
//     let mut prev_date = today;
//     let mut month_label = None;
//     let mut max_events = 0;

//     for d in today.iter_days().rev().take(365) {
//         let ymd = d.format("%Y-%m-%d").to_string();
//         let events_count = stats.get(&ymd).copied().unwrap_or_default() as usize;
//         max_events = events_count.max(max_events);
//         current_week.push(DayEvents { events_count, date: d });

//         if prev_date.month() != d.month() {
//             month_label = Some(prev_date.format("%b").to_string());
//         }

//         if d.weekday() == Weekday::Mon {
//             current_week.reverse();

//             weeks.push(WeekEvents {
//                 month_label: month_label.take(),
//                 days: std::mem::take(&mut current_week),
//             });
//         }

//         prev_date = d;
//     }

//     weeks.reverse();
//     view.set("occurrences", weeks);
//     view.set("max_occurrences", max_events);

//     Ok(view)
// }

// #[get("/reports/{report_id}/get-event")]
// async fn report_event(ctx: web::Data<AppContext<'_>>, identity: Identity, path: web::Path<u32>, query: web::Query<ReportViewQuery>) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("reports/event");

//     let user = Users::find_by_id(identity.user_id).one(&ctx.db).await?.ok_or(Error::LoginRequired)?;
//     view.set("user", user);

//     let report_id = path.into_inner();
//     let event_id = query.event_id.filter(|id| *id != 0);

//     view.set("report_id", report_id);

//     // make sure this report exists and is owned by the currently logged user
//     let report = ProjectReports::find_by_id(report_id)
//         .filter(organization_users::Column::UserId.eq(identity.user_id))
//         .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
//         .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
//         .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
//         .one(&ctx.db)
//         .await?
//         .ok_or(Error::NotFound)?;

//     view.set("back_url", query.back_url.clone().unwrap_or_else(|| format!("/reports?project_id={}", report.project_id)));

//     let maybe_event = if let Some(event_id) = event_id {
//         ProjectReportEvents::find_by_id(event_id)
//             .filter(project_report_events::Column::ProjectReportId.eq(report_id))
//             .one(&ctx.db)
//             .await?
//     } else {
//         ProjectReportEvents::find()
//             .filter(project_report_events::Column::ProjectReportId.eq(report_id))
//             .order_by(project_report_events::Column::ProjectReportEventId, Order::Desc)
//             .one(&ctx.db)
//             .await?
//     };

//     let Some(event) = maybe_event else {
//         return Ok(ViewModel::with_template("reports/no_event"));
//     };

//     let events_count: u64 = ProjectReportEvents::find()
//         .filter(project_report_events::Column::ProjectReportId.eq(report_id))
//         .count(&ctx.db)
//         .await?;

//     view.set("events_count", events_count);

//     let event_pos: u64 = ProjectReportEvents::find()
//         .filter(project_report_events::Column::ProjectReportId.eq(report_id))
//         .filter(project_report_events::Column::ProjectReportEventId.lte(event.project_report_event_id))
//         .count(&ctx.db)
//         .await?;

//     view.set("event_pos", event_pos);

//     let data: EventData = serde_json::from_str(&event.event_data)?;

//     view.set("data", data);
//     view.set("event", event);

//     Ok(view)
// }

// #[derive(Deserialize, Debug)]
// struct ReportsList {
//     report_ids: Option<Vec<u32>>,
// }

// #[delete("/reports")]
// async fn reports_delete(ctx: web::Data<AppContext<'_>>, identity: Identity, form: QsForm<ReportsList>) -> Result<ViewModel> {
//     let mut view = ViewModel::default();

//     let form = form.into_inner();

//     if let Some(ids) = form.report_ids {
//         // make sure the user owns those reports
//         let owned_reports: Vec<u32> = ProjectReports::find()
//             .select_only()
//             .column(project_reports::Column::ProjectReportId)
//             .filter(project_reports::Column::ProjectReportId.is_in(ids))
//             .filter(organization_users::Column::UserId.eq(identity.user_id))
//             .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
//             .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
//             .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
//             .into_tuple()
//             .all(&ctx.db)
//             .await?;

//         let res = ProjectReports::delete_many()
//             .filter(project_reports::Column::ProjectReportId.is_in(owned_reports))
//             .exec(&ctx.db)
//             .await?;

//         view.message(format!("{} reports deleted", res.rows_affected));
//     }

//     Ok(view)
// }

// #[post("/reports/resolve")]
// async fn reports_resolve(ctx: web::Data<AppContext<'_>>, identity: Identity, form: QsForm<ReportsList>) -> Result<ViewModel> {
//     let mut view = ViewModel::default();

//     let form = form.into_inner();

//     if let Some(ids) = form.report_ids {
//         // make sure the user owns those reports
//         let owned_reports: Vec<u32> = ProjectReports::find()
//             .select_only()
//             .column(project_reports::Column::ProjectReportId)
//             .filter(project_reports::Column::ProjectReportId.is_in(ids))
//             .filter(organization_users::Column::UserId.eq(identity.user_id))
//             .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
//             .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
//             .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
//             .into_tuple()
//             .all(&ctx.db)
//             .await?;

//         let res = ProjectReports::update_many()
//             .col_expr(project_reports::Column::IsResolved, Expr::value(1))
//             .filter(project_reports::Column::ProjectReportId.is_in(owned_reports))
//             .exec(&ctx.db)
//             .await?;

//         view.message(format!("{} reports resolved", res.rows_affected));
//     }

//     Ok(view)
// }
