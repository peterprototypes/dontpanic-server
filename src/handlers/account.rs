use actix_session::Session;
use actix_web::{delete, post, route, web};
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::entity::{organization_users, prelude::*};

use crate::{AppContext, Identity, Result, ViewModel};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(account).service(delete).service(password_update);
}

#[derive(Serialize, Deserialize, Clone, Validate)]
struct AccountEditForm {
    name: Option<String>,
}

#[route("/account", method = "GET", method = "POST")]
async fn account(ctx: web::Data<AppContext<'_>>, identity: Identity, form: Option<web::Form<AccountEditForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("account/view");

    let mut user = identity.user(&ctx).await?;

    let form = form.map(|f| f.into_inner());
    view.set("form", form.clone().unwrap_or_else(|| AccountEditForm { name: user.name.clone() }));

    if let Some(fields) = form {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let mut user_model = user.into_active_model();
        user_model.name = ActiveValue::set(fields.name);
        user = user_model.save(&ctx.db).await?.try_into_model()?;

        view.message("Account updated");
        view.set("saved", true);
    }

    view.set("user", user);

    Ok(view)
}

#[delete("/account")]
async fn delete(ctx: web::Data<AppContext<'_>>, identity: Identity, session: Session) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    let user = identity.user(&ctx).await?;

    // delete organizations only if this user is the only one in them

    let user_organizations = user.find_related(OrganizationUsers).all(&ctx.db).await?;

    for user_org in user_organizations {
        let org_id = user_org.organization_id;

        let member_count = OrganizationUsers::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .count(&ctx.db)
            .await?;

        user_org.delete(&ctx.db).await?;

        if member_count > 1 {
            // maybe make someone else owner?
            continue;
        }

        Organizations::delete(&ctx.db, org_id).await?;
    }

    user.delete(&ctx.db).await?;

    session.purge();
    view.redirect("/", true);

    Ok(view)
}

#[derive(Clone, Serialize, Deserialize, Validate)]
struct PasswordUpdateForm {
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    old_password: String,
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    new_password: String,
    #[validate(
        must_match(other = "new_password", message = "Password do not match"),
        length(min = 8, message = "Must be at least 8 characters long")
    )]
    new_password_repeat: String,
}

#[post("/account/password-update")]
pub async fn password_update(ctx: web::Data<AppContext<'_>>, identity: Identity, session: Session, form: Option<web::Form<PasswordUpdateForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template("account/view");

    let mut user = identity.user(&ctx).await?;

    let form = form.map(|f| f.into_inner());
    view.set("password_form", form.clone());

    if let Some(fields) = form {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let password_hash = std::str::from_utf8(&user.password)?;

        if !bcrypt::verify(&fields.old_password, password_hash)? {
            let mut errors = ValidationErrors::new();
            errors.add("old_password", ValidationError::new("incorrect").with_message("Password is incorrect".into()));
            view.set("errors", errors);
            return Ok(view);
        }

        let hashed_password = bcrypt::hash(&fields.new_password, bcrypt::DEFAULT_COST)?;

        let mut user_model = user.into_active_model();
        user_model.password = ActiveValue::set(hashed_password.into_bytes());
        user = user_model.save(&ctx.db).await?.try_into_model()?;

        session.remove("uid");
        view.redirect("/login", true);
    }

    view.set("user", user);

    Ok(view)
}
