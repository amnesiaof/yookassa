//! Saved payment methods: create and get (`/payment_methods`).

use crate::client::Client;
use crate::error::Result;
use crate::payment::Metadata;
use serde::{Deserialize, Serialize};

/// Confirmation of a saved payment method (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum SavedMethodConfirmation {
    #[serde(rename = "redirect")]
    Redirect {
        confirmation_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        return_url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enforce: Option<bool>,
    },
    #[serde(rename = "qr")]
    Qr {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmation_data: Option<String>,
    },
}

/// Confirmation of a saved payment method (request).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum SavedMethodConfirmationData {
    #[serde(rename = "redirect")]
    Redirect {
        return_url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enforce: Option<bool>,
        /// `ru_RU` or `en_US`.
        #[serde(skip_serializing_if = "Option::is_none")]
        locale: Option<String>,
    },
    #[serde(rename = "qr")]
    Qr {
        #[serde(skip_serializing_if = "Option::is_none")]
        return_url: Option<String>,
    },
}

/// Common fields of a saved payment method (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SavedPaymentMethod {
    pub id: String,
    pub saved: bool,
    /// `active`, `pending` or `inactive`.
    pub status: String,
    /// The shop the method is saved for.
    #[serde(default)]
    pub holder: Option<SavedMethodHolder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation: Option<SavedMethodConfirmation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(flatten)]
    pub data: SavedMethodData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SavedMethodHolder {
    pub account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_id: Option<String>,
}

/// Type-specific part of a saved payment method (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)] // card data is the dominant payload; boxing it hurts usability
pub enum SavedMethodData {
    #[serde(rename = "bank_card")]
    BankCard {
        #[serde(default)]
        card: Option<crate::payment::BankCardData>,
    },
    #[serde(rename = "sbp")]
    Sbp {
        #[serde(default)]
        payer_bank_details: Option<SbpPayerBank>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SbpPayerBank {
    pub bank_id: String,
}

/// Saved payment method data (request).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum SavedMethodDataRequest {
    #[serde(rename = "bank_card")]
    BankCard {
        card: crate::payment::CardData,
        /// The shop the method is saved for (OAuth/partner flows).
        #[serde(skip_serializing_if = "Option::is_none")]
        holder: Option<crate::payment::RecipientData>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_ip: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        confirmation: Option<SavedMethodConfirmationData>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<Metadata>,
    },
    #[serde(rename = "sbp")]
    Sbp {
        #[serde(skip_serializing_if = "Option::is_none")]
        holder: Option<crate::payment::RecipientData>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_ip: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        confirmation: Option<SavedMethodConfirmationData>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<Metadata>,
    },
}

impl Client {
    /// Create a saved payment method (`POST /payment_methods`).
    pub async fn create_payment_method(
        &self,
        request: &SavedMethodDataRequest,
        idempotency_key: Option<&str>,
    ) -> Result<SavedPaymentMethod> {
        let value = self
            .post("payment_methods", request, idempotency_key)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a saved payment method (`GET /payment_methods/{payment_method_id}`).
    pub async fn get_payment_method(&self, payment_method_id: &str) -> Result<SavedPaymentMethod> {
        let value = self
            .get(&format!("payment_methods/{payment_method_id}"), &[])
            .await?;
        Ok(serde_json::from_value(value)?)
    }
}
