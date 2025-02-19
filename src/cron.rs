use actix_web::middleware::Condition;
use anyhow::Result;
use chrono::prelude::*;

use lettre::AsyncTransport;
use sea_orm::{prelude::*, JoinType, QuerySelect};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, TryIntoModel};

use tokio::join;
use tokio_schedule::{every, Job};

use crate::notifications::{Notification, ReportStatus};
use crate::AppContext;

use crate::entity::prelude::*;
use crate::entity::{
    organization_stats, organization_users, organizations, project_report_events, project_report_stats, users,
};

pub async fn run_command(ctx: AppContext<'_>, cmd: &str) -> Result<()> {
    match cmd {
        "notify-spiking" => notify_spiking_reports(ctx).await,
        "notify-limits" => notify_organization_limits(ctx).await,
        _ => Err(anyhow::anyhow!("Unknown command")),
    }
}

pub async fn cronjobs(ctx: AppContext<'_>) {
    let spiking_reports = every(10).minutes().perform(|| async {
        if let Err(e) = notify_spiking_reports(ctx.clone()).await {
            log::error!("Error notifying for spiking reports: {}", e);
        }
    });

    let organization_limits = every(10).minutes().perform(|| async {
        if let Err(e) = notify_organization_limits(ctx.clone()).await {
            log::error!("Error notifying for organization limits: {}", e);
        }
    });

    join!(spiking_reports, organization_limits);
}

pub async fn notify_spiking_reports(ctx: AppContext<'_>) -> Result<()> {
    let last_hour = Utc::now() - chrono::Duration::hours(1);
    let last_hour_start = last_hour.clone().with_minute(0).unwrap().with_second(0).unwrap();
    let last_hour_start = last_hour_start.format("%Y-%m-%d %H:%M:%S").to_string();
    let current_hour_start = Utc::now().with_minute(0).unwrap().with_second(0).unwrap();
    let current_hour_start = current_hour_start.format("%Y-%m-%d %H:%M:%S").to_string();

    let reports_stats = ProjectReportStats::find()
        .filter(project_report_stats::Column::Category.eq("event"))
        .filter(project_report_stats::Column::Name.eq("total_count"))
        .filter(project_report_stats::Column::Date.gte(&last_hour_start))
        .filter(project_report_stats::Column::Spiking.eq(false))
        .order_by_desc(project_report_stats::Column::Date)
        .all(&ctx.db)
        .await?;

    for report_stat in &reports_stats {
        let report_hour = report_stat.date.format("%Y-%m-%d %H:%M:%S").to_string();

        if report_hour != current_hour_start {
            continue;
        }

        let last_hour_report = reports_stats
            .iter()
            .find(|r| r.date.format("%Y-%m-%d %H:%M:%S").to_string() == last_hour_start);

        let Some(last_hour_report) = last_hour_report else {
            continue;
        };

        let diff = report_stat.count.saturating_sub(last_hour_report.count);
        let diff_percent = (diff as f64 / last_hour_report.count as f64) * 100.0;

        if diff_percent > 50.0 {
            let mut report_stat = report_stat.clone().into_active_model();
            report_stat.spiking = ActiveValue::set(true as i8);
            let report_stat = report_stat.save(&ctx.db).await?.try_into_model()?;

            let Some(report) = report_stat.find_related(ProjectReports).one(&ctx.db).await? else {
                continue;
            };

            let Some(project) = report.find_related(Projects).one(&ctx.db).await? else {
                continue;
            };

            let event = report
                .find_related(ProjectReportEvents)
                .order_by_desc(project_report_events::Column::ProjectReportEventId)
                .one(&ctx.db)
                .await?;

            let Some(event) = event else {
                continue;
            };

            let environment = report.find_related(ProjectEnvironments).one(&ctx.db).await?;

            let res = ctx.notifications.send(Notification {
                status: Some(ReportStatus::Spiking {
                    percentage: diff_percent.round() as u32,
                }),
                project,
                event,
                report,
                environment,
            });

            if let Err(e) = res {
                log::error!("Error sending notification: {:?}", e);
            }
        }
    }

    Ok(())
}

pub async fn notify_organization_limits(ctx: AppContext<'_>) -> Result<()> {
    let Some(mailer) = ctx.mailer.as_ref() else {
        log::warn!("Mailer is not configured");
        return Ok(());
    };

    let current_day_start = Utc::now().format("%Y-%m-%d").to_string();

    let organizations = Organizations::find()
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationStats.def())
        .filter(organization_stats::Column::Category.eq("event"))
        .filter(organization_stats::Column::Name.eq("total_count"))
        .filter(organization_stats::Column::Date.eq(&current_day_start))
        .filter(organization_stats::Column::IsOverAlertThreshold.eq(false))
        .filter(organizations::Column::RequestsAlertThreshold.is_not_null())
        .filter(Expr::cust(
            "organization_stats.count >= organizations.requests_alert_threshold",
        ))
        .all(&ctx.db)
        .await?;

    for org in organizations {
        let mut stats_row = org
            .find_related(OrganizationStats)
            .filter(organization_stats::Column::Category.eq("event"))
            .filter(organization_stats::Column::Name.eq("total_count"))
            .filter(organization_stats::Column::Date.eq(&current_day_start))
            .one(&ctx.db)
            .await?
            .ok_or(anyhow::anyhow!("Organization stats row not found"))?
            .into_active_model();

        stats_row.is_over_alert_threshold = ActiveValue::set(true as i8);
        let stats_row = stats_row.save(&ctx.db).await?.try_into_model()?;

        let owners = Users::find()
            .filter(organization_users::Column::Role.eq("owner"))
            .filter(organization_users::Column::OrganizationId.eq(org.organization_id))
            .join(JoinType::InnerJoin, users::Relation::OrganizationUsers.def())
            .all(&ctx.db)
            .await?;

        let title = format!(
            "Don't Panic: Your organization \"{}\" has exceeded the daily requests alert threshold",
            org.name
        );

        for owner in owners {
            let email = lettre::Message::builder()
                .from(ctx.config.email_from.clone().into())
                .to(owner.email.parse()?)
                .subject(&title)
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(ctx.hb.render(
                    "email/org_requests_alert",
                    &serde_json::json!({
                        "base_url": ctx.config.base_url,
                        "scheme": ctx.config.scheme,
                        "title": title,
                        "organization": org,
                        "actual": stats_row.count,
                    }),
                )?)?;

            mailer.send(email).await?;
        }
    }

    Ok(())
}
