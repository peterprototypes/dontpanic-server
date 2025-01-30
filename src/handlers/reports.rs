use std::collections::{HashMap, HashSet};

use actix_web::{
    get, post,
    web::{self, Data, Json, Path, Query},
    Responder,
};

use chrono::prelude::*;
use chrono::Days;
use rust_decimal::prelude::*;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, Condition, IntoActiveModel, JoinType, Order, QueryOrder, QuerySelect, QueryTrait};
use serde::{Deserialize, Serialize};

use crate::entity::prelude::*;
use crate::entity::{
    organization_users, organizations, project_environments, project_report_events, project_report_stats,
    project_reports, projects,
};

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list).service(delete).service(resolve).service(get_report);
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

#[get("/{report_id}")]
async fn get_report(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
    let report_id = path.into_inner();

    let report = ProjectReports::find_by_id(report_id)
        .filter(organization_users::Column::UserId.eq(id.user_id))
        .join(JoinType::InnerJoin, project_reports::Relation::Projects.def())
        .join(JoinType::InnerJoin, projects::Relation::Organizations.def())
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    if report.is_seen == 0 {
        let mut report_model = report.clone().into_active_model();
        report_model.is_seen = ActiveValue::set(1);
        report_model.save(&ctx.db).await?;
    }

    let project = report
        .find_related(Projects)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let org = project.find_related(Organizations).one(&ctx.db).await?;
    let env = report.find_related(ProjectEnvironments).one(&ctx.db).await?;

    // number of events each day for the last 365 days
    let daily_events: HashMap<DateTimeUtc, u32> = ProjectReportStats::find()
        .select_only()
        .column(project_report_stats::Column::Date)
        .column(project_report_stats::Column::Count)
        .filter(project_report_stats::Column::ProjectReportId.eq(report_id))
        .filter(project_report_stats::Column::Category.eq("event"))
        .filter(project_report_stats::Column::Name.eq("total_count"))
        .filter(project_report_stats::Column::Date.gte(Utc::now().date_naive() - Days::new(365)))
        .into_tuple::<(DateTimeUtc, u32)>()
        .all(&ctx.db)
        .await?
        .into_iter()
        .collect();

    // OS and version stats
    let (os_dataset, os_names) = get_dataset(&ctx.db, report_id, "os").await?;
    let (version_dataset, version_names) = get_dataset(&ctx.db, report_id, "version").await?;

    // Last received log event
    let last_event = ProjectReportEvents::find()
        .filter(project_report_events::Column::ProjectReportId.eq(report_id))
        .order_by(project_report_events::Column::ProjectReportEventId, Order::Desc)
        .one(&ctx.db)
        .await?;

    Ok(Json(serde_json::json!({
        "project": project,
        "report": report,
        "env": env,
        "org": org,
        "daily_events": daily_events,
        "os_dataset": os_dataset,
        "os_names": os_names,
        "version_dataset": version_dataset,
        "version_names": version_names,
        "last_event": last_event,
    })))
}

use sea_orm::sea_query::Alias;

async fn get_dataset(
    db: &DatabaseConnection,
    report_id: u32,
    category: &str,
) -> Result<(Vec<serde_json::Map<String, serde_json::Value>>, HashSet<String>)> {
    let os_rows: Vec<(DateTimeUtc, String, i64)> = ProjectReportStats::find()
        .select_only()
        .column(project_report_stats::Column::Date)
        .column(project_report_stats::Column::Name)
        .column_as(
            project_report_stats::Column::Count.sum().cast_as(Alias::new("INTEGER")),
            "value",
        )
        .filter(project_report_stats::Column::ProjectReportId.eq(report_id))
        .filter(project_report_stats::Column::Category.eq(category))
        .group_by(project_report_stats::Column::Name)
        .group_by(project_report_stats::Column::Date)
        .order_by_desc(project_report_stats::Column::Date)
        .limit(30)
        .into_tuple()
        .all(db)
        .await?;

    let today = Utc::now().date_naive();
    let mut dataset = vec![];
    let mut names = HashSet::new();

    for date in today.iter_days().rev().take(30) {
        let mut daily_os_stats = serde_json::Map::new();

        for row in &os_rows {
            names.insert(row.1.clone());

            if row.0.date_naive() == date {
                let count = daily_os_stats.entry(row.1.clone()).or_insert(0.into());
                *count = (count.as_u64().unwrap() + row.2.to_u64().unwrap()).into();
            }
        }

        daily_os_stats.insert("date".into(), date.format("%b %d").to_string().into());
        dataset.push(daily_os_stats);
    }

    dataset.reverse();

    Ok((dataset, names))
}
