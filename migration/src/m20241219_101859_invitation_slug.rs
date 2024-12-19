use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OrganizationInvitations {
    Table,
    Slug,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationInvitations::Table)
                    .add_column(char_len(OrganizationInvitations::Slug, 64))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("organization_invitations_slug_index")
                    .table(OrganizationInvitations::Table)
                    .col(OrganizationInvitations::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationInvitations::Table)
                    .drop_column(OrganizationInvitations::Slug)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
