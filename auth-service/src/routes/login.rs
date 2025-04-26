use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{AuthAPIError, Email, Password},
    utils::auth,
    AppState,
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = request.email;
    let password = request.password;

    // Validate input
    let Ok(email) = Email::parse(&email) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let Ok(password) = Password::parse(&password) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    // Check if user exists and password is correct
    let user_store = &state.user_store.read().await;
    let Ok(user) = user_store.get_user(&email).await else {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    };

    if user.password != password {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    // Return success response
    let Ok(auth_cookie) = auth::generate_auth_cookie(&email) else {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };

    let updated_jar = jar.add(auth_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}
