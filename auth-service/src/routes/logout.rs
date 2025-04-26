use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension};
use axum_extra::extract::CookieJar;

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    AppState,
};

pub async fn logout(
    Extension(prefix): Extension<String>,
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

    println!("Before removal: {:?}", jar);
    // Delete JWT cookie from the `CookieJar`
    let mut cloned_cookie = cookie.clone();
    cloned_cookie.make_removal();

    let jar = jar.remove(cloned_cookie);
    println!("After removal: {:?}", jar);

    (jar, Ok(StatusCode::OK))
}
