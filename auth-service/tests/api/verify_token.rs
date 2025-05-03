use crate::helpers_arrange::setup_logged_in_user;
use crate::helpers_assert::assert_status;
use crate::helpers_harness::TestApp;
use rstest::rstest;

#[tokio::test]
async fn should_return_200_valid_token() {
    // Arrange
    let app = TestApp::new().await;
    let (_user, token) = setup_logged_in_user(&app).await;

    // Act
    let response = app
        .post_verify_token(&serde_json::json!({
            "token": token,
        }))
        .await;

    // Assert
    assert_status(&response, 200, None);
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
    assert_status(&response, 402, None);
}

#[tokio::test]
async fn should_return_401_if_token_is_banned() {
    // Arrange
    let app = TestApp::new().await;
    let (_user, token) = setup_logged_in_user(&app).await;

    // Ban token by logging out
    let logout_response = app.post_logout().await;
    assert_status(&logout_response, 200, None);

    // Act
    let response = app
        .post_verify_token(&serde_json::json!({
            "token": token,
        }))
        .await;

    // Assert
    let banned_token_store = app.banned_token_store.read().await;
    assert!(banned_token_store.check_banned_token(&token).await);
    assert_status(&response, 402, None);
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
    assert_status(
        &response,
        422,
        Some(&format!("Failed for input: {:?}", test_case)),
    );
}
