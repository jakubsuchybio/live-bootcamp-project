use auth_service::{Email, JWT_COOKIE_NAME, TwoFactorAuthResponse};
use reqwest::Url;

use crate::helpers_harness::{get_random_email, TestApp};

/// Represents a test user's credentials
#[derive(Clone)]
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool,
}

impl TestUser {
    /// Create a new test user with default values
    pub fn new() -> Self {
        Self {
            email: get_random_email(),
            password: "password123".to_string(),
            requires_2fa: false,
        }
    }

    /// Create a new test user with 2FA enabled
    pub fn new_with_2fa() -> Self {
        Self {
            email: get_random_email(),
            password: "password123".to_string(),
            requires_2fa: true,
        }
    }

    /// Create a user with a specific email
    pub fn with_email(email: &str) -> Self {
        Self {
            email: email.to_string(),
            password: "password123".to_string(),
            requires_2fa: false,
        }
    }

    /// Customize a test user with specific attributes
    pub fn with_attributes(email: Option<&str>, password: Option<&str>, requires_2fa: bool) -> Self {
        Self {
            email: email.map(String::from).unwrap_or_else(get_random_email),
            password: password.map(String::from).unwrap_or_else(|| "password123".to_string()),
            requires_2fa,
        }
    }
    
    /// Get a signup JSON payload for this user
    pub fn signup_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "email": self.email,
            "password": self.password,
            "requires2FA": self.requires_2fa
        })
    }
    
    /// Get a login JSON payload for this user
    pub fn login_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "email": self.email,
            "password": self.password,
        })
    }
}

/// Struct to hold 2FA verification data
#[derive(Clone)]
pub struct TwoFAData {
    pub login_attempt_id: String,
    pub two_fa_code: String,
}

/// Register a user and assert success
/// (Use this in the arrange phase only, not act)
pub async fn setup_registered_user(app: &TestApp, user: &TestUser) -> TestUser {
    let response = app
        .post_signup(&user.signup_payload())
        .await;
    
    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed to register user with email: {}",
        user.email
    );
    
    user.clone()
}

/// Setup a user with standard login (no 2FA)
/// (Use this in the arrange phase only, not act)
pub async fn setup_logged_in_user(app: &TestApp) -> (TestUser, String) {
    let user = TestUser::new();
    
    // Register the user
    let registered_user = setup_registered_user(app, &user).await;
    
    // Log in the user
    let login_response = app
        .post_login(&registered_user.login_payload())
        .await;
    
    assert_eq!(login_response.status().as_u16(), 200, "Login failed");
    
    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    
    (registered_user, auth_cookie.value().to_string())
}

/// Setup a user with 2FA and start the login process
/// (Use this in the arrange phase only, not act)
pub async fn setup_2fa_login_started(app: &TestApp) -> (TestUser, TwoFAData) {
    let user = TestUser::new_with_2fa();
    
    // Register the user
    let registered_user = setup_registered_user(app, &user).await;
    
    // Start the login process
    let login_response = app
        .post_login(&registered_user.login_payload())
        .await;
    
    assert_eq!(login_response.status().as_u16(), 206, "Expected 206 for 2FA login");
    
    let response_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");
    
    assert_eq!(response_body.message, "2FA required".to_owned());
    
    // Get the 2FA code
    let (login_attempt_id, two_fa_code) = get_2fa_code_tuple(app, &registered_user.email).await;
    
    // Verify login_attempt_id matches what's in the response
    assert_eq!(login_attempt_id, response_body.login_attempt_id);
    
    let two_fa_data = TwoFAData {
        login_attempt_id,
        two_fa_code,
    };
    
    (registered_user, two_fa_data)
}

/// Get the 2FA code tuple for a user
/// (Use this in the arrange phase only, not act)
pub async fn get_2fa_code_tuple(app: &TestApp, email: &str) -> (String, String) {
    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(email).unwrap())
        .await
        .expect("Failed to get 2FA code");
    
    (
        code_tuple.0.as_ref().to_owned(), // login_attempt_id
        code_tuple.1.as_ref().to_owned()  // 2fa_code
    )
}

/// Create 2FA verification JSON payload
pub fn create_2fa_payload(email: &str, data: &TwoFAData) -> serde_json::Value {
    serde_json::json!({
        "email": email,
        "loginAttemptId": data.login_attempt_id,
        "2FACode": data.two_fa_code
    })
}

/// Add an arbitrary token to the cookie jar
pub fn add_token_to_cookie_jar(app: &TestApp, token: &str) {
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME, token
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );
}