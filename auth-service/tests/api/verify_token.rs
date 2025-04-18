use crate::helpers::TestApp;

#[tokio::test]
async fn verify_token_returns_200() {
    // Arrange
    let app = TestApp::new().await;

    // Act
    let response = app.verify_token("123456").await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}
