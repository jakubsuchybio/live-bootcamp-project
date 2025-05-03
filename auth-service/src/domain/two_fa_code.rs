use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: &str) -> Result<Self, String> {
        // Check if the code is a 6-digit number
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err("2FA code must be a 6-digit number".to_string());
        }

        Ok(TwoFACode(code.to_string()))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let mut rng = rand::thread_rng();
        let code: String = (0..6).map(|_| rng.gen_range(0..10).to_string()).collect();
        TwoFACode(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_code() {
        // Arrange
        let valid_code = "123456";

        // Act
        let result = TwoFACode::parse(valid_code);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "123456");
    }

    #[test]
    fn test_parse_invalid_length() {
        // Arrange
        let short_code = "12345";
        let long_code = "1234567";

        // Act
        let short_result = TwoFACode::parse(short_code);
        let long_result = TwoFACode::parse(long_code);

        // Assert
        assert!(short_result.is_err());
        assert!(long_result.is_err());
    }

    #[test]
    fn test_parse_non_digits() {
        // Arrange
        let invalid_code = "12345a";

        // Act
        let result = TwoFACode::parse(invalid_code);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_default_generates_valid_code() {
        // Arrange & Act
        let code = TwoFACode::default();

        // Assert
        assert_eq!(code.as_ref().len(), 6);
        assert!(code.as_ref().chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_as_ref() {
        // Arrange
        let code = "987654";
        let parsed_code = TwoFACode::parse(code).unwrap();

        // Act
        let reference = parsed_code.as_ref();

        // Assert
        assert_eq!(reference, code);
    }
}
