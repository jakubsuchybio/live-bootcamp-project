use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    Email, {LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let code_key = get_key(&email);
        let tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );

        let serialized_tuple =
            serde_json::to_string(&tuple).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex::<String, String, ()>(code_key, serialized_tuple, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let code_key = get_key(email);

        self.conn
            .write()
            .await
            .del::<String, ()>(code_key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let code_key = get_key(email);

        let Ok(result) = self.conn.write().await.get::<String, String>(code_key) else {
            return Err(TwoFACodeStoreError::LoginAttemptIdNotFound);
        };

        let tuple: TwoFATuple =
            serde_json::from_str(&result).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let login_attempt_id = LoginAttemptId::parse(tuple.0.as_str())
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let two_fa_code =
            TwoFACode::parse(tuple.1.as_str()).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
