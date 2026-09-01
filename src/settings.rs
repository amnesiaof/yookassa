//! Account settings (`GET /me`).

use crate::client::Client;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Store or gateway settings (`Me`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Me {
    pub account_id: String,
    /// `enabled` or `disabled`.
    pub status: String,
    pub test: bool,
    #[serde(default)]
    pub fiscalization: Option<FiscalizationData>,
    /// Deprecated by the API: use [`Me::fiscalization`].
    #[serde(default)]
    pub fiscalization_enabled: Option<bool>,
    /// Payment method type codes available for the account.
    #[serde(default)]
    pub payment_methods: Vec<String>,
    /// INN, 1вЂ“20 digits.
    #[serde(default)]
    pub itn: Option<String>,
    /// Payout method type codes: `bank_card`, `yoo_money`, `sbp`.
    #[serde(default)]
    pub payout_methods: Vec<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// Gateway payout balance (gateways only).
    #[serde(default)]
    pub payout_balance: Option<crate::amount::MoneyAmount>,
}

impl Me {
    pub fn is_enabled(&self) -> bool {
        self.status == "enabled"
    }
}

/// Fiscalization settings of the account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FiscalizationData {
    pub enabled: bool,
    /// Provider code, e.g. `atol`, `yoo_receipt` (15 documented values).
    pub provider: String,
}

impl Client {
    /// Get settings of your account (`GET /me`).
    ///
    /// Partners may request settings of a sub-merchant by passing its
    /// account id as `on_behalf_of`.
    pub async fn get_settings(&self, on_behalf_of: Option<&str>) -> Result<Me> {
        let query = match on_behalf_of {
            Some(id) => vec![("on_behalf_of".to_string(), id.to_string())],
            None => Vec::new(),
        };
        let value = self.get("me", &query).await?;
        Ok(serde_json::from_value(value)?)
    }
}
