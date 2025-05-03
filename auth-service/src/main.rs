use auth_service::{
    prod, AppState, Application, HashMapTwoFACodeStore, HashMapUserStore, HashSetBannedTokenStore,
    MockEmailClient,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = HashMapUserStore::new();
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
