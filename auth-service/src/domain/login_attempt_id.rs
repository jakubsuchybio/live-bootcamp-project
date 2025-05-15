use color_eyre::eyre::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: &str) -> Result<Self> {
        let id = Uuid::parse_str(id).wrap_err("Invalid login attempt id")?;
        Ok(LoginAttemptId(id.to_string()))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_attempt_id_parse_valid_uuid() {
        // Arrange
        let valid_uuid = Uuid::new_v4().to_string();

        // Act
        let result = LoginAttemptId::parse(&valid_uuid);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, valid_uuid);
    }

    #[test]
    fn test_login_attempt_id_parse_invalid_uuid() {
        // Arrange
        let invalid_uuid = "not-a-uuid";

        // Act
        let result = LoginAttemptId::parse(invalid_uuid);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_login_attempt_id_default() {
        // Arrange & Act
        let id = LoginAttemptId::default();

        // Assert
        assert!(Uuid::parse_str(&id.0).is_ok());
    }

    #[test]
    fn test_login_attempt_id_as_ref() {
        // Arrange
        let uuid_str = Uuid::new_v4().to_string();
        let id = LoginAttemptId(uuid_str.clone());

        // Act
        let result = id.as_ref();

        // Assert
        assert_eq!(result, uuid_str);
    }
}
