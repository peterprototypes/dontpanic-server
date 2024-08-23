use std::env::var;
use std::env::VarError;
use std::ffi::OsStr;
use std::fmt::Display;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use chrono_tz::Tz;
use lettre::address::Address;
use rand::RngCore;
use rand_seeder::SipHasher;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,

    pub cookie_secret: [u8; 64],
    pub database_url: String,

    pub base_url: String,
    pub scheme: String,

    pub email_from: Address,
    pub email_url: Option<String>,

    pub slack_client_id: Option<String>,
    pub slack_client_secret: Option<String>,

    pub default_user_timezone: Tz,

    pub default_user_email: Option<String>,
    pub default_user_password: Option<String>,
    pub default_user_organization: Option<String>,

    pub registration_enabled: bool,
    pub require_email_verification: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::from_filename(".env").ok();

        let slack_client_id = get_var("SLACK_CLIENT_ID").ok();

        // if slack client id is provided - secret is required
        let slack_client_secret = if slack_client_id.is_some() {
            Some(get_var("SLACK_CLIENT_SECRET")?)
        } else {
            get_var("SLACK_CLIENT_SECRET").ok()
        };

        // get and validate default user password
        let default_user_password = get_var("DEFAULT_USER_PASSWORD").ok();

        if let Some(pass) = default_user_password.as_ref() {
            if pass.len() < 8 {
                anyhow::bail!("DEFAULT_USER_PASSWORD must be minimum 8 characters long");
            }
        }

        // extract default user timezone
        let default_user_timezone = get_var("DEFAULT_USER_TIMEZONE").unwrap_or_else(|_| "UTC".into());
        let default_user_timezone = default_user_timezone.parse::<Tz>().unwrap_or_default();

        let cookie_secret = if let Ok(var) = get_var("COOKIE_SECRET") {
            let hasher = SipHasher::from(var);
            let mut rng = hasher.into_rng();
            let mut data = [0u8; 64];
            rng.fill_bytes(&mut data);
            data
        } else {
            let mut data = [0u8; 64];
            rand::thread_rng().fill_bytes(&mut data);
            data
        };

        Ok(Self {
            bind_addr: get_var("BIND_ADDRESS").ok().unwrap_or_else(|| "0.0.0.0:8080".into()).parse()?,
            cookie_secret,
            database_url: get_var("DATABASE_URL")?,
            base_url: get_var("BASE_URL").ok().unwrap_or_else(|| "localhost".into()),
            scheme: get_var("SCHEME").ok().unwrap_or_else(|| "http".into()),
            email_from: get_var("EMAIL_FROM").ok().unwrap_or_else(|| "no-rely@dontpanic.rs".into()).parse()?,
            email_url: get_var("EMAIL_URL").ok(),
            slack_client_id,
            slack_client_secret,
            default_user_timezone,
            default_user_email: get_var("DEFAULT_USER_EMAIL").ok(),
            default_user_password,
            default_user_organization: get_var("DEFAULT_USER_ORGANIZATION").ok(),
            registration_enabled: get_bool_var("REGISTRATION_ENABLED")?.unwrap_or(true),
            require_email_verification: get_bool_var("REQUIRE_EMAIL_VERIFICATION")?.unwrap_or(true),
        })
    }
}

fn get_var<K: AsRef<OsStr> + Display + Sync + Send + 'static>(key: K) -> Result<String> {
    let res = var(key.as_ref()).context(key)?;
    Ok(res)
}

fn get_bool_var<K: AsRef<OsStr> + Display + Sync + Send + 'static>(key: K) -> Result<Option<bool>> {
    match var(key.as_ref()) {
        Ok(res) => {
            let trim = res.trim();

            Ok(Some(trim == "yes" || trim == "1" || trim == "true"))
        }
        Err(VarError::NotPresent) => Ok(None),
        Err(e) => Err(e).context(key),
    }
}
