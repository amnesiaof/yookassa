//! Personal data of payout recipients (`/personal_data`).

use crate::client::Client;
use crate::error::Result;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// Personal data of a payout recipient (`PersonalData`).
#[derive(Debug, Clone, Deserialize)]
pub struct PersonalData {
    pub id: String,
    /// `payout_statement_recipient` or `sbp_payout_recipient`.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    /// `waiting_for_operation`, `active` or `canceled`.
    pub status: String,
    pub created_at: DateTime<FixedOffset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancellation_details: Option<PersonalDataCancellationDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<crate::payment::Metadata>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersonalDataCancellationDetails {
    /// Always `yoo_money`.
    #[serde(default)]
    pub party: Option<String>,
    /// Always `expired_by_timeout`.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Request for creating personal data (`POST /personal_data`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum PersonalDataRequest {
    #[serde(rename = "sbp_payout_recipient")]
    SbpPayoutRecipient {
        last_name: String,
        first_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        middle_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<crate::payment::Metadata>,
    },
    #[serde(rename = "payout_statement_recipient")]
    PayoutStatementRecipient {
        last_name: String,
        first_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        middle_name: Option<String>,
        /// `YYYY-MM-DD`.
        birthdate: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<crate::payment::Metadata>,
    },
}

impl Client {
    /// Create personal data (`POST /personal_data`).
    pub async fn create_personal_data(
        &self,
        request: &PersonalDataRequest,
        idempotency_key: Option<&str>,
    ) -> Result<PersonalData> {
        let value = self.post("personal_data", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get personal data (`GET /personal_data/{personal_data_id}`).
    pub async fn get_personal_data(&self, personal_data_id: &str) -> Result<PersonalData> {
        let value = self
            .get(&format!("personal_data/{personal_data_id}"), &[])
            .await?;
        Ok(serde_json::from_value(value)?)
    }
}
