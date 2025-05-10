use db_test_macro::db_test;

use crate::helpers_arrange::{add_token_to_cookie_jar, setup_logged_in_user};
use crate::helpers_assert::assert_status;
use crate::helpers_harness::TestApp;

#[db_test]
async fn should_return_200_if_valid_jwt_cookie() {
    // Arrange
    let mut app = TestApp::new().await;
    let (_user, token) = setup_logged_in_user(&app).await;

    // Act
    let response = app.post_logout().await;

    // Assert
    assert_status(&response, 200, None);

    assert!(app
        .banned_token_store
        .read()
        .await
        .check_banned_token(&token)
        .await
        .unwrap());
}

#[db_test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    // Arrange
    let mut app = TestApp::new().await;
    let (_user, _token) = setup_logged_in_user(&app).await;

    // Act
    let logout_response1 = app.post_logout().await;
    let logout_response2 = app.post_logout().await;

    // Assert
    assert_status(&logout_response1, 200, None);
    assert_status(&logout_response2, 400, None);
}

#[db_test]
async fn should_return_400_if_jwt_cookie_missing() {
    // Arrange
    let mut app = TestApp::new().await;

    // Act
    let response = app.post_logout().await;

    // Assert
    assert_status(&response, 400, None);
}

#[db_test]
async fn should_return_401_if_invalid_token() {
    // Arrange
    let mut app = TestApp::new().await;
    add_token_to_cookie_jar(&app, "invalid");

    // Act
    let response = app.post_logout().await;

    // Assert
    assert_status(&response, 401, None);
}
