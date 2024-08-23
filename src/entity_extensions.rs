use anyhow::anyhow;
use anyhow::Result;
use sea_orm::prelude::*;

use crate::entity::organization_users;
use crate::entity::prelude::*;
use crate::entity::projects;
use crate::entity::users;

impl users::Model {
    pub async fn role(&self, db: &DatabaseConnection, org_id: u32) -> Result<Option<String>> {
        let user_org = OrganizationUsers::find_by_id((self.user_id, org_id)).one(db).await?;

        Ok(user_org.map(|u| u.role))
    }
}

impl Organizations {
    pub async fn delete(db: &DatabaseConnection, org_id: u32) -> Result<()> {
        let org = Organizations::find_by_id(org_id).one(db).await?.ok_or(anyhow!("Organization not found"))?;

        OrganizationUsers::delete_many()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .exec(db)
            .await?;

        Projects::delete_many().filter(projects::Column::OrganizationId.eq(org_id)).exec(db).await?;

        org.delete(db).await?;

        Ok(())
    }
}
