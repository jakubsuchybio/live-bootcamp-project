use auth_service::{
    get_postgres_pool, prod, AppState, Application, HashMapTwoFACodeStore, HashSetBannedTokenStore,
    MockEmailClient, PostgresUserStore, DATABASE_URL,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;

    let user_store = PostgresUserStore::new(pg_pool);
    let banned_token_store = HashSetBannedTokenStore::new();
    let two_fa_code_store = HashMapTwoFACodeStore::new();
    let email_client = MockEmailClient {};
    let app_state = AppState {
        user_store: Arc::from(RwLock::from(user_store)),
        banned_token_store: Arc::from(RwLock::from(banned_token_store)),
        two_fa_code_store: Arc::from(RwLock::from(two_fa_code_store)),
        email_client: Arc::from(RwLock::from(email_client)),
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
