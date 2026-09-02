//! Deals (Safe Deal): models and the `Deal` facade methods.

use crate::amount::MoneyAmount;
use crate::client::Client;
use crate::error::Result;
use crate::payment::Metadata;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// A deal object (`SafeDeal` in the API).
#[derive(Debug, Clone, Deserialize)]
pub struct Deal {
    pub id: String,
    /// Always `safe_deal`.
    #[serde(rename = "type", default)]
    pub type_: String,
    /// `payment_succeeded` or `deal_closed`.
    #[serde(default)]
    pub fee_moment: Option<String>,
    /// `opened` or `closed`.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub balance: MoneyAmount,
    pub payout_balance: MoneyAmount,
    pub created_at: DateTime<FixedOffset>,
    pub expires_at: DateTime<FixedOffset>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
    pub test: bool,
}

/// Request for creating a deal (`POST /deals`).
#[derive(Debug, Clone, Serialize)]
pub struct CreateDealRequest {
    /// Always `safe_deal`.
    #[serde(rename = "type")]
    pub type_: String,
    /// `payment_succeeded` or `deal_closed`.
    pub fee_moment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

impl Default for CreateDealRequest {
    fn default() -> Self {
        Self {
            type_: "safe_deal".to_string(),
            fee_moment: "payment_succeeded".to_string(),
            description: None,
            metadata: None,
        }
    }
}

/// Query for listing deals (`GET /deals`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct DealListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_gte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_gt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_lte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_lt: Option<DateTime<Utc>>,
    /// `opened` or `closed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// 4–128 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_search: Option<String>,
    /// At most one metadata key (`metadata[key]=value`, deepObject).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Page of deals (`DealList`).
#[derive(Debug, Clone, Deserialize)]
pub struct DealList {
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    pub items: Vec<Deal>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

impl Client {
    /// Create a deal (`POST /deals`).
    pub async fn create_deal(
        &self,
        request: &CreateDealRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Deal> {
        let value = self.post("deals", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a deal by id (`GET /deals/{deal_id}`).
    pub async fn get_deal(&self, deal_id: &str) -> Result<Deal> {
        let value = self.get(&format!("deals/{deal_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List deals (`GET /deals`).
    pub async fn list_deals(&self, query: &DealListQuery) -> Result<DealList> {
        let mut pairs = crate::client::to_query(query)?;
        crate::client::push_metadata(&mut pairs, &query.metadata);
        let value = self.get("deals", &pairs).await?;
        Ok(serde_json::from_value(value)?)
    }
}
