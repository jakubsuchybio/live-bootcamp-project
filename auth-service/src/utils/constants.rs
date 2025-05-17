use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::Secret;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref DATABASE_URL: Secret<String> = set_database_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
    pub static ref SLACK_WEBHOOK: Secret<String> = set_slack_webhook();
}

fn set_token() -> String {
    dotenv().ok();
    let secret =
        std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET environment variable not set");
    if secret.is_empty() {
        panic!("JWT_SECRET environment variable is empty");
    }
    secret
}

fn set_database_url() -> Secret<String> {
    dotenv().ok();
    let database_url =
        std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL environment variable not set");
    if database_url.is_empty() {
        panic!("DATABASE_URL environment variable is empty");
    }
    Secret::new(database_url)
}

fn set_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

fn set_slack_webhook() -> Secret<String> {
    dotenv().ok();
    let slack_webhook = std_env::var(env::SLACK_WEBHOOK_ENV_VAR)
        .expect("SLACK_WEBHOOK environment variable not set");
    if slack_webhook.is_empty() {
        panic!("DATABASE_URL environment variable is empty");
    }
    Secret::new(slack_webhook)
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const SLACK_WEBHOOK_ENV_VAR: &str = "SLACK_WEBHOOK";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
