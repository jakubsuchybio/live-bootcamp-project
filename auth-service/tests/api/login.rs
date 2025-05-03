use auth_service::{TwoFactorAuthResponse, JWT_COOKIE_NAME};
use rstest::rstest;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    app.post_signup(&serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    }))
    .await;

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123",
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    app.post_signup(&serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    }))
    .await;

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123",
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 206);
    assert_eq!(
        response
            .json::<TwoFactorAuthResponse>()
            .await
            .expect("Could not deserialize response body to TwoFactorAuthResponse")
            .message,
        "2FA required".to_owned()
    );
}

#[rstest]
#[case::email_not_containing_at(serde_json::json!({
            "email": "abc",
            "password": "12345678",
        }))]
#[case::password_too_short(serde_json::json!({
            "email": "text@example.com",
            "password": "123",
        }))]
#[tokio::test]
async fn should_return_400_if_invalid_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_login(&test_case).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        400,
        "Failed for input: {:?}",
        test_case
    );
}

#[tokio::test]
async fn should_return_401_if_email_not_registered() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    app.post_signup(&serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    }))
    .await;

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": "text@example.com",
            "password": "password123",
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_wrong_password() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();
    app.post_signup(&serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    }))
    .await;

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password12345678",
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[rstest]
#[case::missing_email_field(serde_json::json!({
            "password": "password123",
        }))]
#[case::missing_password_field(serde_json::json!({
            "email": "test@example.com",
        }))]
#[tokio::test]
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_login(&test_case).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}
