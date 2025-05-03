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

    // Handle request based on user's 2FA configuration
    match user.requires_2fa {
        true => handle_2fa(jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

async fn handle_2fa(
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    (
        jar,
        Ok((
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
                message: "2FA required".to_string(),
                login_attempt_id: "123456".to_string(),
            })),
        )),
    )
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // Return success response
    let Ok(auth_cookie) = auth::generate_auth_cookie(&email) else {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };

    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
