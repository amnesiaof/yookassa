//! Payouts: models and the `Payout` facade methods.

use crate::amount::MoneyAmount;
use crate::client::Client;
use crate::error::Result;
use crate::payment::Metadata;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// Destination of a payout (response, polymorphic by `type`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum PayoutDestination {
    #[serde(rename = "bank_card")]
    BankCard { card: PayoutCardData },
    #[serde(rename = "yoo_money")]
    YooMoney { account_number: String },
    #[serde(rename = "sbp")]
    Sbp {
        phone: String,
        bank_id: String,
        #[serde(default)]
        sbp_operation_id: Option<String>,
        recipient_checked: bool,
    },
}

/// Bank card data of a payout destination.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PayoutCardData {
    pub first6: String,
    pub last4: String,
    pub card_type: Option<String>,
    #[serde(default)]
    pub issuer_country: Option<String>,
    #[serde(default)]
    pub issuer_name: Option<String>,
}

/// A payout object (`Payout` in the API).
#[derive(Debug, Clone, Deserialize)]
pub struct Payout {
    pub id: String,
    pub amount: MoneyAmount,
    /// `pending`, `succeeded` or `canceled`.
    pub status: String,
    pub payout_destination: PayoutDestination,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    #[serde(default)]
    pub succeeded_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub deal: Option<PayoutDealInfo>,
    #[serde(default)]
    pub self_employed: Option<PayoutSelfEmployedInfo>,
    /// Deprecated self-employed income receipt.
    #[serde(default)]
    pub receipt: Option<IncomeReceipt>,
    #[serde(default)]
    pub cancellation_details: Option<PayoutCancellationDetails>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
    pub test: bool,
}

impl Payout {
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

/// Deal reference inside a payout (request and response).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PayoutDealInfo {
    pub id: String,
}

/// Self-employed info inside a payout.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PayoutSelfEmployedInfo {
    /// `se-…` id.
    pub id: String,
}

/// Deprecated self-employed income receipt attached to a payout.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IncomeReceipt {
    pub service_name: String,
    #[serde(default)]
    pub npd_receipt_id: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub amount: Option<MoneyAmount>,
}

/// Reason for the cancellation of a payout.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PayoutCancellationDetails {
    /// `yoo_money` or `payout_network`.
    #[serde(default)]
    pub party: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Destination data for creating a payout (request, polymorphic by `type`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum PayoutDestinationData {
    #[serde(rename = "bank_card")]
    BankCard { card: CardDataForPayout },
    #[serde(rename = "yoo_money")]
    YooMoney { account_number: String },
    #[serde(rename = "sbp")]
    Sbp { phone: String, bank_id: String },
}

/// Card data for a bank-card payout: only the number.
#[derive(Debug, Clone, Serialize)]
pub struct CardDataForPayout {
    pub number: String,
}

/// Personal data reference (`pd-…`), 1–2 per payout.
#[derive(Debug, Clone, Serialize)]
pub struct PersonalDataRef {
    pub id: String,
}

/// Request for creating a payout (`POST /payouts`).
///
/// Exactly one of `payout_destination_data`, `payout_token` or
/// `payment_method_id` must be set (validated by the API).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreatePayoutRequest {
    pub amount: MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payout_destination_data: Option<PayoutDestinationData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payout_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal: Option<PayoutDealInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_data: Option<Vec<PersonalDataRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Query for listing payouts (`GET /payouts`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct PayoutListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_gt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_lt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub succeeded_at_gte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub succeeded_at_gt: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub succeeded_at_lte: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub succeeded_at_lt: Option<DateTime<Utc>>,
    /// `bank_card`, `yoo_money` or `sbp`.
    #[serde(
        rename = "payout_destination.type",
        skip_serializing_if = "Option::is_none"
    )]
    pub payout_destination_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Query for searching payouts (`GET /payouts/search`).
#[derive(Debug, Clone, Default)]
pub struct PayoutSearchQuery {
    pub created_at_gte: Option<DateTime<Utc>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lte: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    /// At most one metadata key (`metadata[key]=value`).
    pub metadata: Option<(String, String)>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl PayoutSearchQuery {
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let mut push_dt = |k: &str, v: &Option<DateTime<Utc>>| {
            if let Some(v) = v {
                pairs.push((
                    k.to_string(),
                    v.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                ));
            }
        };
        push_dt("created_at.gte", &self.created_at_gte);
        push_dt("created_at.gt", &self.created_at_gt);
        push_dt("created_at.lte", &self.created_at_lte);
        push_dt("created_at.lt", &self.created_at_lt);
        if let Some((k, v)) = &self.metadata {
            pairs.push((format!("metadata[{k}]"), v.clone()));
        }
        if let Some(limit) = self.limit {
            pairs.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(cursor) = &self.cursor {
            pairs.push(("cursor".to_string(), cursor.clone()));
        }
        pairs
    }
}

/// Page of payouts (`PayoutsList`).
#[derive(Debug, Clone, Deserialize)]
pub struct PayoutList {
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    pub items: Vec<Payout>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

impl Client {
    /// Create a payout (`POST /payouts`).
    pub async fn create_payout(
        &self,
        request: &CreatePayoutRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Payout> {
        let value = self.post("payouts", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a payout by id (`GET /payouts/{payout_id}`).
    pub async fn get_payout(&self, payout_id: &str) -> Result<Payout> {
        let value = self.get(&format!("payouts/{payout_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List payouts (`GET /payouts`).
    pub async fn list_payouts(&self, query: &PayoutListQuery) -> Result<PayoutList> {
        let value = self
            .get("payouts", &crate::client::to_query(query)?)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Search payouts (`GET /payouts/search`).
    pub async fn search_payouts(&self, query: &PayoutSearchQuery) -> Result<PayoutList> {
        let value = self.get("payouts/search", &query.to_pairs()).await?;
        Ok(serde_json::from_value(value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_pairs() {
        let query = PayoutSearchQuery {
            created_at_gte: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:00:00Z")
                    .unwrap()
                    .into(),
            ),
            metadata: Some(("order".to_string(), "42".to_string())),
            limit: Some(20),
            ..Default::default()
        };
        let pairs = query.to_pairs();
        assert_eq!(
            pairs,
            vec![
                (
                    "created_at.gte".to_string(),
                    "2024-01-15T10:00:00.000Z".to_string()
                ),
                ("metadata[order]".to_string(), "42".to_string()),
                ("limit".to_string(), "20".to_string()),
            ]
        );
    }
}
