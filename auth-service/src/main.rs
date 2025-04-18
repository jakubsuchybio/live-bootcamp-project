use auth_service::{AppState, Application, HashmapUserStore};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore {
        ..Default::default()
    };
    let app_state = AppState {
        user_store: Arc::from(RwLock::from(user_store)),
    };
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
