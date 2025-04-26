use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }))
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;

    // Act
    let logout_response = app.post_logout().await;

    // Assert
    assert_eq!(signup_response.status().as_u16(), 201);
    assert_eq!(login_response.status().as_u16(), 200);
    assert_eq!(logout_response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }))
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;

    // Act
    let logout_response1 = app.post_logout().await;
    let logout_response2 = app.post_logout().await;

    // Assert
    assert_eq!(signup_response.status().as_u16(), 201);
    assert_eq!(login_response.status().as_u16(), 200);
    assert_eq!(logout_response1.status().as_u16(), 200);
    assert_eq!(logout_response2.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_logout().await;

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    // Arrange
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    // Act
    let response = app.post_logout().await;

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}
