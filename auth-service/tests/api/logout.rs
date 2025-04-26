use crate::helpers::TestApp;

#[tokio::test]
async fn logout_returns_200() {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.post_logout().await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}
