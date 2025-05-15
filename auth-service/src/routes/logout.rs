// use axum::{extract::State, http::StatusCode};
// use axum_extra::extract::cookie::Cookie;
// use axum_extra::extract::CookieJar;

// use crate::{
//     domain::AuthAPIError,
//     utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
//     AppState,
// };

// pub async fn logout(
//     State(state): State<AppState>,
//     jar: CookieJar,
// ) -> Result<(StatusCode, CookieJar), AuthAPIError> {
//     // Retrieve JWT cookie from the `CookieJar`
//     let Some(cookie) = jar.get(JWT_COOKIE_NAME) else {
//         return Err(AuthAPIError::MissingToken);
//     };

//     // Validate JWT
//     let token = cookie.value().to_owned();
//     let Ok(_claims) = validate_token(&token, state.banned_token_store.clone()).await else {
//         return Err(AuthAPIError::InvalidToken);
//     };

//     // Add the token to the banned token store
//     if let Err(_) = state
//         .banned_token_store
//         .write()
//         .await
//         .add_banned_token(token)
//         .await
//     {
//         return Err(AuthAPIError::UnexpectedError);
//     }

//     // Delete JWT cookie from the `CookieJar`
//     let mut cookie_for_removal = Cookie::from(JWT_COOKIE_NAME);
//     cookie_for_removal.set_path("/"); // Needed for https context removal
//     let jar = jar.remove(cookie_for_removal);

//     Ok((StatusCode::OK, jar))
// }
