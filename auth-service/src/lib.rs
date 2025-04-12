mod routes;

use askama::Template;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::{Html, Response},
    routing::{get, post},
    serve::Serve,
    Extension, Router,
};
use routes::{login, logout, signup, verify_2fa, verify_token};
use std::error::Error;
use tower_http::services::ServeDir;

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
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .route("/", get(root))
            .nest_service("/assets", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .layer(middleware::from_fn(handle_prefix));

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
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
