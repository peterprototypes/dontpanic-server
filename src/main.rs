use std::sync::Arc;

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::{fn_service, ServiceRequest, ServiceResponse},
    middleware, web, App, HttpServer,
};

use chrono::prelude::*;
use chrono_tz::Tz;
use handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderErrorReason};
use key_lock::KeyLock;
use lettre::{transport::smtp::PoolConfig, AsyncSmtpTransport, Tokio1Executor};
use tokio::sync::mpsc;

use sea_orm::{prelude::*, ConnectOptions, Database, IntoActiveModel, TryIntoModel};

use migration::{Migrator, MigratorTrait};

mod config;
mod entity;
mod entity_extensions;
mod error;
mod event;
mod handlers;
mod identity;
mod notifications;

use config::Config;
use notifications::Notification;

pub use error::Error;
pub use identity::Identity;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Clone)]
pub struct AppContext<'reg> {
    pub config: Config,
    pub hb: Handlebars<'reg>,
    pub db: DatabaseConnection,
    pub mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    pub notifications: mpsc::UnboundedSender<Notification>,
    pub locked_projects: Arc<KeyLock<u32>>,
}

impl AppContext<'static> {
    pub async fn new() -> anyhow::Result<Self> {
        let config = Config::from_env()?;

        let mut connection_opt = ConnectOptions::new(&config.database_url);
        connection_opt.sqlx_logging(false);

        let connection = Database::connect(connection_opt).await?;
        Migrator::up(&connection, None).await?;

        create_default_user(&connection, &config).await?;

        let mut handlebars = Handlebars::new();
        handlebars.set_dev_mode(cfg!(debug_assertions));
        handlebars.register_templates_directory("./templates", Default::default())?;
        handlebars.register_helper("dateFmt", Box::new(date));

        // mailer
        let mailer = if let Some(url) = config.email_url.as_ref() {
            let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::from_url(url)?
                .pool_config(PoolConfig::new().max_size(100))
                .build();

            Some(mailer)
        } else {
            None
        };

        let (notifications, mut notifications_rx) = mpsc::unbounded_channel();

        let ctx = Self {
            config,
            hb: handlebars,
            db: connection,
            mailer,
            notifications,
            locked_projects: Arc::new(KeyLock::new()),
        };

        // message handler
        let handler_context = ctx.clone();

        actix_web::rt::spawn(async move {
            log::info!("Notifications handler task started");

            loop {
                let Some(notification) = notifications_rx.recv().await else {
                    log::info!("Notifications handler receiving channel closed");
                    break;
                };

                if let Err(e) = notifications::send(&handler_context, &notification).await {
                    log::error!("Error sending notification: {:?} error: {:?}", notification, e);
                }
            }
        });

        Ok(ctx)
    }

    pub async fn testing() -> anyhow::Result<Self> {
        let mut config = Config::from_env()?;

        let connection = Database::connect(ConnectOptions::new(&config.database_url)).await?;
        Migrator::up(&connection, None).await?;

        config.default_user_email = Some("testing@dontpanic.rs".into());
        config.default_user_password = Some("password".into());
        config.default_user_organization = Some("Testing Organization".into());

        create_default_user(&connection, &config).await?;

        let mut handlebars = Handlebars::new();
        handlebars.set_dev_mode(cfg!(debug_assertions));
        handlebars.register_templates_directory("./templates", Default::default())?;
        handlebars.register_helper("dateFmt", Box::new(date));

        let (notifications, _notifications_rx) = mpsc::unbounded_channel();

        let ctx = Self {
            config,
            hb: handlebars,
            db: connection,
            mailer: None,
            notifications,
            locked_projects: Arc::new(KeyLock::new()),
        };

        Ok(ctx)
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let env = env_logger::Env::new().default_filter_or(if cfg!(debug_assertions) {
        "debug,handlebars=info"
    } else {
        "info"
    });
    env_logger::init_from_env(env);

    log::info!("Starting");

    let ctx = AppContext::new().await?;

    let bind_addr = ctx.config.bind_addr;

    log::info!("Starting http server. Listen: {}", bind_addr);

    let cookie_secret = Key::from(&ctx.config.cookie_secret);

    HttpServer::new(move || {
        let cors = if cfg!(debug_assertions) {
            Cors::permissive()
        } else {
            Cors::default().allowed_methods(vec!["POST", "GET"])
        };

        App::new()
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), cookie_secret.clone())
                    .cookie_name("dontpanic-session".into())
                    .build(),
            )
            .app_data(web::Data::new(ctx.clone()))
            .configure(handlers::ingress::routes)
            .service(web::scope("/api").configure(handlers::routes))
            .service(
                Files::new("/", "./frontend/dist")
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .default_handler(fn_service(|req: ServiceRequest| async {
                        let (req, _) = req.into_parts();
                        let file = NamedFile::open_async("./frontend/dist/index.html").await?;
                        let res = file.into_response(&req);
                        Ok(ServiceResponse::new(req, res))
                    })),
            )
            .wrap(middleware::Logger::default())
    })
    .shutdown_timeout(10)
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}

fn date(h: &Helper, _: &Handlebars, _: &Context, _rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let date_param = h
        .hash_get("date")
        .map(|v| v.value())
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("dateFmt", 0))?;
    let tz_name_param = h
        .hash_get("tz")
        .map(|v| v.value())
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("dateFmt", 1))?;
    let simple = h
        .hash_get("simple")
        .map(|v| v.value())
        .map(|v| v.render())
        .filter(|s| !s.is_empty());

    let date = date_param.render();
    let tz_name = tz_name_param.render();

    let tz_name = if tz_name.is_empty() { "UTC".to_string() } else { tz_name };

    let tz: Tz = tz_name
        .parse()
        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))?;

    let date_user = NaiveDateTime::parse_and_remainder(&date, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))?
        .0
        .and_utc()
        .with_timezone(&tz);

    if simple.is_none() {
        out.write(&date_user.format("%a %b %e %T %Y").to_string())?;
    } else {
        out.write(&date_user.format("%e %b %Y @ %R").to_string())?;
    }

    Ok(())
}

async fn create_default_user(db: &DatabaseConnection, config: &Config) -> Result<()> {
    use entity::organization_users;
    use entity::organizations;
    use entity::prelude::*;
    use entity::users;
    use sea_orm::prelude::*;
    use sea_orm::ActiveValue;

    let Some(user_email) = config.default_user_email.as_ref() else {
        return Ok(());
    };

    let Some(user_pass) = config.default_user_password.as_ref() else {
        return Ok(());
    };

    let hashed_password = bcrypt::hash(user_pass, bcrypt::DEFAULT_COST)?;

    let probe = Users::find()
        .filter(users::Column::Email.eq(user_email))
        .one(db)
        .await?;

    let mut user = if let Some(user) = probe {
        user.into_active_model()
    } else {
        log::info!("Creating default user {}", user_email);

        users::ActiveModel {
            email: ActiveValue::set(user_email.clone()),
            iana_timezone_name: ActiveValue::set(config.default_user_timezone.to_string()),
            ..Default::default()
        }
    };

    let is_new = user.user_id.is_not_set();

    user.password = ActiveValue::set(hashed_password.into_bytes());
    let user = user.save(db).await?.try_into_model()?;

    if is_new {
        let organization_name = config
            .default_user_organization
            .as_deref()
            .unwrap_or("Default Organization");

        let requests_limit = config.organization_requests_limit;

        let organization = organizations::ActiveModel {
            name: ActiveValue::set(organization_name.to_string()),
            is_enabled: ActiveValue::set(1),
            requests_limit: ActiveValue::set(requests_limit),
            requests_count_start: ActiveValue::set(requests_limit.map(|_| Utc::now().naive_utc())),
            ..Default::default()
        };

        let organization = organization.insert(db).await?.try_into_model()?;

        let organization_member = organization_users::ActiveModel {
            organization_id: ActiveValue::set(organization.organization_id),
            user_id: ActiveValue::set(user.user_id),
            role: ActiveValue::set("owner".to_string()),
            ..Default::default()
        };

        organization_member.insert(db).await?;
    }

    Ok(())
}

#[cfg(test)]
pub async fn test_app_with_auth() -> Result<(
    impl actix_web::dev::Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
    actix_web::cookie::Cookie<'static>,
)> {
    let ctx = crate::AppContext::testing().await.unwrap();

    let signing_key = Key::generate();

    let app = actix_web::test::init_service(
        App::new()
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), signing_key.clone()).build())
            .app_data(web::Data::new(ctx))
            .service(web::scope("/api").configure(handlers::routes)),
    )
    .await;

    let req = actix_web::test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "email": "testing@dontpanic.rs",
            "password": "password"
        }))
        .to_request();

    let resp = actix_web::test::call_service(&app, req).await;
    let sess = resp.response().cookies().next().unwrap().to_owned();
    assert!(resp.status().is_success());

    let sess = actix_web::cookie::Cookie::parse(sess.to_string()).unwrap();

    Ok((app, sess))
}
