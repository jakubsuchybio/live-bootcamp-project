use std::env;

use askama::Template;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use tower_http::services::ServeDir;

fn get_auth_address(prefix: &str, path: &str, ipc: bool) -> String {
    let mut address = match env::var("AUTH_SERVICE_IP") {
        Err(_) => "localhost".to_owned(),
        Ok(addr) if addr.is_empty() => "localhost".to_owned(),
        Ok(addr) => addr,
    };

    if address == "localhost" && ipc {
        address = "auth-service".to_string();
    }

    // Determine protocol based on host - use http for localhost, https for others (production)
    let (protocol, suffix) = if prefix.is_empty() {
        ("http://", ":3000")
    } else {
        ("https://", "/auth")
    };

    // Always use the path with prefix
    format!("{}{}{}{}", protocol, address, suffix, path)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/protected", get(protected))
        .route("/health", get(health))
        .layer(middleware::from_fn(handle_prefix));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());

    // Set up graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = axum::serve(listener, app);

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
    if let Err(e) = server
        .with_graceful_shutdown(async {
            let _ = rx.await;
        })
        .await
    {
        eprintln!("Server error: {}", e);
    }

    println!("Server shutdown complete");
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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
    prefix: String,
}

async fn root(Extension(prefix): Extension<String>) -> impl IntoResponse {
    let login_link = get_auth_address(&prefix, "", false);
    let logout_link = get_auth_address(&prefix, "/logout", false);

    let template = IndexTemplate {
        login_link,
        logout_link,
        prefix,
    };
    Html(template.render().unwrap())
}

async fn protected(jar: CookieJar, Extension(prefix): Extension<String>) -> impl IntoResponse {
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let api_client = reqwest::Client::builder().build().unwrap();

    let verify_token_body = serde_json::json!({
        "token": &jwt_cookie.value(),
    });

    let verify_token_link = get_auth_address(&prefix, "/verify-token", true);
    let verify_token_response = api_client
        .post(&verify_token_link)
        .json(&verify_token_body)
        .send()
        .await;

    println!("verify-token response: {:?}", verify_token_response);

    let Ok(response) = verify_token_response else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match response.status() {
        reqwest::StatusCode::OK => Json(ProtectedRouteResponse {
            img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
        })
        .into_response(),
        _ => response.status().into_response(),
    }
}

#[derive(Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}

// Simple health check endpoint for container health monitoring
async fn health() -> impl IntoResponse {
    StatusCode::OK
}
