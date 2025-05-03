use auth_service::{ErrorResponse, JWT_COOKIE_NAME};

/// Extract token from auth cookie in response
pub fn extract_token(response: &reqwest::Response) -> String {
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    
    auth_cookie.value().to_string()
}

/// Assert HTTP status code with optional context
pub fn assert_status(response: &reqwest::Response, expected: u16, context: Option<&str>) {
    let status = response.status().as_u16();
    let context_msg = context.unwrap_or("");
    
    assert_eq!(
        status,
        expected,
        "Expected status {}, got {} {}",
        expected,
        status,
        context_msg
    );
}

/// Assert the response contains an auth cookie
pub fn assert_has_auth_cookie(response: &reqwest::Response) {
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME);
    
    assert!(auth_cookie.is_some(), "No auth cookie found");
    assert!(!auth_cookie.unwrap().value().is_empty(), "Auth cookie value is empty");
}

/// Assert response contains the expected error message
pub async fn assert_error_message(response: reqwest::Response, expected_message: &str) {
    let body = response
        .json::<ErrorResponse>()
        .await
        .expect("Could not deserialize response body to ErrorResponse");
    
    assert_eq!(
        body.error,
        expected_message.to_owned(),
        "Expected error message '{}', got '{}'",
        expected_message,
        body.error
    );
}