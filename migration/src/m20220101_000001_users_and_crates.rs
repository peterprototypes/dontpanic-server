use sea_orm_migration::prelude::*;

use crate::TableDefaults;

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
    Email,
    Password,
    Name,
    PasswordResetHash,
    PasswordResetHashCreated,
    EmailVerificationHash,
    EmailVerificationHashCreated,
    TotpSecret,
    IanaTimezoneName,
    Created,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    OrganizationId,
    Name,
    RequestsLimit,
    RequestsCount,
    RequestsCountStart,
    IsEnabled,
    Created,
}

#[derive(DeriveIden)]
enum OrganizationUsers {
    Table,
    UserId,
    OrganizationId,
    Role,
    Created,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    ProjectId,
    OrganizationId,
    Name,
    ApiKey,
    SlackBotToken,
    SlackChannel,
    Created,
}

#[derive(DeriveIden)]
enum ProjectUserSettings {
    Table,
    ProjectId,
    UserId,
    NotifyEmail,
}

#[derive(DeriveIden)]
enum ProjectEnvironments {
    Table,
    ProjectEnvironmentId,
    ProjectId,
    Name,
}

#[derive(DeriveIden)]
enum ProjectReports {
    Table,
    ProjectReportId,
    ProjectId,
    ProjectEnvironmentId,
    Title,
    LastSeen,
    IsResolved,
    IsSeen,
    Created,
}

#[derive(DeriveIden)]
enum ProjectReportEvents {
    Table,
    ProjectReportEventId,
    ProjectReportId,
    PrevEventId,
    NextEventId,
    EventData,
    Created,
}

#[derive(DeriveIden)]
enum OrganizationInvitations {
    Table,
    OrganizationInvitationId,
    OrganizationId,
    Email,
    Role,
    Created,
}

#[derive(DeriveIden)]
enum SeaqlMigrations {
    Table,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(ColumnDef::new(Users::UserId).unsigned().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Users::Email).string_len(320).not_null())
                    .col(ColumnDef::new(Users::Password).binary_len(60).not_null())
                    .col(ColumnDef::new(Users::Name).string_len(100).null())
                    .col(ColumnDef::new(Users::PasswordResetHash).char_len(64).null())
                    .col(ColumnDef::new(Users::PasswordResetHashCreated).date_time().null())
                    .col(ColumnDef::new(Users::EmailVerificationHash).char_len(64).null())
                    .col(ColumnDef::new(Users::EmailVerificationHashCreated).date_time().null())
                    .col(ColumnDef::new(Users::TotpSecret).char_len(32).null())
                    .col(ColumnDef::new(Users::IanaTimezoneName).string_len(32).not_null().default("UTC"))
                    .col(ColumnDef::new(Users::Created).date_time().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Organizations::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(ColumnDef::new(Organizations::OrganizationId).unsigned().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Organizations::Name).string_len(80).not_null())
                    .col(ColumnDef::new(Organizations::RequestsLimit).unsigned().null())
                    .col(ColumnDef::new(Organizations::RequestsCount).unsigned().null())
                    .col(ColumnDef::new(Organizations::RequestsCountStart).date_time().null())
                    .col(ColumnDef::new(Organizations::IsEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(Organizations::Created).date_time().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrganizationUsers::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(ColumnDef::new(OrganizationUsers::UserId).unsigned().not_null())
                    .col(ColumnDef::new(OrganizationUsers::OrganizationId).unsigned().not_null())
                    .col(ColumnDef::new(OrganizationUsers::Role).string_len(20).not_null())
                    .col(ColumnDef::new(OrganizationUsers::Created).date_time().not_null().default(Expr::current_timestamp()))
                    .primary_key(Index::create().name("PRIMARY").col(OrganizationUsers::UserId).col(OrganizationUsers::OrganizationId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_members_1")
                            .from_col(OrganizationUsers::UserId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_members_2")
                            .from_col(OrganizationUsers::OrganizationId)
                            .to(Organizations::Table, Organizations::OrganizationId)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(ColumnDef::new(Projects::ProjectId).unsigned().not_null().primary_key().auto_increment())
                    .col(ColumnDef::new(Projects::OrganizationId).unsigned().not_null())
                    .col(ColumnDef::new(Projects::Name).string_len(80).not_null())
                    .col(ColumnDef::new(Projects::ApiKey).char_len(32).not_null())
                    .col(ColumnDef::new(Projects::SlackBotToken).string_len(255).null())
                    .col(ColumnDef::new(Projects::SlackChannel).string_len(255).null())
                    .col(ColumnDef::new(Projects::Created).date_time().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_projects_1")
                            .from_col(Projects::OrganizationId)
                            .to(Organizations::Table, Organizations::OrganizationId)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(Index::create().name("ApiKey").table(Projects::Table).col(Projects::ApiKey).unique().to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectUserSettings::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(ColumnDef::new(ProjectUserSettings::ProjectId).unsigned().not_null())
                    .col(ColumnDef::new(ProjectUserSettings::UserId).unsigned().not_null())
                    .col(ColumnDef::new(ProjectUserSettings::NotifyEmail).boolean().not_null())
                    .primary_key(Index::create().name("PRIMARY").col(ProjectUserSettings::ProjectId).col(ProjectUserSettings::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_member_settings_1")
                            .from_col(ProjectUserSettings::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_member_settings_2")
                            .from_col(ProjectUserSettings::UserId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectEnvironments::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(
                        ColumnDef::new(ProjectEnvironments::ProjectEnvironmentId)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(ProjectEnvironments::ProjectId).unsigned().not_null())
                    .col(ColumnDef::new(ProjectEnvironments::Name).string_len(80).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_environments_1")
                            .from_col(ProjectEnvironments::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectReports::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(ColumnDef::new(ProjectReports::ProjectReportId).unsigned().not_null().primary_key().auto_increment())
                    .col(ColumnDef::new(ProjectReports::ProjectId).unsigned().not_null())
                    .col(ColumnDef::new(ProjectReports::ProjectEnvironmentId).unsigned().null())
                    .col(ColumnDef::new(ProjectReports::Title).string_len(500).not_null())
                    .col(ColumnDef::new(ProjectReports::LastSeen).not_null().date_time().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(ProjectReports::IsSeen).not_null().boolean().default(false))
                    .col(ColumnDef::new(ProjectReports::IsResolved).not_null().boolean().default(false))
                    .col(ColumnDef::new(ProjectReports::Created).date_time().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_reports_1")
                            .from_col(ProjectReports::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_reports_2")
                            .from_col(ProjectReports::ProjectEnvironmentId)
                            .to(ProjectEnvironments::Table, ProjectEnvironments::ProjectEnvironmentId)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

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
                    .col(ColumnDef::new(ProjectReportEvents::ProjectReportId).unsigned().not_null())
                    .col(ColumnDef::new(ProjectReportEvents::PrevEventId).unsigned().null())
                    .col(ColumnDef::new(ProjectReportEvents::NextEventId).unsigned().null())
                    .col(ColumnDef::new(ProjectReportEvents::EventData).text().not_null())
                    .col(ColumnDef::new(ProjectReportEvents::Created).date_time().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_report_events_1")
                            .from_col(ProjectReportEvents::ProjectReportId)
                            .to(ProjectReports::Table, ProjectReports::ProjectReportId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_report_events_2")
                            .from_col(ProjectReportEvents::PrevEventId)
                            .to(ProjectReportEvents::Table, ProjectReportEvents::ProjectReportEventId)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_report_events_3")
                            .from_col(ProjectReportEvents::NextEventId)
                            .to(ProjectReportEvents::Table, ProjectReportEvents::ProjectReportEventId)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrganizationInvitations::Table)
                    .if_not_exists()
                    .apply_defaults(manager)
                    .col(
                        ColumnDef::new(OrganizationInvitations::OrganizationInvitationId)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(OrganizationInvitations::OrganizationId).unsigned().not_null())
                    .col(ColumnDef::new(OrganizationInvitations::Email).string_len(320).not_null())
                    .col(ColumnDef::new(OrganizationInvitations::Role).string_len(20).not_null())
                    .col(ColumnDef::new(OrganizationInvitations::Created).date_time().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_organization_invitations_1")
                            .from_col(OrganizationInvitations::OrganizationId)
                            .to(Organizations::Table, Organizations::OrganizationId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OrganizationInvitations::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ProjectReportEvents::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ProjectReports::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ProjectEnvironments::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Projects::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(OrganizationUsers::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Organizations::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeaqlMigrations::Table).to_owned()).await?;

        Ok(())
    }
}
