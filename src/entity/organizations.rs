//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub organization_id: u32,
    pub name: String,
    pub requests_limit: Option<u32>,
    pub requests_count: Option<u32>,
    pub requests_count_start: Option<DateTime>,
    pub is_enabled: i8,
    pub created: DateTime,
    pub requests_alert_threshold: Option<u32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::organization_invitations::Entity")]
    OrganizationInvitations,
    #[sea_orm(has_many = "super::organization_stats::Entity")]
    OrganizationStats,
    #[sea_orm(has_many = "super::organization_users::Entity")]
    OrganizationUsers,
    #[sea_orm(has_many = "super::projects::Entity")]
    Projects,
}

impl Related<super::organization_invitations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationInvitations.def()
    }
}

impl Related<super::organization_stats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationStats.def()
    }
}

impl Related<super::organization_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationUsers.def()
    }
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::organization_users::Relation::Users.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::organization_users::Relation::Organizations.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
