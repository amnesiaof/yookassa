//! Webhooks management (`/webhooks`, OAuth only).

use crate::client::Client;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Events a webhook can subscribe to (`NotificationEventType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WebhookEvent {
    PaymentWaitingForCapture,
    PaymentSucceeded,
    PaymentCanceled,
    RefundSucceeded,
    PaymentMethodActive,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PaymentWaitingForCapture => "payment.waiting_for_capture",
            Self::PaymentSucceeded => "payment.succeeded",
            Self::PaymentCanceled => "payment.canceled",
            Self::RefundSucceeded => "refund.succeeded",
            Self::PaymentMethodActive => "payment_method.active",
        }
    }
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for WebhookEvent {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "payment.waiting_for_capture" => Self::PaymentWaitingForCapture,
            "payment.succeeded" => Self::PaymentSucceeded,
            "payment.canceled" => Self::PaymentCanceled,
            "refund.succeeded" => Self::RefundSucceeded,
            "payment_method.active" => Self::PaymentMethodActive,
            _ => unreachable!(),
        })
    }
}

/// A registered webhook.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Webhook {
    /// `wh-...`.
    pub id: String,
    /// [`WebhookEvent::as_str`] value.
    pub event: String,
    pub url: String,
}

impl Webhook {
    /// Parse the `event` string into a [`WebhookEvent`].
    ///
    /// Unknown events (added to the API later) are returned as `Err`.
    pub fn event(&self) -> std::result::Result<WebhookEvent, String> {
        self.event
            .parse::<WebhookEvent>()
            .map_err(|_| self.event.clone())
    }
}

/// Page of webhooks.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookList {
    /// Always `list`.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub items: Vec<Webhook>,
}

/// Request for registering a webhook (`POST /webhooks`).
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    /// [`WebhookEvent::as_str`] value.
    pub event: String,
    pub url: String,
}

impl CreateWebhookRequest {
    pub fn new(event: WebhookEvent, url: impl Into<String>) -> Self {
        Self {
            event: event.as_str().to_string(),
            url: url.into(),
        }
    }
}

impl Client {
    /// Register a webhook (`POST /webhooks`).
    ///
    /// Webhooks are OAuth-only: calling with Basic auth returns
    /// [`Error::Configuration`].
    pub async fn create_webhook(
        &self,
        request: &CreateWebhookRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Webhook> {
        self.require_oauth()?;
        let value = self.post("webhooks", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List active webhooks (`GET /webhooks`).
    pub async fn list_webhooks(&self) -> Result<WebhookList> {
        self.require_oauth()?;
        let value = self.get("webhooks", &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Delete a webhook (`DELETE /webhooks/{webhook_id}`).
    pub async fn delete_webhook(
        &self,
        webhook_id: &str,
        idempotency_key: Option<&str>,
    ) -> Result<()> {
        self.require_oauth()?;
        let value = self
            .delete(&format!("webhooks/{webhook_id}"), idempotency_key)
            .await?;
        let _ = value;
        Ok(())
    }
}
