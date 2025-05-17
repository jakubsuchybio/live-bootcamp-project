use color_eyre::eyre::{eyre, Context, Result};
use rand::Rng;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug)]
pub struct TwoFACode(Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl TwoFACode {
    pub fn parse(code: &Secret<String>) -> Result<Self> {
        let code_as_u32 = code
            .expose_secret()
            .parse::<u32>()
            .wrap_err("Invalid 2FA code")?;
        if (100_000..=999_999).contains(&code_as_u32) {
            Ok(Self(code.to_owned()))
        } else {
            Err(eyre!("Invalid 2FA code"))
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let mut rng = rand::thread_rng();
        let code: String = (0..6).map(|_| rng.gen_range(0..10).to_string()).collect();
        TwoFACode(Secret::new(code))
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
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
        let result = TwoFACode::parse(&Secret::new(valid_code.to_string()));

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref().expose_secret(), "123456");
    }

    #[test]
    fn test_parse_invalid_length() {
        // Arrange
        let short_code = "12345";
        let long_code = "1234567";

        // Act
        let short_result = TwoFACode::parse(&Secret::new(short_code.to_string()));
        let long_result = TwoFACode::parse(&Secret::new(long_code.to_string()));

        // Assert
        assert!(short_result.is_err());
        assert!(long_result.is_err());
    }

    #[test]
    fn test_parse_non_digits() {
        // Arrange
        let invalid_code = "12345a";

        // Act
        let result = TwoFACode::parse(&Secret::new(invalid_code.to_string()));

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_default_generates_valid_code() {
        // Arrange & Act
        let code = TwoFACode::default();

        // Assert
        assert_eq!(code.as_ref().expose_secret().len(), 6);
        assert!(code
            .as_ref()
            .expose_secret()
            .chars()
            .all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_as_ref() {
        // Arrange
        let code = "987654";
        let parsed_code = TwoFACode::parse(&Secret::new(code.to_string())).unwrap();

        // Act
        let reference = parsed_code.as_ref();

        // Assert
        assert_eq!(reference.expose_secret(), code);
    }
}
