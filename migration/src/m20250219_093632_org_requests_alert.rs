use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum Organizations {
    Table,
    RequestsAlertThreshold,
}

#[derive(DeriveIden)]
enum OrganizationStats {
    Table,
    IsOverAlertThreshold,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .add_column(unsigned_null(Organizations::RequestsAlertThreshold))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationStats::Table)
                    .add_column(boolean(OrganizationStats::IsOverAlertThreshold).default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .drop_column(Organizations::RequestsAlertThreshold)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationStats::Table)
                    .drop_column(OrganizationStats::IsOverAlertThreshold)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
