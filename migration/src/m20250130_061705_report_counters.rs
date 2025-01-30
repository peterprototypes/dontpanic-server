use sea_orm_migration::prelude::*;

use crate::TableDefaults;

#[derive(DeriveIden)]
enum ProjectReports {
    Table,
    ProjectReportId,
}

#[derive(DeriveIden)]
enum ProjectReportEvents {
    Table,
}

#[derive(DeriveIden)]
enum ProjectReportStats {
    Table,
    ProjectReportStatId,
    ProjectReportId,
    Date,
    Category,
    Name,
    Count,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(ProjectReportEvents::Table).to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectReportStats::Table)
                    .apply_defaults(manager)
                    .col(
                        ColumnDef::new(ProjectReportStats::ProjectReportStatId)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProjectReportStats::ProjectReportId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectReportStats::Category).string_len(255).not_null())
                    .col(ColumnDef::new(ProjectReportStats::Name).string_len(255).not_null())
                    .col(ColumnDef::new(ProjectReportStats::Count).unsigned().not_null())
                    .col(ColumnDef::new(ProjectReportStats::Date).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_report_stats_1")
                            .from_col(ProjectReportStats::ProjectReportId)
                            .to(ProjectReports::Table, ProjectReports::ProjectReportId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_project_report_stats_1")
                    .table(ProjectReportStats::Table)
                    .col(ProjectReportStats::ProjectReportId)
                    .col(ProjectReportStats::Category)
                    .col(ProjectReportStats::Name)
                    .col(ProjectReportStats::Date)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
