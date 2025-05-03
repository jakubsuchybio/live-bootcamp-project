use crate::helpers::get_random_email;
use crate::helpers::TestApp;
use auth_service::JWT_COOKIE_NAME;
use auth_service::{Email, LoginAttemptId, TwoFACode};
use auth_service::{ErrorResponse, TwoFactorAuthResponse};
use rstest::rstest;

#[tokio::test]
async fn should_return_200_if_correct_code() {
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
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);
    let login_response_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");
    assert_eq!(login_response_body.message, "2FA required".to_owned());
    assert!(!login_response_body.login_attempt_id.is_empty());

    let login_attempt_id = login_response_body.login_attempt_id;
    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();
    let two_fa_code = code_tuple.1.as_ref();

    // Act
    let two_fa_response = app
        .post_verify_2fa(&serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id,
            "2FACode": two_fa_code
        }))
        .await;

    // Assert
    assert_eq!(two_fa_response.status().as_u16(), 200);
    let auth_cookie = two_fa_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    // Make sure to assert the auth cookie gets set
}

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
#[tokio::test]
async fn should_return_400_if_invalid_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_verify_2fa(&test_case).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        400,
        "Failed for input: {:?}",
        test_case
    );
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
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
    assert_eq!(login_response.status().as_u16(), 206);
    let login_response_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");
    assert!(!login_response_body.login_attempt_id.is_empty());
    let login_attempt_id = login_response_body.login_attempt_id;

    // Get the 2FA code and then immediately drop the read lock
    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();
    let two_fa_code = code_tuple.1.as_ref();

    let incorrect_email = get_random_email();
    let incorrect_login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let incorrect_two_fa_code = TwoFACode::default().as_ref().to_owned();
    let test_cases = vec![
        (
            incorrect_email.as_str(),
            login_attempt_id.as_str(),
            two_fa_code,
        ),
        (
            random_email.as_str(),
            incorrect_login_attempt_id.as_str(),
            two_fa_code,
        ),
        (
            random_email.as_str(),
            login_attempt_id.as_str(),
            incorrect_two_fa_code.as_ref(),
        ),
    ];
    // Assert prerequisities
    assert_eq!(signup_response.status().as_u16(), 201);
    assert_eq!(login_response_body.message, "2FA required".to_owned());

    // Act & Assert
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

    // Assert
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    // First login call

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

    // Get the code and then drop the read lock
    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();
    let two_fa_code = code_tuple.1.as_ref();

    // Second login call

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    // 2FA attempt with old login_attempt_id and code

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code
    });

    let response = app.post_verify_2fa(&request_body).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
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
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);
    let login_response_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");
    assert_eq!(login_response_body.message, "2FA required".to_owned());
    assert!(!login_response_body.login_attempt_id.is_empty());
    let login_attempt_id = login_response_body.login_attempt_id;

    // Get the code and then drop the read lock
    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();
    let two_fa_code = code_tuple.1.as_ref();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code
    });

    let response = app.post_verify_2fa(&request_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let response = app.post_verify_2fa(&request_body).await;

    assert_eq!(response.status().as_u16(), 401);
}

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
#[tokio::test]
async fn should_return_422_if_malformed_input(#[case] test_case: serde_json::Value) {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_verify_2fa(&test_case).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}
