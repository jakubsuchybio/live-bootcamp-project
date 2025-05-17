use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Default)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

impl HashMapTwoFACodeStore {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
        }
    }
}

#[async_trait]
impl TwoFACodeStore for HashMapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        // Remove the code entry and return an error if it doesn't exist
        if let Some(_) = self.codes.remove(email) {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use super::*;
    use crate::domain::{Email, LoginAttemptId, TwoFACode};

    #[tokio::test]
    async fn test_add_code_successfully() {
        // Arrange
        let mut store = HashMapTwoFACodeStore::default();
        let email = Email::parse(Secret::new("test@example.com".to_string())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse(&Secret::new("123456".to_string())).unwrap();

        // Act
        let result = store
            .add_code(email.clone(), login_attempt_id, code.clone())
            .await;

        // Assert
        assert!(result.is_ok());
        let stored = store.codes.get(&email).unwrap();
        assert_eq!(stored.1, code);
    }

    #[tokio::test]
    async fn test_remove_code_successfully() {
        // Arrange
        let mut store = HashMapTwoFACodeStore::default();
        let email = Email::parse(Secret::new("test@example.com".to_string())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse(&Secret::new("123456".to_string())).unwrap();
        let _ = store.add_code(email.clone(), login_attempt_id, code).await;

        // Act
        let result = store.remove_code(&email).await;

        // Assert
        assert!(result.is_ok());
        assert!(store.codes.get(&email).is_none());
    }

    #[tokio::test]
    async fn test_remove_code_returns_error_when_email_not_found() {
        // Arrange
        let mut store = HashMapTwoFACodeStore::default();
        let email = Email::parse(Secret::new("test@example.com".to_string())).unwrap();

        // Act
        let result = store.remove_code(&email).await;

        // Assert
        assert!(matches!(
            result,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        ));
    }

    #[tokio::test]
    async fn test_get_code_successfully() {
        // Arrange
        let mut store = HashMapTwoFACodeStore::default();
        let email = Email::parse(Secret::new("test@example.com".to_string())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse(&Secret::new("123456".to_string())).unwrap();
        let _ = store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await;

        // Act
        let result = store.get_code(&email).await;

        // Assert
        assert!(result.is_ok());
        let (retrieved_id, retrieved_code) = result.unwrap();
        assert_eq!(retrieved_id, login_attempt_id);
        assert_eq!(retrieved_code, code);
    }

    #[tokio::test]
    async fn test_get_code_returns_error_when_email_not_found() {
        // Arrange
        let store = HashMapTwoFACodeStore::default();
        let email = Email::parse(Secret::new("test@example.com".to_string())).unwrap();

        // Act
        let result = store.get_code(&email).await;

        // Assert
        assert!(matches!(
            result,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        ));
    }
}
