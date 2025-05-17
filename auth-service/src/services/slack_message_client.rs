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

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::Secret;
    use tokio;

    #[tokio::test]
    async fn test_send_slack_message() {
        // This test is disabled by default as it requires a valid Slack webhook URL
        // To run it: SLACK_WEBHOOK_URL=your_webhook_url cargo test -- --ignored test_send_slack_message

        let webhook_url = std::env::var("SLACK_WEBHOOK")
            .expect("SLACK_WEBHOOK_URL environment variable must be set to run this test");

        let client = SlackMessageClient::new(&Secret::new(webhook_url));

        let recipient = Email::parse(Secret::new("test@example.com".to_string())).unwrap();

        let subject = "Test Subject";
        let content = "This is a test message from the SlackMessageClient test.";

        let result = client.send_email(&recipient, subject, content).await;

        assert!(result.is_ok(), "Failed to send Slack message: {:?}", result);
    }
}
