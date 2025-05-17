use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    domain::{AuthAPIError, LoginAttemptId, TwoFACode},
    utils::auth,
    AppState, Email,
};

#[tracing::instrument(name = "Verifying 2fa", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email = request.email;
    let login_attempt_id = request.login_attempt_id;
    let two_fa_code = request.two_fa_code;

    // Validate input
    let Ok(email) = Email::parse(email) else {
        return Err(AuthAPIError::InvalidCredentials);
    };
    let Ok(login_attempt_id) = LoginAttemptId::parse(&login_attempt_id) else {
        return Err(AuthAPIError::InvalidCredentials);
    };
    let Ok(two_fa_code) = TwoFACode::parse(&two_fa_code) else {
        return Err(AuthAPIError::InvalidCredentials);
    };

    // Validate 2fa
    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    let Ok(found_two_fa_tuple) = two_fa_code_store.get_code(&email).await else {
        return Err(AuthAPIError::IncorrectCredentials);
    };

    if found_two_fa_tuple.0 != login_attempt_id || found_two_fa_tuple.1 != two_fa_code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    if two_fa_code_store.remove_code(&email).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    // Return success response
    let auth_cookie = match auth::generate_auth_cookie(&email) {
        Ok(auth_cookie) => auth_cookie,
        Err(e) => {
            return Err(AuthAPIError::UnexpectedError(e));
        }
    };

    let updated_jar = jar.add(auth_cookie);

    Ok((updated_jar, StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: Secret<String>,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: Secret<String>,
    #[serde(rename = "2FACode")]
    pub two_fa_code: Secret<String>,
}
