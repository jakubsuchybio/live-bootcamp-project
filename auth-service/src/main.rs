use auth_service::{
    configure_redis, get_postgres_pool, init_tracing, prod, AppState, Application,
    PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore, SlackMessageClient,
    DATABASE_URL, SLACK_WEBHOOK,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    let pg_pool = configure_postgresql().await;
    let redis_conn = Arc::new(RwLock::new(configure_redis()));
    let slack_client = configure_slack_email_client();

    let user_store = PostgresUserStore::new(pg_pool);
    let banned_token_store = RedisBannedTokenStore::new(redis_conn.clone());
    let two_fa_code_store = RedisTwoFACodeStore::new(redis_conn.clone());
    let app_state = AppState {
        user_store: Arc::from(RwLock::from(user_store)),
        banned_token_store: Arc::from(RwLock::from(banned_token_store)),
        two_fa_code_store: Arc::from(RwLock::from(two_fa_code_store)),
        email_client: Arc::from(RwLock::from(slack_client)),
    };

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_slack_email_client() -> SlackMessageClient {
    // Create a new SlackMessageClient with the webhook URL
    SlackMessageClient::new(&SLACK_WEBHOOK)
}
