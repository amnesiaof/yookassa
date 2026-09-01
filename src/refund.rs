//! Refunds: models and the `Refund` facade methods.

use crate::amount::MoneyAmount;
use crate::client::Client;
use crate::error::Result;
use crate::payment::{Metadata, ReceiptData, Settlement};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// A refund object (`Refund` in the API).
#[derive(Debug, Clone, Deserialize)]
pub struct Refund {
    pub id: String,
    pub payment_id: String,
    /// `pending`, `succeeded` or `canceled`.
    pub status: String,
    pub amount: MoneyAmount,
    pub created_at: DateTime<FixedOffset>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub cancellation_details: Option<RefundCancellationDetails>,
    /// `pending`, `succeeded` or `canceled`.
    #[serde(default)]
    pub receipt_registration: Option<String>,
    #[serde(default)]
    pub sources: Option<Vec<RefundSource>>,
    #[serde(default)]
    pub deal: Option<RefundDealInfo>,
    #[serde(default)]
    pub refund_method: Option<RefundMethod>,
    #[serde(default)]
    pub refund_authorization_details: Option<RefundAuthorizationDetails>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

impl Refund {
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }
    pub fn is_succeeded(&self) -> bool {
        self.status == "succeeded"
    }
    pub fn is_canceled(&self) -> bool {
        self.status == "canceled"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundCancellationDetails {
    /// `yoo_money` or `refund_network`.
    #[serde(default)]
    pub party: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// A source of a split refund.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundSource {
    pub account_id: String,
    pub amount: MoneyAmount,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_fee_amount: Option<MoneyAmount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundDealInfo {
    pub id: String,
    pub refund_settlements: Vec<Settlement>,
}

/// Deal settlements of a refund (request side).
#[derive(Debug, Clone, Serialize)]
pub struct RefundDealData {
    pub refund_settlements: Vec<Settlement>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundAuthorizationDetails {
    #[serde(default)]
    pub rrn: Option<String>,
}

/// Refund method (response, polymorphic by `type`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum RefundMethod {
    #[serde(rename = "sbp")]
    Sbp {
        #[serde(default)]
        sbp_operation_id: Option<String>,
    },
    #[serde(rename = "electronic_certificate")]
    ElectronicCertificate {
        #[serde(default)]
        electronic_certificate: Option<ElectronicCertificateRefundData>,
        #[serde(default)]
        articles: Option<Vec<ElectronicCertificateRefundArticle>>,
    },
}

/// Electronic certificate refund data (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElectronicCertificateRefundData {
    pub basket_id: String,
    pub amount: MoneyAmount,
}

/// An article of an electronic certificate refund.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElectronicCertificateRefundArticle {
    pub article_number: u32,
    pub payment_article_number: u32,
    pub tru_code: String,
    pub quantity: u32,
}

/// Refund method data (request, polymorphic by `type`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum RefundMethodData {
    #[serde(rename = "electronic_certificate")]
    ElectronicCertificate {
        #[serde(skip_serializing_if = "Option::is_none")]
        electronic_certificate: Option<ElectronicCertificateRefundData>,
        #[serde(skip_serializing_if = "Option::is_none")]
        articles: Option<Vec<ElectronicCertificateRefundArticle>>,
    },
}

/// Request for creating a refund (`POST /refunds`).
#[derive(Debug, Clone, Serialize)]
pub struct CreateRefundRequest {
    pub payment_id: String,
    pub amount: MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ReceiptData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<RefundSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal: Option<RefundDealData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_method_data: Option<RefundMethodData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Query for listing refunds (`GET /refunds`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct RefundListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Page of refunds (`RefundList`).
#[derive(Debug, Clone, Deserialize)]
pub struct RefundList {
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    pub items: Vec<Refund>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

impl Client {
    /// Create a refund (`POST /refunds`).
    pub async fn create_refund(
        &self,
        request: &CreateRefundRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Refund> {
        let value = self.post("refunds", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a refund by id (`GET /refunds/{refund_id}`).
    pub async fn get_refund(&self, refund_id: &str) -> Result<Refund> {
        let value = self.get(&format!("refunds/{refund_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List refunds (`GET /refunds`).
    pub async fn list_refunds(&self, query: &RefundListQuery) -> Result<RefundList> {
        let value = self
            .get("refunds", &crate::client::to_query(query)?)
            .await?;
        Ok(serde_json::from_value(value)?)
    }
}
