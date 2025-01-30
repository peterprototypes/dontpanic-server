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
    ProjectReportEventId,
    ProjectReportId,
    Log,
    Backtrace,
    Created,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectReportEvents::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(
                        ColumnDef::new(ProjectReportEvents::ProjectReportEventId)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProjectReportEvents::ProjectReportId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectReportEvents::Backtrace).text().null())
                    .col(ColumnDef::new(ProjectReportEvents::Log).text().null())
                    .col(
                        ColumnDef::new(ProjectReportEvents::Created)
                            .date_time()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_report_events_1")
                            .from_col(ProjectReportEvents::ProjectReportId)
                            .to(ProjectReports::Table, ProjectReports::ProjectReportId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
