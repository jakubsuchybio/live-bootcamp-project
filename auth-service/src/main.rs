use auth_service::{
    utils::constants::prod, AppState, Application, HashMapUserStore, HashSetBannedTokenStore,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = HashMapUserStore::new();
    let banned_token_store = HashSetBannedTokenStore::new();
    let app_state = AppState {
        user_store: Arc::from(RwLock::from(user_store)),
        banned_token_store: Arc::from(RwLock::from(banned_token_store)),
    };
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
