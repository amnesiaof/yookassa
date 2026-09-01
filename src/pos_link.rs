//! POS links: offline QR/NFC payment links (`/pos_links`).

use crate::client::Client;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// A POS link (`PosLinkInfo`), the response of every POS link operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PosLink {
    /// `pl-...`, 36вЂ“50 chars.
    pub id: String,
    /// `active` or `inactive`.
    pub status: String,
    /// Always `nspk`.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    /// Shop location the tablet is bound to.
    #[serde(default)]
    pub recipient: Option<PosLinkRecipient>,
    /// Last payment made through this link, if any.
    #[serde(default)]
    pub payment: Option<PosLinkLastPayment>,
}

impl PosLink {
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}

/// `recipient` in a POS link response (`gateway_id` required by the API).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PosLinkRecipient {
    #[serde(default)]
    pub gateway_id: Option<String>,
}

/// The last payment made through a POS link.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PosLinkLastPayment {
    pub id: String,
    /// `pending`, `waiting_for_capture`, `succeeded` or `canceled`.
    pub status: String,
}

/// Request for creating a POS link (`POST /pos_links`).
#[derive(Debug, Clone, Serialize)]
pub struct CreatePosLinkRequest {
    pub recipient: crate::payment::RecipientData,
    pub pos_link_data: PosLinkData,
}

/// Data of the tablet QR code.
#[derive(Debug, Clone, Serialize)]
pub struct PosLinkData {
    /// URL scanned from the tablet, e.g. `https://qr.nspk.ru/...` (10вЂ“255 chars).
    pub link: String,
}

/// Request for rebinding a POS link to another shop
/// (`POST /pos_links/{pos_link_id}/recipient`).
#[derive(Debug, Clone, Serialize)]
pub struct ChangePosLinkRecipientRequest {
    pub recipient: crate::payment::RecipientData,
}

impl Client {
    /// Create a POS link (`POST /pos_links`).
    pub async fn create_pos_link(
        &self,
        request: &CreatePosLinkRequest,
        idempotency_key: Option<&str>,
    ) -> Result<PosLink> {
        let value = self.post("pos_links", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a POS link (`GET /pos_links/{pos_link_id}`).
    pub async fn get_pos_link(&self, pos_link_id: &str) -> Result<PosLink> {
        let value = self.get(&format!("pos_links/{pos_link_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Rebind a POS link to another shop
    /// (`POST /pos_links/{pos_link_id}/recipient`).
    pub async fn change_pos_link_recipient(
        &self,
        pos_link_id: &str,
        request: &ChangePosLinkRecipientRequest,
        idempotency_key: Option<&str>,
    ) -> Result<PosLink> {
        let value = self
            .post(
                &format!("pos_links/{pos_link_id}/recipient"),
                request,
                idempotency_key,
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Activate a deactivated POS link
    /// (`POST /pos_links/{pos_link_id}/activate`).
    pub async fn activate_pos_link(&self, pos_link_id: &str) -> Result<PosLink> {
        let value = self
            .post(
                &format!("pos_links/{pos_link_id}/activate"),
                &serde_json::json!({}),
                None,
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Deactivate a POS link (`POST /pos_links/{pos_link_id}/deactivate`).
    pub async fn deactivate_pos_link(&self, pos_link_id: &str) -> Result<PosLink> {
        let value = self
            .post(
                &format!("pos_links/{pos_link_id}/deactivate"),
                &serde_json::json!({}),
                None,
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }
}
