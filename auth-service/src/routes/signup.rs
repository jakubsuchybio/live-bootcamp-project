use crate::{
    domain::{AuthAPIError, Email, Password, User},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    // Validate input
    let Ok(email) = Email::parse(&email) else {
        return Err(AuthAPIError::InvalidCredentials);
    };

    let Ok(password) = Password::parse(&password) else {
        return Err(AuthAPIError::InvalidCredentials);
    };

    // Create user object
    let user = User::new(email, password, request.requires_2fa);

    // Acquire lock on user store
    let mut user_store = state.user_store.write().await;

    // Check if user already exists
    if let Ok(_) = user_store.get_user(&user.email).await {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    // Add user to store
    user_store
        .add_user(user)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    // Return success response
    let response = Json(SignupResponse {
        message: "User created successfully".to_string(),
    });

    Ok((StatusCode::CREATED, response))
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
