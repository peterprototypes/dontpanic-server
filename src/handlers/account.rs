use actix_session::Session;
use actix_web::{delete, get, post, route, web, Responder};
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::entity::users;
use crate::entity::{organization_users, prelude::*};

use crate::{AppContext, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get)
        .service(update)
        .service(delete)
        .service(update_password);
}

#[derive(Clone, Debug, Serialize, Validate)]
struct AccountResponse {
    user_id: u32,
    email: String,
    name: Option<String>,
    iana_timezone_name: String,
    created: DateTime,
}

impl From<users::Model> for AccountResponse {
    fn from(value: users::Model) -> Self {
        Self {
            user_id: value.user_id,
            email: value.email,
            name: value.name,
            iana_timezone_name: value.iana_timezone_name,
            created: value.created,
        }
    }
}

#[get("")]
async fn get(ctx: web::Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;
    Ok(web::Json(AccountResponse::from(user)))
}

#[derive(Serialize, Deserialize, Clone, Validate)]
struct InfoInput {
    name: Option<String>,
}

#[post("")]
async fn update(ctx: web::Data<AppContext<'_>>, id: Identity, input: web::Json<InfoInput>) -> Result<impl Responder> {
    input.validate()?;

    let mut user = id.user(&ctx).await?.into_active_model();
    user.name = ActiveValue::set(input.into_inner().name);
    let user = user.save(&ctx.db).await?.try_into_model()?;

    Ok(web::Json(AccountResponse::from(user)))
}

#[post("/delete")]
async fn delete(ctx: web::Data<AppContext<'_>>, id: Identity, session: Session) -> Result<impl Responder> {
    let user = id.user(&ctx).await?;

    // delete organizations only if this user is the only one in them
    let user_organizations = user.find_related(OrganizationUsers).all(&ctx.db).await?;

    for user_org in user_organizations {
        let org_id = user_org.organization_id;

        let member_count = OrganizationUsers::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .count(&ctx.db)
            .await?;

        user_org.delete(&ctx.db).await?;

        if member_count == 1 {
            Organizations::delete(&ctx.db, org_id).await?;
        }
    }

    user.delete(&ctx.db).await?;
    session.purge();

    Ok(web::Json(()))
}

#[post("/update-password")]
async fn update_password(ctx: web::Data<AppContext<'_>>, id: Identity) -> Result<impl Responder> {
    Ok(web::Json(()))
}

// #[delete("/account")]
// async fn delete(ctx: web::Data<AppContext<'_>>, identity: Identity, session: Session) -> Result<ViewModel> {
//     let mut view = ViewModel::default();

//     let user = identity.user(&ctx).await?;

//     // delete organizations only if this user is the only one in them

//     let user_organizations = user.find_related(OrganizationUsers).all(&ctx.db).await?;

//     for user_org in user_organizations {
//         let org_id = user_org.organization_id;

//         let member_count = OrganizationUsers::find()
//             .filter(organization_users::Column::OrganizationId.eq(org_id))
//             .count(&ctx.db)
//             .await?;

//         user_org.delete(&ctx.db).await?;

//         if member_count > 1 {
//             // maybe make someone else owner?
//             continue;
//         }

//         Organizations::delete(&ctx.db, org_id).await?;
//     }

//     user.delete(&ctx.db).await?;

//     session.purge();
//     view.redirect("/", true);

//     Ok(view)
// }

// #[derive(Clone, Serialize, Deserialize, Validate)]
// struct PasswordUpdateForm {
//     #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
//     old_password: String,
//     #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
//     new_password: String,
//     #[validate(
//         must_match(other = "new_password", message = "Password do not match"),
//         length(min = 8, message = "Must be at least 8 characters long")
//     )]
//     new_password_repeat: String,
// }

// #[post("/account/password-update")]
// pub async fn password_update(
//     ctx: web::Data<AppContext<'_>>,
//     identity: Identity,
//     session: Session,
//     form: Option<web::Form<PasswordUpdateForm>>,
// ) -> Result<ViewModel> {
//     let mut view = ViewModel::with_template("account/view");

//     let mut user = identity.user(&ctx).await?;

//     let form = form.map(|f| f.into_inner());
//     view.set("password_form", form.clone());

//     if let Some(fields) = form {
//         if let Err(errors) = fields.validate() {
//             view.set("errors", &errors);
//             return Ok(view);
//         }

//         let password_hash = std::str::from_utf8(&user.password)?;

//         if !bcrypt::verify(&fields.old_password, password_hash)? {
//             let mut errors = ValidationErrors::new();
//             errors.add(
//                 "old_password",
//                 ValidationError::new("incorrect").with_message("Password is incorrect".into()),
//             );
//             view.set("errors", errors);
//             return Ok(view);
//         }

//         let hashed_password = bcrypt::hash(&fields.new_password, bcrypt::DEFAULT_COST)?;

//         let mut user_model = user.into_active_model();
//         user_model.password = ActiveValue::set(hashed_password.into_bytes());
//         user = user_model.save(&ctx.db).await?.try_into_model()?;

//         session.remove("uid");
//         view.redirect("/login", true);
//     }

//     view.set("user", user);

//     Ok(view)
// }
