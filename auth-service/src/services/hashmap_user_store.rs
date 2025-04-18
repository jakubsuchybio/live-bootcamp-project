use crate::domain::{Email, Password, User, UserStore, UserStoreError};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.get_user(email).await {
            Ok(user) => {
                if user.password == *password {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            Err(err) => Err(err),
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::User;

    #[tokio::test]
    async fn test_add_user_ok() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );

        // Act
        let result = store.add_user(user).await;

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_add_user_already_exists() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).await.unwrap();

        // Act
        let result = store.add_user(user).await;

        // Assert
        assert_eq!(result, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user_ok() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).await.unwrap();

        // Act
        let result = store.get_user(&user.email).await;

        // Assert
        assert_eq!(result, Ok(user));
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).await.unwrap();

        // Act
        let result = store.get_user("not_found@example.com").await;

        // Assert
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user_ok() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).await.unwrap();

        // Act
        let result = store.validate_user(&user.email, &user.password).await;

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_validate_user_invalid_credentials() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).await.unwrap();

        // Act
        let result = store.validate_user(&user.email, "wrong_password").await;

        // Assert
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }

    #[tokio::test]
    async fn test_validate_user_not_found() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).await.unwrap();

        // Act
        let result = store
            .validate_user("not_found@example.com", &user.password)
            .await;

        // Assert
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }
}
