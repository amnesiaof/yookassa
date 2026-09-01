//! SBP participant banks (`GET /sbp_banks`).

use crate::client::Client;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// A bank or payment service connected to SBP.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SbpParticipantBank {
    /// 12 alphanumeric characters, e.g. `100000000111`.
    pub bank_id: String,
    pub name: String,
    /// 9 digits, e.g. `044525225`.
    pub bic: String,
}

impl Client {
    /// List all SBP participant banks (`GET /sbp_banks`).
    pub async fn list_sbp_banks(&self) -> Result<Vec<SbpParticipantBank>> {
        let value = self.get("sbp_banks", &[]).await?;
        let items = value
            .get("items")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(serde_json::from_value(items)?)
    }
}
