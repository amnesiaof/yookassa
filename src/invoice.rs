//! Invoices: create and get (`/invoices`).

use crate::client::Client;
use crate::error::Result;
use crate::payment::Metadata;
use crate::payment::ReceiptData;
use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Request for creating an invoice (`POST /invoices`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateInvoiceRequest {
    pub payment_data: InvoicePaymentData,
    pub cart: Vec<InvoiceLineItem>,
    pub expires_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_method_data: Option<InvoiceDeliveryMethodData>,
    /// `ru_RU` or `en_US`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Payment parameters of an invoice (`payment_data`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct InvoicePaymentData {
    pub amount: crate::amount::MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ReceiptData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<crate::payment::RecipientData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_payment_method: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// A cart line of an invoice.
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub price: crate::amount::MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_price: Option<crate::amount::MoneyAmount>,
    #[serde(with = "crate::payment::quantity_serde")]
    pub quantity: Decimal,
}

/// Delivery method of an invoice (request).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum InvoiceDeliveryMethodData {
    #[serde(rename = "self")]
    Self_ {},
    #[serde(rename = "sms")]
    Sms { phone: String },
    #[serde(rename = "email")]
    Email { email: String },
}

/// An invoice (`Invoice` in the API).
#[derive(Debug, Clone, Deserialize)]
pub struct Invoice {
    pub id: String,
    /// `pending`, `succeeded` or `canceled`.
    pub status: String,
    pub cart: Vec<InvoiceLineItemResponse>,
    pub created_at: DateTime<FixedOffset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_method: Option<InvoiceDeliveryMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_details: Option<InvoicePaymentDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancellation_details: Option<InvoiceCancellationDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// A cart line in an invoice response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvoiceLineItemResponse {
    pub description: String,
    pub price: crate::amount::MoneyAmount,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_price: Option<crate::amount::MoneyAmount>,
    #[serde(with = "crate::payment::quantity_serde")]
    pub quantity: Decimal,
}

/// Delivery method of an invoice (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum InvoiceDeliveryMethod {
    #[serde(rename = "self")]
    Self_ {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    #[serde(rename = "sms")]
    Sms {},
    #[serde(rename = "email")]
    Email {},
}

/// The payment made for an invoice.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvoicePaymentDetails {
    pub id: String,
    /// Payment status (`pending`, `succeeded`, ...).
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvoiceCancellationDetails {
    /// `yoo_money` or `merchant`.
    #[serde(default)]
    pub party: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

impl Invoice {
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

impl Client {
    /// Create an invoice (`POST /invoices`).
    pub async fn create_invoice(
        &self,
        request: &CreateInvoiceRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Invoice> {
        let value = self.post("invoices", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get an invoice (`GET /invoices/{invoice_id}`).
    pub async fn get_invoice(&self, invoice_id: &str) -> Result<Invoice> {
        let value = self.get(&format!("invoices/{invoice_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }
}
