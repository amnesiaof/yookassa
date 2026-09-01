//! Monetary amounts.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Currency code. Values from the official specification;
/// unknown codes from the API fall back to [`CurrencyCode::Unknown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CurrencyCode {
    #[serde(rename = "RUB")]
    Rub,
    #[serde(rename = "EUR")]
    Eur,
    #[serde(rename = "USD")]
    Usd,
    #[serde(rename = "KZT")]
    Kzt,
    #[serde(rename = "BYN")]
    Byn,
    #[serde(rename = "UAH")]
    Uah,
    #[serde(rename = "UZS")]
    Uzs,
    #[serde(rename = "TRY")]
    Try,
    #[serde(rename = "INR")]
    Inr,
    #[serde(rename = "MDL")]
    Mdl,
    #[serde(rename = "AZN")]
    Azn,
    #[serde(rename = "AMD")]
    Amd,
    #[serde(other)]
    Unknown,
}

impl CurrencyCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rub => "RUB",
            Self::Eur => "EUR",
            Self::Usd => "USD",
            Self::Kzt => "KZT",
            Self::Byn => "BYN",
            Self::Uah => "UAH",
            Self::Uzs => "UZS",
            Self::Try => "TRY",
            Self::Inr => "INR",
            Self::Mdl => "MDL",
            Self::Azn => "AZN",
            Self::Amd => "AMD",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Monetary amount: `{"value": "1000.00", "currency": "RUB"}`.
///
/// The value is serialized exactly as the API expects — a decimal string
/// with two fractional digits — using `rust_decimal`, never floats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyAmount {
    pub value: Decimal,
    pub currency: CurrencyCode,
}

impl MoneyAmount {
    /// `MoneyAmount::new(100, CurrencyCode::Rub)` → `{"value":"100","currency":"RUB"}`.
    pub fn new(value: impl Into<Decimal>, currency: CurrencyCode) -> Self {
        Self {
            value: value.into(),
            currency,
        }
    }
}

impl Default for MoneyAmount {
    fn default() -> Self {
        Self::new(Decimal::ZERO, CurrencyCode::Rub)
    }
}

impl fmt::Display for MoneyAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_string_roundtrip() {
        let json = r#"{"value":"100.00","currency":"RUB"}"#;
        let amount: MoneyAmount = serde_json::from_str(json).unwrap();
        assert_eq!(amount.value, Decimal::new(10000, 2));
        assert_eq!(amount.currency, CurrencyCode::Rub);
        assert_eq!(serde_json::to_string(&amount).unwrap(), json);
    }

    #[test]
    fn amount_scale_preserved() {
        let amount = MoneyAmount::new(Decimal::new(10000, 2), CurrencyCode::Rub);
        assert_eq!(
            serde_json::to_string(&amount).unwrap(),
            r#"{"value":"100.00","currency":"RUB"}"#
        );
    }

    #[test]
    fn unknown_currency_falls_back() {
        let amount: MoneyAmount =
            serde_json::from_str(r#"{"value":"1.00","currency":"CNY"}"#).unwrap();
        assert_eq!(amount.currency, CurrencyCode::Unknown);
    }
}
