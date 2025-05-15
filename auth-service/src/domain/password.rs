use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Password(String);

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password is too short")]
    TooShort,
}

impl Password {
    pub fn parse(password: &str) -> Result<Self, PasswordError> {
        if password.len() < 8 {
            return Err(PasswordError::TooShort);
        }

        Ok(Password(password.to_string()))
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use rstest::rstest;

    #[test]
    fn valid_password_can_be_created() {
        // Arrange
        let valid_password = "password123";

        // Act
        let result = Password::parse(valid_password);

        // Assert
        assert!(result.is_ok());
        let password = result.unwrap();
        assert_eq!(password.as_ref(), valid_password);
    }

    #[rstest]
    #[case("1234567", PasswordError::TooShort)] // 7 chars
    #[case("", PasswordError::TooShort)] // Empty
    #[case("123", PasswordError::TooShort)] // 3 chars
    fn invalid_passwords_return_errors(#[case] input: &str, #[case] expected_error: PasswordError) {
        // Act
        let result = Password::parse(input);

        // Assert
        assert!(result.is_err());

        // Convert to a variant we can match against without moving
        match result.unwrap_err() {
            PasswordError::TooShort => {
                assert!(
                    matches!(expected_error, PasswordError::TooShort),
                    "Expected TooShort error for password: {}",
                    input
                );
            }
        }
    }

    #[rstest]
    #[case("12345678")] // Exactly 8 chars
    #[case("password123")] // Common password
    #[case("abcdefghijklmno")] // 15 chars
    #[case("P@ssw0rd!2023")] // Complex password
    fn test_as_ref_returns_original_password(#[case] password_str: &str) {
        // Act
        let password = Password::parse(password_str).unwrap();

        // Assert
        assert_eq!(password.as_ref(), password_str);
    }

    // Property-based test for valid passwords (length >= 8)
    #[quickcheck]
    fn prop_valid_password_creates_password(password: String) -> TestResult {
        if password.len() < 8 {
            return TestResult::discard();
        }

        match Password::parse(&password) {
            Ok(pwd) => TestResult::from_bool(pwd.as_ref() == password),
            Err(_) => TestResult::failed(),
        }
    }

    // Property-based test for invalid passwords (length < 8)
    #[quickcheck]
    fn prop_short_password_returns_error(password: String) -> TestResult {
        if password.len() >= 8 {
            return TestResult::discard();
        }

        match Password::parse(&password) {
            Ok(_) => TestResult::failed(),
            Err(e) => TestResult::from_bool(matches!(e, PasswordError::TooShort)),
        }
    }
}
