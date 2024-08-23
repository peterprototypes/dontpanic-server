use actix_web::{get, web};
use sea_orm::prelude::*;
use sea_orm::{JoinType, QuerySelect};
use serde::Serialize;

use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::projects;

use crate::AppContext;
use crate::Identity;
use crate::Result;
use crate::ViewModel;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(sidemenu);
}

#[derive(Serialize, Debug)]
struct OrganizationAndProjects {
    organization: organizations::Model,
    projects: Vec<projects::Model>,
}

#[get("/sidemenu")]
async fn sidemenu(ctx: web::Data<AppContext<'_>>, identity: Identity) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("sidemenu");

    let organizations: Vec<OrganizationAndProjects> = Organizations::find()
        .filter(organization_users::Column::UserId.eq(identity.user_id))
        .join(JoinType::InnerJoin, organizations::Relation::OrganizationUsers.def())
        .find_with_related(Projects)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|(organization, projects)| OrganizationAndProjects { organization, projects })
        .collect();

    view.set("organizations", organizations);

    Ok(view)
}
