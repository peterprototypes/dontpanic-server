use actix_web::{
    post,
    web::{self, Data, Json, Path},
    Responder,
};
use sea_orm::{prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use validator::Validate;

use crate::entity::prelude::*;

use crate::notifications::ReportStatus;
use crate::{AppContext, Error, Identity, Result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(save).service(delete).service(test);
}

#[derive(Deserialize, Validate)]
struct WebhookInput {
    #[validate(url(message = "Please enter a valid URL"))]
    webhook_url: String,
}

#[post("/save")]
async fn save(
    ctx: Data<AppContext<'_>>,
    id: Identity,
    path: Path<u32>,
    input: Json<WebhookInput>,
) -> Result<impl Responder> {
    input.validate()?;
    let input = input.into_inner();

    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _ = user
        .role(&ctx.db, project.organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    let mut project_model = project.into_active_model();
    project_model.webhook = ActiveValue::set(Some(input.webhook_url));
    project_model.save(&ctx.db).await?;

    Ok(Json(()))
}

#[post("/delete")]
async fn delete(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _ = user
        .role(&ctx.db, project.organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    let mut project_model = project.into_active_model();
    project_model.webhook = ActiveValue::set(None);
    project_model.save(&ctx.db).await?;

    Ok(Json(()))
}

#[post("/test")]
async fn test(ctx: Data<AppContext<'_>>, id: Identity, path: Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();
    let project = Projects::find_by_id(project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let user = id.user(&ctx).await?;
    let _ = user
        .role(&ctx.db, project.organization_id)
        .await?
        .ok_or(Error::LoginRequired)?;

    let Some(webhook_url) = project.webhook else {
        return Err(Error::new("Webhook URL is not set"));
    };

    let example_log = serde_json::json!([{
        "timestamp": chrono::Utc::now().timestamp() as u64,
        "level": 1,
        "message": "Error message",
        "module": "my_module",
        "file": "src/main.rs",
        "line": 42,
    }]);

    let params = serde_json::json!({
        "status": ReportStatus::New,
        "title": "Called `Option::unwrap()` on a `None` value (Webhook Test)",
        "project": project.name,
        "environment": "development",
        "backtrace": EXAMPLE_BACKTRACE,
        "log": serde_json::to_string(&example_log).unwrap(),
        "url": "https://dontpanic.rs",
    });

    let client = reqwest::Client::new();
    let _res = client.post(webhook_url).json(&params).send().await?;

    Ok(Json(()))
}

const EXAMPLE_BACKTRACE: &str = r#"stack backtrace:
0: playground::main::h6849180917e9510b (0x55baf1676201)
            at src/main.rs:4
1: std::rt::lang_start::{{closure}}::hb3ceb20351fe39ee (0x55baf1675faf)
            at /rustc/3c235d5600393dfe6c36eeed34042efad8d4f26e/src/libstd/rt.rs:64
2: {{closure}} (0x55baf16be492)
            at src/libstd/rt.rs:49
    do_call<closure,i32>
            at src/libstd/panicking.rs:293
3: __rust_maybe_catch_panic (0x55baf16c00b9)
            at src/libpanic_unwind/lib.rs:87
4: try<i32,closure> (0x55baf16bef9c)
            at src/libstd/panicking.rs:272
    catch_unwind<closure,i32>
            at src/libstd/panic.rs:388
    lang_start_internal
            at src/libstd/rt.rs:48
5: std::rt::lang_start::h2c4217f9057b6ddb (0x55baf1675f88)
            at /rustc/3c235d5600393dfe6c36eeed34042efad8d4f26e/src/libstd/rt.rs:64
6: main (0x55baf16762f9)
7: __libc_start_main (0x7fab051b9b96)
8: _start (0x55baf1675e59)
9: <unknown> (0x0)"#;
