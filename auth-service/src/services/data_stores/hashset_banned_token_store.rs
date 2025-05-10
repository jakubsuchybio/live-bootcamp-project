use std::collections::HashSet;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashSetBannedTokenStore {
    banned_tokens: HashSet<String>,
}

impl HashSetBannedTokenStore {
    pub fn new() -> Self {
        Self {
            banned_tokens: HashSet::new(),
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn add_banned_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        self.banned_tokens.insert(token);
        Ok(())
    }

    async fn check_banned_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.banned_tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_check_banned_token() {
        // Arrange
        let mut store = HashSetBannedTokenStore {
            banned_tokens: HashSet::new(),
        };
        let token = "banned_token".to_string();

        // Act
        store.add_banned_token(token.clone()).await.unwrap();
        let is_banned = store.check_banned_token(&token).await.unwrap();

        // Assert
        assert!(is_banned);
    }

    #[tokio::test]
    async fn test_check_non_existent_token() {
        // Arrange
        let store = HashSetBannedTokenStore {
            banned_tokens: HashSet::new(),
        };
        let token = "not_banned_token".to_string();

        // Act
        let is_banned = store.check_banned_token(&token).await.unwrap();

        // Assert
        assert!(!is_banned);
    }

    #[tokio::test]
    async fn test_multiple_tokens() {
        // Arrange
        let mut store = HashSetBannedTokenStore {
            banned_tokens: HashSet::new(),
        };
        let token1 = "banned_token1".to_string();
        let token2 = "banned_token2".to_string();
        let token3 = "not_banned_token".to_string();

        // Act
        store.add_banned_token(token1.clone()).await.unwrap();
        store.add_banned_token(token2.clone()).await.unwrap();

        let is_token1_banned = store.check_banned_token(&token1).await.unwrap();
        let is_token2_banned = store.check_banned_token(&token2).await.unwrap();
        let is_token3_banned = store.check_banned_token(&token3).await.unwrap();

        // Assert
        assert!(is_token1_banned);
        assert!(is_token2_banned);
        assert!(!is_token3_banned);
    }
}
