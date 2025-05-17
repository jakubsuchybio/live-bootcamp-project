use crate::helpers_arrange::{create_2fa_payload, setup_2fa_login_started};
use crate::helpers_assert::{assert_has_auth_cookie, assert_status};
use crate::helpers_harness::{get_random_email, TestApp};
use auth_service::{Email, ErrorResponse, LoginAttemptId, TwoFACode, TwoFactorAuthResponse};
use db_test_macro::db_test;
use rstest::rstest;
use secrecy::{ExposeSecret, Secret};

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

#[db_test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    // --------------------------

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(Secret::new(random_email.clone())).unwrap())
        .await
        .unwrap();

    let two_fa_code = code_tuple.1.as_ref();

    // --------------------------

    let incorrect_email = get_random_email();
    let incorrect_login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let incorrect_two_fa_code = TwoFACode::default().as_ref().to_owned();

    let test_cases = vec![
        (
            incorrect_email.as_str(),
            login_attempt_id.as_str(),
            two_fa_code.expose_secret().as_str(),
        ),
        (
            random_email.as_str(),
            incorrect_login_attempt_id.expose_secret(),
            two_fa_code.expose_secret().as_str(),
        ),
        (
            random_email.as_str(),
            login_attempt_id.as_str(),
            &incorrect_two_fa_code.expose_secret().as_str(),
        ),
    ];

    for (email, login_attempt_id, code) in test_cases {
        let request_body = serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id,
            "2FACode": code
        });

        let response = app.post_verify_2fa(&request_body).await;

        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:?}",
            request_body
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Incorrect credentials".to_owned()
        );
    }
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
