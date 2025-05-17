use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode},
    utils::auth::generate_auth_cookie,
    AppState,
};

#[tracing::instrument(name = "Logging in", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email = request.email;
    let password = request.password;

    // Validate input
    let Ok(email) = Email::parse(email) else {
        return Err(AuthAPIError::InvalidCredentials);
    };

    let Ok(password) = Password::parse(password) else {
        return Err(AuthAPIError::InvalidCredentials);
    };

    let user_store = &state.user_store.read().await;
    if user_store.validate_user(&email, &password).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let Ok(user) = user_store.get_user(&email).await else {
        return Err(AuthAPIError::IncorrectCredentials);
    };

    // Handle request based on user's 2FA configuration
    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

#[tracing::instrument(name = "Logging in with 2fa", skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let login_atempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    if let Err(e) = two_fa_code_store
        .add_code(email.clone(), login_atempt_id.clone(), two_fa_code.clone())
        .await
    {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    let email_client = state.email_client.read().await;
    if let Err(e) = email_client
        .send_email(email, "Your 2FA Code", two_fa_code.as_ref().expose_secret())
        .await
    {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_atempt_id.as_ref().expose_secret().to_owned(),
    }));

    Ok((jar, (StatusCode::PARTIAL_CONTENT, response)))
}

#[tracing::instrument(name = "Logging in without 2fa", skip_all)]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    // Return success response
    let auth_cookie = match generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(e) => return Err(AuthAPIError::UnexpectedError(e)),
    };

    let updated_jar = jar.add(auth_cookie);

    Ok((
        updated_jar,
        (StatusCode::OK, Json(LoginResponse::RegularAuth)),
    ))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
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
