//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "project_report_stats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub project_report_stat_id: u32,
    pub project_report_id: u32,
    pub category: String,
    pub name: String,
    pub count: u32,
    pub date: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project_reports::Entity",
        from = "Column::ProjectReportId",
        to = "super::project_reports::Column::ProjectReportId",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ProjectReports,
}

impl Related<super::project_reports::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectReports.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
