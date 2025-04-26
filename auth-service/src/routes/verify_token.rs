use crate::domain::AuthAPIError;
use crate::utils::auth::validate_token;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

pub async fn verify_token(
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate incoming JWT
    let token = request.token;
    let Ok(_claims) = validate_token(&token).await else {
        return Err(AuthAPIError::InvalidToken);
    };
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
