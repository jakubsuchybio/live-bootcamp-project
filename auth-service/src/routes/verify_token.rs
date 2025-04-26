use crate::utils::auth::validate_token;
use crate::{domain::AuthAPIError, AppState};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate incoming JWT
    let token = request.token;
    let Ok(_claims) = validate_token(&token, state.banned_token_store.clone()).await else {
        return Err(AuthAPIError::InvalidToken);
    };
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
