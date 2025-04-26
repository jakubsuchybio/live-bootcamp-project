use crate::helpers::{get_random_email, TestApp};
use auth_service::{domain::BannedTokenStore, utils::constants::JWT_COOKIE_NAME};
use rstest::rstest;

#[tokio::test]
async fn should_return_200_valid_token() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": false
        }))
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No jwt auth cookie found");

    // Act
    let response = app
        .post_verify_token(&serde_json::json!({
            "token": auth_cookie.value(),
        }))
        .await;

    // Assert
    assert_eq!(signup_response.status().as_u16(), 201);
    assert_eq!(login_response.status().as_u16(), 200);
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app
        .post_verify_token(&serde_json::json!({
            "token": "invalid_token",
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_token_is_banned() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": false
        }))
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No jwt auth cookie found");

    let token = auth_cookie.value().to_string();
    let logout_response = app.post_logout().await;
    let banned_token_store = app.banned_token_store.read().await;

    // Act
    let response = app
        .post_verify_token(&serde_json::json!({
            "token": token,
        }))
        .await;

    // Assert
    assert_eq!(signup_response.status().as_u16(), 201);
    assert_eq!(login_response.status().as_u16(), 200);
    assert_eq!(logout_response.status().as_u16(), 200);
    assert_eq!(response.status().as_u16(), 401);
    assert!(banned_token_store.check_banned_token(&token).await);
}

#[rstest]
#[case::empty_json(serde_json::json!({}))]
#[case::not_a_token(serde_json::json!({
            "email": "test@example.com",
        }))]
#[tokio::test]
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_verify_token(&test_case).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}
