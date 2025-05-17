use crate::domain::{Email, EmailClient};
use color_eyre::Result;
use secrecy::{ExposeSecret, Secret};
use tracing::warn;

pub struct SlackMessageClient {
    webhook_url: Secret<String>,
}

impl SlackMessageClient {
    pub fn new(webhook_url: &Secret<String>) -> Self {
        Self {
            webhook_url: webhook_url.clone(),
        }
    }
}

#[async_trait::async_trait]
impl EmailClient for SlackMessageClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        let payload = serde_json::json!({
            "text": format!(
                "Email to: {}\nSubject: {}\n\n{}",
                recipient.as_ref().expose_secret(),
                subject,
                content
            )
        });

        // Use reqwest to make a POST request to the Slack webhook
        let client = reqwest::Client::new();

        let response = client
            .post(&self.webhook_url.expose_secret().to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .json(&payload)
            .send()
            .await?;

        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            warn!(
                "Failed to send Slack message: Status {}, Error: {}",
                status, error_text
            );
            return Err(color_eyre::eyre::eyre!(
                "Slack API error: {} - {}",
                status,
                error_text
            ));
        }

        Ok(())
    }
}
