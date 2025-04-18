use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::ErrorResponse;
use rstest::rstest;

#[tokio::test]
async fn should_return_201_if_valid_input() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // Act
    let response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 201);
}

#[rstest]
#[case::empty_email(serde_json::json!({
            "email": "",
            "password": "password",
            "requires2FA": true
        }))]
#[case::invalid_email_format(serde_json::json!({
            "email": "aaa",
            "password": "password",
            "requires2FA": true
        }))]
#[case::short_password(serde_json::json!({
            "email": "test@example.com",
            "password": "123",
            "requires2FA": true
        }))]
#[tokio::test]
async fn should_return_400_if_invalid_input(#[case] request: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_signup(&request).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        400,
        "Failed for request: {:?}",
        request
    );
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid credentials".to_owned()
    );
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Arrange
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // Act
    // First signup
    let first_response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }))
        .await;

    // Second signup with same email
    let second_response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }))
        .await;

    // Assert
    // First signup successful
    assert_eq!(first_response.status().as_u16(), 201);
    // Second signup fails with conflict
    assert_eq!(second_response.status().as_u16(), 409);

    assert_eq!(
        second_response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    )
}

#[rstest]
#[case::missing_email_field(serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }))]
#[case::missing_password_field(serde_json::json!({
            "email": "test@example.com",
            "requires2FA": true
        }))]
#[case::missing_requires2fa_field(serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))]
#[tokio::test]
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_signup(&test_case).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}
