use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum ProjectReportStats {
    Table,
    Spiking,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectReportStats::Table)
                    .add_column(boolean(ProjectReportStats::Spiking).default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectReportStats::Table)
                    .drop_column(ProjectReportStats::Spiking)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
