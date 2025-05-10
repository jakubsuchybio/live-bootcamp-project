use crate::helpers_arrange::TestUser;
use crate::helpers_assert::{assert_error_message, assert_status};
use crate::helpers_harness::TestApp;
use db_test_macro::db_test;
use rstest::rstest;

#[db_test]
async fn should_return_201_if_valid_input() {
    // Arrange
    let mut app = TestApp::new().await;
    let user = TestUser::new();

    // Act
    let response = app.post_signup(&user.signup_payload()).await;

    // Assert
    assert_status(&response, 201, None);
}

#[db_test]
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
async fn should_return_400_if_invalid_input(#[case] request: serde_json::Value) {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_signup(&request).await;

    // Assert
    assert_status(
        &response,
        400,
        Some(&format!("Failed for request: {:?}", request)),
    );
    assert_error_message(response, "Invalid credentials").await;
}

#[db_test]
async fn should_return_409_if_email_already_exists() {
    // Arrange
    let mut app = TestApp::new().await;
    let user = TestUser::new();

    // Act
    let first_response = app.post_signup(&user.signup_payload()).await;
    let second_response = app.post_signup(&user.signup_payload()).await;

    // Assert
    assert_status(&first_response, 201, None);
    assert_status(&second_response, 409, None);
    assert_error_message(second_response, "User already exists").await;
}

#[db_test]
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
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_signup(&test_case).await;

    // Assert
    assert_status(
        &response,
        422,
        Some(&format!("Failed for input: {:?}", test_case)),
    );
}
