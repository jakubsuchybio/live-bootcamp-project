use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    AppState,
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    let Some(cookie) = jar.get(JWT_COOKIE_NAME) else {
        return (jar, Err(AuthAPIError::MissingToken));
    };

    // Validate JWT
    let token = cookie.value().to_owned();
    let Ok(_claims) = validate_token(&token, state.banned_token_store.clone()).await else {
        return (jar, Err(AuthAPIError::InvalidToken));
    };

    let mut banned_token_store = state.banned_token_store.write().await;
    banned_token_store.add_banned_token(token).await;

    // Delete JWT cookie from the `CookieJar`
    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
