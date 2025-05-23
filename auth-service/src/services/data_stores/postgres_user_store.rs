use color_eyre::eyre::{eyre, Result};

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    domain::{Password, User, UserStore, UserStoreError},
    Email,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Check if a user with the same email already exists
        let existing_user = sqlx::query!(
            r#"
            SELECT email FROM users WHERE email = $1
            "#,
            user.email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        if existing_user.is_some() {
            return Err(UserStoreError::UserAlreadyExists);
        }

        let password_hash = compute_password_hash_async(user.password.as_ref().to_owned())
            .await
            .map_err(UserStoreError::UnexpectedError)?;

        sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.email.as_ref().expose_secret(),
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        sqlx::query!(
            r#"
            SELECT email, password_hash, requires_2fa
            FROM users
            WHERE email = $1
            "#,
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
        .map(|row| {
            Ok(User {
                email: Email::parse(Secret::new(row.email))
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                password: Password::parse(Secret::new(row.password_hash))
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                requires_2fa: row.requires_2fa,
            })
        })
        .ok_or(UserStoreError::UserNotFound)?
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let password_hash = sqlx::query!(
            r#"
            SELECT password_hash
            FROM users
            WHERE email = $1
            "#,
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
        .map(|row| Secret::new(row.password_hash))
        .ok_or(UserStoreError::UserNotFound)?;

        if verify_password_hash(password_hash, password.as_ref().to_owned())
            .await
            .is_ok()
        {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }
}

// Helper function to verify if a given password matches an expected hash
#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> =
            PasswordHash::new(expected_password_hash.expose_secret())?;

        Argon2::default()
            .verify_password(
                password_candidate.expose_secret().as_bytes(),
                &expected_password_hash,
            )
            .map_err(|e| e.into())
    })
    .await?
}

// Helper function to hash passwords before persisting them in the database.
// Performs hashing on a separate thread pool to avoid blocking the async runtime
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash_async(password: Secret<String>) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();

        Ok(password_hash)
    })
    .await?
}
