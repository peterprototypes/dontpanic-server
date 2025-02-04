use sea_orm_migration::prelude::*;

use crate::TableDefaults;

#[derive(DeriveIden)]
enum Organizations {
    Table,
    OrganizationId,
}

#[derive(DeriveIden)]
enum OrganizationStats {
    Table,
    OrganizationStatId,
    OrganizationId,
    Category,
    Name,
    Count,
    Date,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrganizationStats::Table)
                    .apply_defaults(manager)
                    .col(
                        ColumnDef::new(OrganizationStats::OrganizationStatId)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(OrganizationStats::OrganizationId).unsigned().not_null())
                    .col(ColumnDef::new(OrganizationStats::Category).string_len(255).not_null())
                    .col(ColumnDef::new(OrganizationStats::Name).string_len(255).not_null())
                    .col(ColumnDef::new(OrganizationStats::Count).unsigned().not_null())
                    .col(ColumnDef::new(OrganizationStats::Date).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_stats_1")
                            .from_col(OrganizationStats::OrganizationId)
                            .to(Organizations::Table, Organizations::OrganizationId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_stats_1")
                    .table(OrganizationStats::Table)
                    .col(OrganizationStats::OrganizationId)
                    .col(OrganizationStats::Category)
                    .col(OrganizationStats::Name)
                    .col(OrganizationStats::Date)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(OrganizationStats::Table).to_owned())
            .await?;

        Ok(())
    }
}
