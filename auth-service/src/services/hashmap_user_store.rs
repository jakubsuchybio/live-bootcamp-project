use crate::domain::User;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.get_user(email) {
            Ok(user) => {
                if user.password == password {
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

    #[test]
    fn test_add_user_ok() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );

        // Act
        let result = store.add_user(user);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_add_user_already_exists() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).unwrap();

        // Act
        let result = store.add_user(user);

        // Assert
        assert_eq!(result, Err(UserStoreError::UserAlreadyExists));
    }

    #[test]
    fn test_get_user_ok() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).unwrap();

        // Act
        let result = store.get_user(&user.email);

        // Assert
        assert_eq!(result, Ok(user));
    }

    #[test]
    fn test_get_user_not_found() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).unwrap();

        // Act
        let result = store.get_user("not_found@example.com");

        // Assert
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[test]
    fn test_validate_user_ok() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).unwrap();

        // Act
        let result = store.validate_user(&user.email, &user.password);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_validate_user_invalid_credentials() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).unwrap();

        // Act
        let result = store.validate_user(&user.email, "wrong_password");

        // Assert
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }

    #[test]
    fn test_validate_user_not_found() {
        // Arrange
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password".to_string(),
            false,
        );
        store.add_user(user.clone()).unwrap();

        // Act
        let result = store.validate_user("not_found@example.com", &user.password);

        // Assert
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }
}
