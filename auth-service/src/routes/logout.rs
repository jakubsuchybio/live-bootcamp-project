use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    let Some(cookie) = jar.get(JWT_COOKIE_NAME) else {
        return (jar, Err(AuthAPIError::MissingToken));
    };

    // Validate JWT
    let token = cookie.value().to_owned();
    let Ok(_claims) = validate_token(&token).await else {
        return (jar, Err(AuthAPIError::InvalidToken));
    };

    // Delete JWT cookie from the `CookieJar`
    let jar = jar.remove(JWT_COOKIE_NAME);

    (jar, Ok(StatusCode::OK))
}
