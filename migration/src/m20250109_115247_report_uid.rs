use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ProjectReports {
    Table,
    Uid,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectReports::Table)
                    .add_column(string_len_uniq(ProjectReports::Uid, 64))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectReports::Table)
                    .drop_column(ProjectReports::Uid)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
