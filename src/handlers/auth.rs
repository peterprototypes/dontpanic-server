use actix_session::Session;
use actix_web::{get, http, post, HttpRequest};
use actix_web::{route, web};
use anyhow::anyhow;
use chrono::{TimeDelta, Utc};
use chrono_tz::Tz;
use lettre::AsyncTransport;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel, TryIntoModel};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entity::organization_invitations;
use crate::entity::organization_users;
use crate::entity::organizations;
use crate::entity::prelude::*;
use crate::entity::users;
use crate::{AppContext, Error, ViewModel};
use crate::{Identity, Result};

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct RegistrationData {
    #[validate(email(message = "A valid email address is required"), length(max = 320, message = "Must be less than 320 chars"))]
    email: String,
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    password: String,
    name: Option<String>,
    company: Option<String>,
    iana_timezone_name: Option<String>,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(register_success)
        .service(verify_email)
        .service(resend_verification_email)
        .service(request_password_reset)
        .service(reset_password);
}

#[route("/register", method = "GET", method = "POST")]
async fn register(ctx: web::Data<AppContext<'static>>, form: Option<web::Form<RegistrationData>>, identity: Option<Identity>, session: Session) -> Result<ViewModel> {
    let mut view = ViewModel::with_template_and_layout("auth/register", "layout_auth");

    if !ctx.config.registration_enabled {
        view.redirect("/login", true);
        return Ok(view);
    }

    if let Some(identity) = identity {
        let user = Users::find_by_id(identity.user_id).one(&ctx.db).await?;

        if user.is_some() {
            view.redirect("/reports", true);
            return Ok(view);
        } else {
            session.remove("uid");
        }
    }

    view.set("form", &form);

    if let Some(fields) = form {
        if let Err(errors) = fields.validate() {
            view.set("errors", &errors);
            return Ok(view);
        }

        let user_search = Users::find().filter(users::Column::Email.eq(&fields.email)).one(&ctx.db).await?;

        if user_search.is_some() {
            view.set("error_message", "Account already exists. Please login instead.");
            return Ok(view);
        }

        let data = fields.clone();
        let registration_ctx = ctx.clone();

        if let Err(e) = create_user(registration_ctx, data).await {
            view.set("error_message", e.to_string());
            return Ok(view);
        }

        let mut view = ViewModel::with_template_and_layout("auth/register_success", "layout_auth");

        view.set("require_email_verification", ctx.config.require_email_verification);
        view.set("email", &fields.email);

        return Ok(view);
    }

    Ok(view)
}

async fn create_user(ctx: web::Data<AppContext<'_>>, data: RegistrationData) -> anyhow::Result<()> {
    let timezone_name = data
        .iana_timezone_name
        .filter(|tz_name| tz_name.parse::<Tz>().is_ok())
        .unwrap_or_else(|| ctx.config.default_user_timezone.to_string());

    let hashed_password = bcrypt::hash(&data.password, bcrypt::DEFAULT_COST)?;

    let name = data.name.filter(|s| !s.is_empty());

    let (email_verification_hash, email_verification_created) = if ctx.config.require_email_verification {
        let hash: String = rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();

        (Some(hash), Some(Utc::now().naive_utc()))
    } else {
        (None, None)
    };

    let user = users::ActiveModel {
        email: ActiveValue::set(data.email.clone()),
        password: ActiveValue::set(hashed_password.into_bytes()),
        name: ActiveValue::set(name),
        email_verification_hash: ActiveValue::set(email_verification_hash.clone()),
        email_verification_hash_created: ActiveValue::set(email_verification_created),
        iana_timezone_name: ActiveValue::set(timezone_name),
        ..Default::default()
    };

    let user = user.insert(&ctx.db).await?.try_into_model()?;

    let company = data.company.filter(|s| !s.is_empty());

    let organization = organizations::ActiveModel {
        name: ActiveValue::set(company.unwrap_or(String::from("Default Organization"))),
        requests_limit: ActiveValue::Set(Some(10)),
        requests_count: ActiveValue::Set(Some(0)),
        requests_count_start: ActiveValue::set(Some(Utc::now().naive_utc())),
        is_enabled: ActiveValue::set(1),
        ..Default::default()
    };

    let organization = organization.insert(&ctx.db).await?.try_into_model()?;

    let organization_member = organization_users::ActiveModel {
        organization_id: ActiveValue::set(organization.organization_id),
        user_id: ActiveValue::set(user.user_id),
        role: ActiveValue::set("owner".to_string()),
        ..Default::default()
    };

    organization_member.insert(&ctx.db).await?;

    let invitations = OrganizationInvitations::find()
        .filter(organization_invitations::Column::Email.eq(&data.email))
        .all(&ctx.db)
        .await?;

    for invitation in invitations {
        let organization_member = organization_users::ActiveModel {
            organization_id: ActiveValue::set(invitation.organization_id),
            user_id: ActiveValue::set(user.user_id),
            role: ActiveValue::set(invitation.role.clone()),
            ..Default::default()
        };

        organization_member.insert(&ctx.db).await?;
        invitation.delete(&ctx.db).await?;

        // TODO: send invitation accepted email to the person who invited this user
    }

    if let Some(email_verification_hash) = email_verification_hash {
        // email verification is not disabled

        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(data.email.parse()?)
            .subject("Please confirm your e-mail address")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/confirmation",
                &serde_json::json!({
                    "hash": email_verification_hash,
                    "base_url": ctx.config.base_url,
                    "scheme": ctx.config.scheme,
                    "title": "Please confirm your e-mail address"
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            mailer.send(email).await?;
        }
    }

    Ok(())
}

#[get("/register-success")]
async fn register_success(ctx: web::Data<AppContext<'_>>) -> ViewModel {
    let mut view = ViewModel::with_template_and_layout("auth/register_success", "layout_auth");

    view.set("require_email_verification", ctx.config.require_email_verification);

    view
}

#[get("/auth/verify-email/{hash}")]
async fn verify_email(ctx: web::Data<AppContext<'_>>, path: web::Path<String>, session: Session) -> Result<ViewModel> {
    let view = ViewModel::with_template_and_layout("auth/email_verified", "layout_auth");

    let hash = path.into_inner();

    let user = Users::find()
        .filter(users::Column::EmailVerificationHash.eq(hash))
        .one(&ctx.db)
        .await?
        .ok_or(Error::new("Email verification request not found or expired."))?;

    let mut user = user.into_active_model();
    user.email_verification_hash = ActiveValue::set(None);
    user.email_verification_hash_created = ActiveValue::set(None);
    let user = user.save(&ctx.db).await?.try_into_model()?;

    //device used to register is trusted
    session.insert(format!("seen_{}", user.user_id), true)?;

    Ok(view)
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct ResendVerificationForm {
    #[validate(email(message = "A valid email address is required"), length(max = 320, message = "Must be less than 320 chars"))]
    email: String,
}

#[post("/auth/resend-verification-email")]
async fn resend_verification_email(ctx: web::Data<AppContext<'static>>, req: HttpRequest, form: web::Form<ResendVerificationForm>) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    // try for constant time response, regardless if user exists or not
    actix_web::rt::spawn(async move {
        if let Err(e) = resend_verification_email_in_bg(ctx, req, form).await {
            log::warn!("Resend verification email error: {}", e);
        }
    });

    view.message("Email verification sent.");

    Ok(view)
}

async fn resend_verification_email_in_bg(ctx: web::Data<AppContext<'_>>, req: HttpRequest, form: web::Form<ResendVerificationForm>) -> anyhow::Result<()> {
    let user = Users::find()
        .filter(users::Column::Email.eq(&form.email))
        .one(&ctx.db)
        .await?
        .ok_or(anyhow!("User not found"))?;

    if user.email_verification_hash.is_none() {
        anyhow::bail!("Email is already verified");
    }

    if !ctx.config.require_email_verification {
        anyhow::bail!("Email verification is not required");
    }

    let Some(hash_created) = user.email_verification_hash_created else {
        anyhow::bail!("User {} has no email_verification_hash_created date", user.user_id);
    };

    if Utc::now() - hash_created.and_utc() < TimeDelta::seconds(50) {
        anyhow::bail!("Attempt to resent email verification without respecting time limit");
    }

    let hash: String = rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();
    let created = Utc::now().naive_utc();

    let mut user = user.into_active_model();
    user.email_verification_hash = ActiveValue::set(Some(hash.clone()));
    user.email_verification_hash_created = ActiveValue::set(Some(created));
    user.save(&ctx.db).await?;

    let scheme = {
        let conn_info = req.connection_info();
        conn_info.scheme().to_string()
    };

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(form.email.parse()?)
        .subject("Please confirm your e-mail address")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(
            "email/confirmation",
            &serde_json::json!({
                "hash": hash,
                "base_url": ctx.config.base_url,
                "scheme": scheme,
                "title": "Please confirm your e-mail address"
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct LoginData {
    #[validate(email(message = "A valid email address is required"), length(max = 320, message = "Must be less than 320 chars"))]
    email: String,
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    password: String,
}

#[route("/login", method = "GET", method = "POST")]
async fn login(ctx: web::Data<AppContext<'_>>, req: HttpRequest, session: Session, form: Option<web::Form<LoginData>>, identity: Option<Identity>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template_and_layout("auth/login", "layout_auth");

    view.set("registration_enabled", ctx.config.registration_enabled);

    if let Some(identity) = identity {
        let user = Users::find_by_id(identity.user_id).one(&ctx.db).await?;

        if user.is_some() {
            view.redirect("/reports", true);
            return Ok(view);
        } else {
            session.remove("uid");
        }
    }

    view.set("form", &form);

    let Some(fields) = form else {
        return Ok(view);
    };

    if let Err(errors) = fields.validate() {
        view.set("errors", &errors);
        return Ok(view);
    }

    let user = Users::find().filter(users::Column::Email.eq(&fields.email)).one(&ctx.db).await?;

    let Some(user) = user else {
        let _ = bcrypt::hash("I want this else branch to take as much time", bcrypt::DEFAULT_COST);
        view.set("error_message", "Login failed; Invalid email or password.");
        return Ok(view);
    };

    let password_hash = std::str::from_utf8(&user.password)?;

    if !bcrypt::verify(&fields.password, password_hash)? {
        view.set("error_message", "Login failed; Invalid email or password.");
        return Ok(view);
    }

    if ctx.config.require_email_verification && user.email_verification_hash.is_some() {
        view.set("error_message", "Your email is not yet verified.");

        view.set("show_resend_verification", true);

        return Ok(view);
    }

    session.insert("uid", user.user_id)?;

    if session.get::<bool>(&format!("seen_{}", user.user_id))?.is_none() {
        let ip = {
            let conn_info = req.connection_info();
            conn_info.realip_remote_addr().map(|s| s.to_string())
        };

        let user_agent = if let Some(value) = req.headers().get(http::header::USER_AGENT) {
            value.to_str()?.to_owned()
        } else {
            "Unknown".into()
        };

        let email = lettre::Message::builder()
            .from(ctx.config.email_from.clone().into())
            .to(user.email.parse()?)
            .subject("Security Alert: Sign-in from a new device")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(ctx.hb.render(
                "email/new_login",
                &serde_json::json!({
                    "ip": ip,
                    "user_agent": user_agent,
                    "title": "Security Alert: Sign-in from a new device"
                }),
            )?)?;

        if let Some(mailer) = ctx.mailer.as_ref() {
            if let Err(e) = mailer.send(email).await {
                log::error!("Error sending sign-in from a new device email: {:?}", e);
            }
        }
    }

    session.insert(format!("seen_{}", user.user_id), true)?;

    view.redirect("/reports", true);

    Ok(view)
}

#[route("/logout", method = "GET")]
async fn logout(session: Session) -> Result<ViewModel> {
    let mut view = ViewModel::default();

    session.remove("uid");

    view.redirect("/login", true);

    Ok(view)
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct PasswordResetRequestForm {
    #[validate(email(message = "A valid email address is required"), length(max = 320, message = "Must be less than 320 chars"))]
    email: String,
}

#[route("/auth/password-reset-request", method = "GET", method = "POST")]
async fn request_password_reset(ctx: web::Data<AppContext<'static>>, req: HttpRequest, form: Option<web::Form<PasswordResetRequestForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template_and_layout("auth/password_reset_request", "layout_auth");

    view.set("form", &form);

    let Some(fields) = form else {
        return Ok(view);
    };

    if let Err(errors) = fields.validate() {
        view.set("errors", &errors);
        return Ok(view);
    }

    let pw_reset_ctx = ctx.clone();
    let email = fields.email.clone();

    // try for constant time response, regardless if user exists or not
    actix_web::rt::spawn(async move {
        if let Err(e) = password_reset_request_in_bg(pw_reset_ctx, req, email).await {
            log::error!("Password reset request error: {}", e);
        }
    });

    view.set("success", true);

    Ok(view)
}

async fn password_reset_request_in_bg(ctx: web::Data<AppContext<'_>>, req: HttpRequest, email: String) -> Result<()> {
    let user = Users::find().filter(users::Column::Email.eq(&email)).one(&ctx.db).await?;

    let Some(user) = user else { return Ok(()) };

    let reset_hash: String = rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();

    let mut user = user.into_active_model();
    user.password_reset_hash = ActiveValue::set(Some(reset_hash.clone()));
    user.password_reset_hash_created = ActiveValue::set(Some(Utc::now().naive_utc()));
    let user = user.save(&ctx.db).await?.try_into_model()?;

    let (scheme, ip) = {
        let conn_info = req.connection_info();
        (conn_info.scheme().to_string(), conn_info.realip_remote_addr().map(|s| s.to_string()))
    };

    let user_agent = if let Some(value) = req.headers().get(http::header::USER_AGENT) {
        value.to_str()?.to_owned()
    } else {
        "Unknown".into()
    };

    let title = format!("Password reset for {} account", ctx.config.base_url);

    let email = lettre::Message::builder()
        .from(ctx.config.email_from.clone().into())
        .to(user.email.parse()?)
        .subject(title.clone())
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(ctx.hb.render(
            "email/password_reset",
            &serde_json::json!({
                "hash": reset_hash,
                "base_url": ctx.config.base_url,
                "scheme": scheme,
                "ip": ip,
                "user_agent": user_agent,
                "title": title
            }),
        )?)?;

    if let Some(mailer) = ctx.mailer.as_ref() {
        mailer.send(email).await?;
    }

    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
struct PasswordResetForm {
    #[validate(length(min = 8, message = "Must be at least 8 characters long"))]
    new_password: String,
    #[validate(
        must_match(other = "new_password", message = "Password do not match"),
        length(min = 8, message = "Must be at least 8 characters long")
    )]
    new_password_repeat: String,
}

#[route("/auth/password-reset/{hash}", method = "GET", method = "POST")]
async fn reset_password(ctx: web::Data<AppContext<'_>>, path: web::Path<String>, form: Option<web::Form<PasswordResetForm>>) -> Result<ViewModel> {
    let mut view = ViewModel::with_template_and_layout("auth/password_reset", "layout_auth");

    let hash = path.into_inner();
    view.set("hash", &hash);

    let user = Users::find()
        .filter(users::Column::PasswordResetHash.eq(hash))
        .one(&ctx.db)
        .await?
        .ok_or(Error::new("Password reset link expired. Go to forgotten password and try again."))?;

    let now = Utc::now();

    let Some(hash_created) = user.password_reset_hash_created else {
        return Err(Error::Internal(anyhow!("Password reset hash present, but without a creation date")));
    };

    if now - hash_created.and_utc() > TimeDelta::hours(1) {
        return Err(Error::new("Password reset request not found or expired."));
    }

    // verified

    view.set("form", &form);

    let Some(fields) = form else {
        return Ok(view);
    };

    if let Err(errors) = fields.validate() {
        view.set("errors", &errors);
        return Ok(view);
    }

    // valid

    let hashed_password = bcrypt::hash(&fields.new_password, bcrypt::DEFAULT_COST)?;

    let mut user = user.into_active_model();
    user.password = ActiveValue::set(hashed_password.into_bytes());
    user.password_reset_hash = ActiveValue::set(None);
    user.password_reset_hash_created = ActiveValue::set(None);
    user.save(&ctx.db).await?;

    view.set("success", true);

    Ok(view)
}
