use std::sync::Arc;

use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "BannedTokenStore", skip_all, err(Debug))]
    async fn add_banned_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let token_key = get_key(&token);
        let value = true;
        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("Failed to cast TOKEN_TTL_SECONDS to u64.")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex::<_, _, ()>(token_key, value, ttl)
            .wrap_err("Failed to set banned token in Redis.")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "BannedTokenStore", skip_all, err(Debug))]
    async fn check_banned_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let token_key = get_key(&token);

        let is_banned = self
            .conn
            .write()
            .await
            .exists::<_, bool>(token_key)
            .wrap_err("Failed to check if token exists in Redis.")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(is_banned)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
