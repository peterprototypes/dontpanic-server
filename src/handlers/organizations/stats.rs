use std::collections::HashSet;

use actix_web::{
    get,
    web::{self, Json, Path, Query},
    Responder,
};
use chrono::prelude::*;
use rust_decimal::prelude::*;
use sea_orm::prelude::*;
use sea_orm::sea_query::Alias;
use sea_orm::{DatabaseBackend, JoinType, QueryOrder, QuerySelect};
use serde::Deserialize;
use serde_json::json;

use crate::entity::prelude::*;
use crate::entity::{organization_stats, organization_users, organizations};

use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}

#[derive(Debug, Deserialize)]
struct UsageQuery {
    grouping: String,
    category: Option<String>,
}

#[get("")]
async fn get_stats(
    ctx: web::Data<AppContext<'_>>,
    path: Path<u32>,
    id: Identity,
    q: Query<UsageQuery>,
) -> Result<impl Responder> {
    let organization_id = path.into_inner();

    let user = id.user(&ctx).await?;

    let _ = Organizations::find_by_id(organization_id)
        .filter(organization_users::Column::UserId.eq(user.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let category = q.category.as_deref().unwrap_or("event");

    let (dataset, names) = match q.grouping.as_str() {
        "daily" => get_daily(&ctx.db, organization_id, category).await?,
        "monthly" => get_monthly(&ctx.db, organization_id, category).await?,
        _ => return Err(Error::new("Invalid grouping")),
    };

    Ok(Json(json!({
        "dataset": dataset,
        "names": names,
    })))
}

async fn get_daily(
    db: &DatabaseConnection,
    organization_id: u32,
    category: &str,
) -> Result<(Vec<serde_json::Map<String, serde_json::Value>>, HashSet<String>)> {
    let rows: Vec<(DateTimeUtc, String, i64)> = OrganizationStats::find()
        .select_only()
        .column(organization_stats::Column::Date)
        .column(organization_stats::Column::Name)
        .column_as(
            organization_stats::Column::Count.sum().cast_as(Alias::new("INTEGER")),
            "value",
        )
        .filter(organization_stats::Column::OrganizationId.eq(organization_id))
        .filter(organization_stats::Column::Category.eq(category))
        .group_by(organization_stats::Column::Name)
        .group_by(organization_stats::Column::Date)
        .order_by_desc(organization_stats::Column::Date)
        .limit(30)
        .into_tuple()
        .all(db)
        .await?;

    let today = Utc::now().date_naive();
    let mut dataset = vec![];
    let mut names = HashSet::new();

    for date in today.iter_days().rev().take(30) {
        let mut daily_os_stats = serde_json::Map::new();

        for row in &rows {
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

async fn get_monthly(
    db: &DatabaseConnection,
    organization_id: u32,
    category: &str,
) -> Result<(Vec<serde_json::Map<String, serde_json::Value>>, HashSet<String>)> {
    let date_group = match db.get_database_backend() {
        DatabaseBackend::Sqlite => Expr::cust("strftime('%Y-%m-01', organization_stats.date)"),
        DatabaseBackend::MySql => Expr::cust("DATE_FORMAT(organization_stats.date, '%Y-%m-01')"),
        _ => Expr::cust("organization_stats.date"),
    };

    let rows: Vec<(String, String, i64)> = OrganizationStats::find()
        .select_only()
        .column_as(date_group.clone(), "date")
        .column(organization_stats::Column::Name)
        .column_as(
            organization_stats::Column::Count.sum().cast_as(Alias::new("INTEGER")),
            "value",
        )
        .filter(organization_stats::Column::OrganizationId.eq(organization_id))
        .filter(organization_stats::Column::Category.eq(category))
        .group_by(organization_stats::Column::Name)
        .group_by(date_group)
        .order_by_desc(organization_stats::Column::Date)
        .limit(12)
        .into_tuple()
        .all(db)
        .await?;

    let today = Utc::now().date_naive();
    let mut dataset = vec![];
    let mut names = HashSet::new();

    for i in 0..12 {
        let date = today - chrono::Duration::days(i * 30);
        let mut monthly_os_stats = serde_json::Map::new();

        for row in &rows {
            names.insert(row.1.clone());

            if row.0 == date.format("%Y-%m-01").to_string() {
                let count = monthly_os_stats.entry(row.1.clone()).or_insert(0.into());
                *count = (count.as_u64().unwrap() + row.2.to_u64().unwrap()).into();
            }
        }

        monthly_os_stats.insert("date".into(), date.format("%b %Y").to_string().into());
        dataset.push(monthly_os_stats);
    }

    dataset.reverse();

    Ok((dataset, names))
}
