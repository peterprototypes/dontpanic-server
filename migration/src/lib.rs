use sea_orm::DbBackend;
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users_and_crates;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_users_and_crates::Migration)]
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
