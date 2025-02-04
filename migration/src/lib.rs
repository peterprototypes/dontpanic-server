use sea_orm::DbBackend;
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users_and_crates;
mod m20240830_153930_project_slack_webhhok;
mod m20240908_063351_webhook_integration;
mod m20241219_101859_invitation_slug;
mod m20250108_135710_pushover_integration;
mod m20250109_082535_teams_integration;
mod m20250109_115247_report_uid;
mod m20250130_061705_report_counters;
mod m20250130_065644_report_events;
mod m20250204_035650_organization_stats;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users_and_crates::Migration),
            Box::new(m20240830_153930_project_slack_webhhok::Migration),
            Box::new(m20240908_063351_webhook_integration::Migration),
            Box::new(m20241219_101859_invitation_slug::Migration),
            Box::new(m20250108_135710_pushover_integration::Migration),
            Box::new(m20250109_082535_teams_integration::Migration),
            Box::new(m20250109_115247_report_uid::Migration),
            Box::new(m20250130_061705_report_counters::Migration),
            Box::new(m20250130_065644_report_events::Migration),
            Box::new(m20250204_035650_organization_stats::Migration),
        ]
    }
}

pub trait TableDefaults {
    fn apply_defaults<'a>(&'a mut self, manager: &SchemaManager) -> &'a mut Self;
}

impl TableDefaults for TableCreateStatement {
    fn apply_defaults<'a>(&'a mut self, manager: &SchemaManager) -> &'a mut Self {
        if manager.get_connection().get_database_backend() == DbBackend::MySql {
            self.engine("InnoDB");
            self.character_set("utf8mb4");
            self.collate("utf8mb4_unicode_ci");
        }
        self
    }
}
