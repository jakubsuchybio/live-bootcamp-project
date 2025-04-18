use crate::{
    domain::{AuthAPIError, User},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate input
    validate_credentials(&request.email, &request.password)?;

    // Create user object
    let user = User::new(request.email, request.password, request.requires_2fa);

    // Acquire lock on user store
    let mut user_store = state.user_store.write().await;

    // Check if user already exists
    if let Ok(_) = user_store.get_user(&user.email) {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    // Add user to store
    user_store
        .add_user(user)
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    // Return success response
    let response = Json(SignupResponse {
        message: "User created successfully".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

/// Validate email and password
fn validate_credentials(email: &str, password: &str) -> Result<(), AuthAPIError> {
    // Validate email
    if email.is_empty() || !email.contains('@') {
        return Err(AuthAPIError::InvalidCredentials);
    }

    // Validate password
    if password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub message: String,
}
