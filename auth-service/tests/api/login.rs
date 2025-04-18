use crate::helpers::TestApp;

#[tokio::test]
async fn login_returns_200() {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.login("test@example.com", "password").await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}
