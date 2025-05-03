mod app_state;
mod services;

mod domain;
mod routes;
mod utils;

use askama::Template;
use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    serve::Serve,
    Extension, Router,
};
use routes::{login, logout, signup, verify_2fa, verify_token};
use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub use app_state::{AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType};
pub use domain::{Email, ErrorResponse, LoginAttemptId, TwoFACode};
pub use routes::TwoFactorAuthResponse;
pub use services::{
    HashMapTwoFACodeStore, HashMapUserStore, HashSetBannedTokenStore, MockEmailClient,
};
pub use utils::constants::{prod, test};
pub use utils::JWT_COOKIE_NAME;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    prefix: String,
}

async fn root(Extension(prefix): Extension<String>) -> impl axum::response::IntoResponse {
    let template = IndexTemplate { prefix };
    Html(template.render().unwrap())
}

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field,
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        // Allow the app service(running on our local machine and in production) to call the auth service
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            "http://127.0.0.1:8000".parse()?,
            "https://live-bootcamp.biosek.cz/auth".parse()?,
        ];

        let cors = CorsLayer::new()
            // Allow GET and POST requests
            .allow_methods([Method::GET, Method::POST])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .route("/", get(root))
            .nest_service("/assets", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .route("/health", get(health))
            .with_state(app_state.clone())
            .layer(middleware::from_fn(handle_prefix))
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);

        // Set up graceful shutdown
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        // Handle both SIGINT (Ctrl+C) and SIGTERM (Docker stop)
        tokio::spawn(async move {
            let ctrl_c = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }

            println!("Received shutdown signal, shutting down gracefully...");
            let _ = tx.send(());
        });

        // Start server with graceful shutdown
        let result = self
            .server
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;

        println!("Server shutdown complete");
        result
    }
}

// New middleware to handle the prefix
async fn handle_prefix(mut request: Request, next: Next) -> Response {
    let prefix = request
        .headers()
        .get("X-Forwarded-Prefix")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    request.extensions_mut().insert(prefix);
    next.run(request).await
}

// Simple health check endpoint for container health monitoring
async fn health() -> impl IntoResponse {
    StatusCode::OK
}
