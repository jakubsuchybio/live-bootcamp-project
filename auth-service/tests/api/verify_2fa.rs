use crate::helpers_arrange::{create_2fa_payload, setup_2fa_login_started};
use crate::helpers_assert::{assert_error_message, assert_has_auth_cookie, assert_status};
use crate::helpers_harness::TestApp;
use db_test_macro::db_test;
use rstest::rstest;

#[db_test]
async fn should_return_200_if_correct_code() {
    // Arrange
    let mut app = TestApp::new().await;
    let (user, two_fa_data) = setup_2fa_login_started(&app).await;

    // Act
    let response = app
        .post_verify_2fa(&create_2fa_payload(&user.email, &two_fa_data))
        .await;

    // Assert
    assert_status(&response, 200, None);
    assert_has_auth_cookie(&response);
}

#[db_test]
#[rstest]
#[case::email_not_containing_at(serde_json::json!({
            "email": "abc",
            "loginAttemptId": "123e4567-e89b-12d3-a456-426614174000",  // valid UUID
            "2FACode": "123456"
        }))]
#[case::invalid_2fa_code_format(serde_json::json!({
            "email": "user@example.com",
            "loginAttemptId": "123e4567-e89b-12d3-a456-426614174000",  // valid UUID
            "2FACode": "12"
        }))]
#[case::invalid_login_attempt_id_format(serde_json::json!({
            "email": "user@example.com",
            "loginAttemptId": "not-a-uuid",  // invalid UUID format
            "2FACode": "123456"
        }))]
async fn should_return_400_if_invalid_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_verify_2fa(&test_case).await;

    // Assert
    assert_status(
        &response,
        400,
        Some(&format!("Failed for input: {:?}", test_case)),
    );
}

// Struct to represent a test case for invalid credentials
#[derive(Copy, Clone)]
struct InvalidCredentialTest {
    field_name: &'static str,
    json_field: &'static str,
    invalid_value: &'static str,
}

#[db_test]
#[rstest]
#[case::incorrect_email(InvalidCredentialTest {
    field_name: "email",
    json_field: "email",
    invalid_value: "incorrect@example.com"
})]
#[case::incorrect_login_attempt_id(InvalidCredentialTest {
    field_name: "login_attempt_id",
    json_field: "loginAttemptId",
    invalid_value: "00000000-0000-0000-0000-000000000000"
})]
#[case::incorrect_2fa_code(InvalidCredentialTest {
    field_name: "two_fa_code",
    json_field: "2FACode",
    invalid_value: "000000"
})]
async fn should_return_401_if_incorrect_credentials(#[case] test_case: InvalidCredentialTest) {
    // Arrange
    let mut app = TestApp::new().await;
    let (user, two_fa_data) = setup_2fa_login_started(&app).await;

    // Create the base payload with valid values
    let mut base_payload = serde_json::json!({
        "email": user.email,
        "loginAttemptId": two_fa_data.login_attempt_id,
        "2FACode": two_fa_data.two_fa_code
    });

    // Apply the invalid value (single line, no branching)
    base_payload[test_case.json_field] = test_case.invalid_value.into();

    // Act
    let response = app.post_verify_2fa(&base_payload).await;

    // Assert
    assert_status(
        &response,
        401,
        Some(&format!(
            "Failed for input with invalid {}",
            test_case.field_name
        )),
    );
    assert_error_message(response, "Incorrect credentials").await;
}

#[db_test]
async fn should_return_401_if_old_code() {
    // Arrange
    let mut app = TestApp::new().await;
    let (user, first_2fa_data) = setup_2fa_login_started(&app).await;

    // Act
    // Second login call (invalidates first login attempt)
    app.post_login(&user.login_payload()).await;

    // Attempt with old login_attempt_id and code
    let response = app
        .post_verify_2fa(&create_2fa_payload(&user.email, &first_2fa_data))
        .await;

    // Assert
    assert_status(&response, 401, None);
}

#[db_test]
async fn should_return_401_if_same_code_twice() {
    // Arrange
    let mut app = TestApp::new().await;
    let (user, two_fa_data) = setup_2fa_login_started(&app).await;

    let request_body = create_2fa_payload(&user.email, &two_fa_data);

    // Act
    let first_response = app.post_verify_2fa(&request_body).await;
    let second_response = app.post_verify_2fa(&request_body).await;

    // Assert
    assert_status(&first_response, 200, None);
    assert_has_auth_cookie(&first_response);
    assert_status(&second_response, 401, None);
}

#[db_test]
#[rstest]
#[case::empty_json(serde_json::json!({}))]
#[case::missing_email(serde_json::json!({
            "loginAttemptId": "123e4567-e89b-12d3-a456-426614174000",
            "2FACode": "123456"
        }))]
#[case::missing_login_attempt_id(serde_json::json!({
            "email": "user@example.com",
            "2FACode": "123456"
        }))]
#[case::missing_2fa_code(serde_json::json!({
            "email": "user@example.com",
            "loginAttemptId": "123e4567-e89b-12d3-a456-426614174000"
        }))]
#[case::invalid_type(serde_json::json!({
            "email": 12345,
            "loginAttemptId": "123e4567-e89b-12d3-a456-426614174000",
            "2FACode": "123456"
        }))]
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_verify_2fa(&test_case).await;

    // Assert
    assert_status(
        &response,
        422,
        Some(&format!("Failed for input: {:?}", test_case)),
    );
}
