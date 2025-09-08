use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum ProjectEnvironments {
    Table,
    SlackChannel,
    SlackWebhook,
    Webhook,
    TeamsWebhook,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // sqlite doesn't support multiple column changes in a single statement

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .add_column(string_len_null(ProjectEnvironments::SlackChannel, 255))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .add_column(string_len_null(ProjectEnvironments::SlackWebhook, 255))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .add_column(string_len_null(ProjectEnvironments::Webhook, 255))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .add_column(string_len_null(ProjectEnvironments::TeamsWebhook, 255))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .drop_column(ProjectEnvironments::SlackChannel)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .drop_column(ProjectEnvironments::SlackWebhook)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .drop_column(ProjectEnvironments::Webhook)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectEnvironments::Table)
                    .drop_column(ProjectEnvironments::TeamsWebhook)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
