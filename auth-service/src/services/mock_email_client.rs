use crate::domain::{Email, EmailClient};
use color_eyre::Result;
use secrecy::ExposeSecret;
use tracing::warn;

pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        // Our mock email client will simply log the recipient, subject, and content to standard output
        warn!(
            "Sending email to {} with subject: {} and content: {}",
            recipient.as_ref().expose_secret(),
            subject,
            content
        );

        Ok(())
    }
}
