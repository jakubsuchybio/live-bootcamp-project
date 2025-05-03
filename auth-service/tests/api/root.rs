use crate::helpers_assert::assert_status;
use crate::helpers_harness::TestApp;

#[tokio::test]
async fn root_returns_auth_ui() {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.get_root().await;

    // Assert
    assert_status(&response, 200, None);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
}