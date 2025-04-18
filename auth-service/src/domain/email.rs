pub struct Email(String);

#[derive(Debug)]
pub enum EmailError {
    InvalidEmail,
}

impl Email {
    pub fn parse(address: &str) -> Result<Self, EmailError> {
        if !validator::validate_email(address) {
            return Err(EmailError::InvalidEmail);
        }

        Ok(Email(address.to_string()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use rstest::rstest;

    #[test]
    fn test_parse_valid_email() {
        // Arrange - Use fake to generate valid emails
        let email_address: String = SafeEmail().fake();

        // Act
        let result = Email::parse(email_address.as_str());

        // Assert
        assert!(result.is_ok());
        let email = result.unwrap();
        assert_eq!(email.as_ref(), email_address);
    }

    #[rstest]
    #[case::empty("".to_string())]
    #[case::not_an_email("notanemail".to_string())]
    #[case::missing_local("@missing-local.com".to_string())]
    #[case::missing_domain("missing-domain@".to_string())]
    #[case::spaces_in_email("spaces in@email.com".to_string())]
    fn test_parse_invalid_email(#[case] invalid_email: String) {
        // Act
        let result = Email::parse(invalid_email.as_str());

        // Assert
        assert!(
            result.is_err(),
            "Expected error for invalid email: {}",
            invalid_email
        );
    }

    #[rstest]
    #[case("user@example.com")]
    #[case("test.email@domain.com")]
    #[case("user123@company.co.uk")]
    fn test_as_ref_returns_original_string(#[case] email_str: &str) {
        // Act
        let email = Email::parse(email_str).unwrap();
        
        // Assert
        assert_eq!(email.as_ref(), email_str);
    }

    // Property-based test using quickcheck
    #[quickcheck]
    fn prop_valid_email_as_ref_returns_original(email_str: String) -> TestResult {
        // Skip empty strings or obviously invalid emails
        if email_str.is_empty() || !email_str.contains('@') {
            return TestResult::discard();
        }

        // If Email::parse succeeds, verify as_ref returns the original string
        match Email::parse(email_str.as_str()) {
            Ok(email) => TestResult::from_bool(email.as_ref() == email_str),
            Err(_) => TestResult::discard(), // Skip invalid emails
        }
    }

    // Use fake to generate multiple valid emails
    #[test]
    fn test_fake_email_generation_and_parsing() {
        for _ in 0..50 {
            // Arrange - Generate a fake email
            let email_str: String = SafeEmail().fake();

            // Act
            let email = Email::parse(email_str.as_str());

            // Assert
            assert!(email.is_ok(), "Failed to parse fake email: {}", email_str);
            assert_eq!(email.unwrap().as_ref(), email_str);
        }
    }
}
