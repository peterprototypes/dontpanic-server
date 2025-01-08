use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum Users {
    Table,
    PushoverUserKey,
}

#[derive(DeriveIden)]
enum ProjectUserSettings {
    Table,
    NotifyPushover,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string_len_null(Users::PushoverUserKey, 64))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectUserSettings::Table)
                    .add_column(boolean(ProjectUserSettings::NotifyPushover))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::PushoverUserKey)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectUserSettings::Table)
                    .drop_column(ProjectUserSettings::NotifyPushover)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
