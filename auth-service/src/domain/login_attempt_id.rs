use color_eyre::eyre::{Context, Result};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LoginAttemptId(Secret<String>);

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl LoginAttemptId {
    pub fn parse(id: &Secret<String>) -> Result<Self> {
        let id = Uuid::parse_str(id.expose_secret()).wrap_err("Invalid login attempt id")?;
        Ok(LoginAttemptId(Secret::new(id.to_string())))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Secret::new(Uuid::new_v4().to_string()))
    }
}

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_attempt_id_parse_valid_uuid() {
        // Arrange
        let valid_uuid = Secret::new(Uuid::new_v4().to_string());

        // Act
        let result = LoginAttemptId::parse(&valid_uuid);

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().0.expose_secret(),
            valid_uuid.expose_secret()
        );
    }

    #[test]
    fn test_login_attempt_id_parse_invalid_uuid() {
        // Arrange
        let invalid_uuid = Secret::new("not-a-uuid".to_string());

        // Act
        let result = LoginAttemptId::parse(&invalid_uuid);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_login_attempt_id_default() {
        // Arrange & Act
        let id = LoginAttemptId::default();

        // Assert
        assert!(Uuid::parse_str(&id.0.expose_secret()).is_ok());
    }

    #[test]
    fn test_login_attempt_id_as_ref() {
        // Arrange
        let uuid_str = Secret::new(Uuid::new_v4().to_string());
        let id = LoginAttemptId(uuid_str.clone());

        // Act
        let result = id.as_ref();

        // Assert
        assert_eq!(result.expose_secret(), uuid_str.expose_secret());
    }
}
