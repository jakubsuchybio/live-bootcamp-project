use crate::helpers_arrange::{get_2fa_code_tuple, setup_registered_user, TestUser};
use crate::helpers_assert::{assert_has_auth_cookie, assert_status};
use crate::helpers_harness::TestApp;
use auth_service::TwoFactorAuthResponse;
use db_test_macro::db_test;
use rstest::rstest;
use secrecy::Secret;

#[db_test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    // Arrange
    let mut app = TestApp::new().await;
    let user = TestUser::new();
    setup_registered_user(&app, &user).await;

    // Act
    let response = app.post_login(&user.login_payload()).await;

    // Assert
    assert_status(&response, 200, None);
    assert_has_auth_cookie(&response);
}

#[db_test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    // Arrange
    let mut app = TestApp::new().await;
    let user = TestUser::new_with_2fa();
    setup_registered_user(&app, &user).await;

    // Act
    let response = app.post_login(&user.login_payload()).await;

    // Assert
    assert_status(&response, 206, None);
    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    // Verify 2FA code was generated and matches the login attempt ID
    let (login_attempt_id, _) = get_2fa_code_tuple(&app, Secret::new(user.email)).await;
    assert_eq!(login_attempt_id, json_body.login_attempt_id);
}

#[db_test]
#[rstest]
#[case::email_not_containing_at(serde_json::json!({
            "email": "abc",
            "password": "12345678",
        }))]
#[case::password_too_short(serde_json::json!({
            "email": "text@example.com",
            "password": "123",
        }))]
async fn should_return_400_if_invalid_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_login(&test_case).await;

    // Assert
    assert_status(
        &response,
        400,
        Some(&format!("Failed for input: {:?}", test_case)),
    );
}

#[db_test]
async fn should_return_401_if_email_not_registered() {
    // Arrange
    let mut app = TestApp::new().await;
    let user = TestUser::new();
    setup_registered_user(&app, &user).await;

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": "unregistered@example.com",
            "password": user.password,
        }))
        .await;

    // Assert
    assert_status(&response, 401, None);
}

#[db_test]
async fn should_return_401_if_wrong_password() {
    // Arrange
    let mut app = TestApp::new().await;
    let user = TestUser::new();
    setup_registered_user(&app, &user).await;

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": user.email,
            "password": "wrongpassword",
        }))
        .await;

    // Assert
    assert_status(&response, 401, None);
}

#[db_test]
#[rstest]
#[case::missing_email_field(serde_json::json!({
            "password": "password123",
        }))]
#[case::missing_password_field(serde_json::json!({
            "email": "test@example.com",
        }))]
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_login(&test_case).await;

    // Assert
    assert_status(
        &response,
        422,
        Some(&format!("Failed for input: {:?}", test_case)),
    );
}
