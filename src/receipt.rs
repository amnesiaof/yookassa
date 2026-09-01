//! Receipts (54-FZ fiscalization): models and the `Receipt` facade methods.

use crate::amount::MoneyAmount;
use crate::client::Client;
use crate::error::Result;
use crate::payment::{IndustryDetails, OperationalDetails, ReceiptCustomer, ReceiptItem};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// A receipt object (`Receipt` in the API).
#[derive(Debug, Clone, Deserialize)]
pub struct Receipt {
    pub id: String,
    /// `payment` or `refund`.
    #[serde(rename = "type")]
    pub type_: String,
    /// `pending`, `succeeded` or `canceled`.
    pub status: String,
    #[serde(default)]
    pub payment_id: Option<String>,
    #[serde(default)]
    pub refund_id: Option<String>,
    #[serde(default)]
    pub fiscal_document_number: Option<String>,
    #[serde(default)]
    pub fiscal_storage_number: Option<String>,
    #[serde(default)]
    pub fiscal_attribute: Option<String>,
    #[serde(default)]
    pub registered_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub fiscal_provider_id: Option<String>,
    pub items: Vec<ReceiptItem>,
    #[serde(default)]
    pub internet: Option<bool>,
    #[serde(default)]
    pub settlements: Option<Vec<ReceiptSettlement>>,
    #[serde(default)]
    pub on_behalf_of: Option<String>,
    #[serde(default)]
    pub tax_system_code: Option<u8>,
    #[serde(default)]
    pub timezone: Option<u8>,
    #[serde(default)]
    pub receipt_industry_details: Option<Vec<IndustryDetails>>,
    #[serde(default)]
    pub receipt_operational_details: Option<OperationalDetails>,
}

/// A settlement line of a receipt (`Settlement`, type `cashless` etc.).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReceiptSettlement {
    /// `cashless`, `prepayment`, `postpayment` or `consideration`.
    #[serde(rename = "type")]
    pub type_: String,
    pub amount: MoneyAmount,
}

/// Additional user props (54-FZ tag 1084).
#[derive(Debug, Clone, Serialize)]
pub struct AdditionalUserProps {
    pub name: String,
    pub value: String,
}

/// Request for registering a receipt (`POST /receipts`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateReceiptRequest {
    /// `payment` or `refund`.
    #[serde(rename = "type")]
    pub type_: String,
    /// Required for `type = payment`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<String>,
    /// Required for `type = refund`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_id: Option<String>,
    pub customer: ReceiptCustomer,
    pub items: Vec<ReceiptItem>,
    /// Whether to send the receipt to the customer (default `true`).
    pub send: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlements: Option<Vec<ReceiptSettlement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<String>,
    /// 1–6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_system_code: Option<u8>,
    /// 1–11.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_user_props: Option<AdditionalUserProps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_industry_details: Option<Vec<IndustryDetails>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_operational_details: Option<OperationalDetails>,
}

/// Query for listing receipts (`GET /receipts`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReceiptListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Mutually exclusive with `refund_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<String>,
    /// Mutually exclusive with `payment_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Page of receipts (`ReceiptList`).
#[derive(Debug, Clone, Deserialize)]
pub struct ReceiptList {
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    pub items: Vec<Receipt>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

impl Client {
    /// Register a receipt (`POST /receipts`).
    pub async fn create_receipt(
        &self,
        request: &CreateReceiptRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Receipt> {
        let value = self.post("receipts", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a receipt by id (`GET /receipts/{receipt_id}`).
    pub async fn get_receipt(&self, receipt_id: &str) -> Result<Receipt> {
        let value = self.get(&format!("receipts/{receipt_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List receipts (`GET /receipts`).
    pub async fn list_receipts(&self, query: &ReceiptListQuery) -> Result<ReceiptList> {
        let value = self
            .get("receipts", &crate::client::to_query(query)?)
            .await?;
        Ok(serde_json::from_value(value)?)
    }
}
