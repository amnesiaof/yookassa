//! Payments: models, requests and the `Payment` facade methods.

use crate::amount::MoneyAmount;
use crate::client::Client;
use crate::error::Result;
use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Free-form metadata: up to 16 keys, values are arbitrary strings.
pub type Metadata = HashMap<String, Option<String>>;

/// A payment object (`Payment` in the API).
#[derive(Debug, Clone, Deserialize)]
pub struct Payment {
    pub id: String,
    /// `pending`, `waiting_for_capture`, `succeeded` or `canceled`.
    pub status: String,
    pub amount: MoneyAmount,
    pub created_at: DateTime<FixedOffset>,
    pub paid: bool,
    pub test: bool,
    pub refundable: bool,
    #[serde(default)]
    pub recipient: Option<Recipient>,
    #[serde(default)]
    pub income_amount: Option<MoneyAmount>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub payment_method: Option<PaymentMethod>,
    #[serde(default)]
    pub captured_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub expires_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub confirmation: Option<Confirmation>,
    #[serde(default)]
    pub refunded_amount: Option<MoneyAmount>,
    /// `pending`, `succeeded` or `canceled`.
    #[serde(default)]
    pub receipt_registration: Option<String>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
    #[serde(default)]
    pub cancellation_details: Option<CancellationDetails>,
    #[serde(default)]
    pub authorization_details: Option<AuthorizationDetails>,
    #[serde(default)]
    pub transfers: Option<Vec<Transfer>>,
    #[serde(default)]
    pub deal: Option<PaymentDealInfo>,
    #[serde(default)]
    pub merchant_customer_id: Option<String>,
    #[serde(default)]
    pub invoice_details: Option<InvoiceDetails>,
}

impl Payment {
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }
    pub fn is_waiting_for_capture(&self) -> bool {
        self.status == "waiting_for_capture"
    }
    pub fn is_succeeded(&self) -> bool {
        self.status == "succeeded"
    }
    pub fn is_canceled(&self) -> bool {
        self.status == "canceled"
    }
}

/// Payment recipient.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recipient {
    pub account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_id: Option<String>,
}

/// Payment recipient (request body: `gateway_id` only).
#[derive(Debug, Clone, Serialize)]
pub struct RecipientData {
    pub gateway_id: String,
}

/// Confirmation method (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Confirmation {
    #[serde(rename = "redirect")]
    Redirect {
        confirmation_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        return_url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enforce: Option<bool>,
    },
    #[serde(rename = "external")]
    External {},
    #[serde(rename = "qr")]
    Qr {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmation_data: Option<String>,
    },
    #[serde(rename = "embedded")]
    Embedded {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmation_token: Option<String>,
    },
    #[serde(rename = "mobile_application")]
    MobileApplication {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confirmation_url: Option<String>,
    },
}

/// Confirmation method (request body).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ConfirmationData {
    #[serde(rename = "redirect")]
    Redirect {
        return_url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enforce: Option<bool>,
    },
    #[serde(rename = "external")]
    External {},
    #[serde(rename = "qr")]
    Qr {
        #[serde(skip_serializing_if = "Option::is_none")]
        return_url: Option<String>,
    },
    #[serde(rename = "embedded")]
    Embedded {},
    #[serde(rename = "mobile_application")]
    MobileApplication { return_url: String },
}

/// Reason for the cancellation of a payment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancellationDetails {
    /// `yoo_money`, `payment_network` or `merchant`.
    #[serde(default)]
    pub party: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Authorization details of a card payment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthorizationDetails {
    #[serde(default)]
    pub rrn: Option<String>,
    #[serde(default)]
    pub auth_code: Option<String>,
    #[serde(default)]
    pub three_d_secure: Option<ThreeDSecure>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThreeDSecure {
    pub applied: bool,
}

/// Bank card data in a payment response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BankCardData {
    #[serde(default)]
    pub first6: Option<String>,
    #[serde(default)]
    pub last4: Option<String>,
    #[serde(default)]
    pub expiry_year: Option<String>,
    #[serde(default)]
    pub expiry_month: Option<String>,
    #[serde(default)]
    pub card_type: Option<String>,
    #[serde(default)]
    pub card_product: Option<BankCardProduct>,
    #[serde(default)]
    pub issuer_country: Option<String>,
    #[serde(default)]
    pub issuer_name: Option<String>,
    /// `apple_pay`, `google_pay` or `mir_pay`.
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BankCardProduct {
    pub code: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Card data as returned for payments through Sberbank/Tinkoff/AlfaPay.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvoicingBankCardData {
    #[serde(default)]
    pub first6: Option<String>,
    #[serde(default)]
    pub last4: Option<String>,
    #[serde(default)]
    pub expiry_year: Option<String>,
    #[serde(default)]
    pub expiry_month: Option<String>,
    #[serde(default)]
    pub card_type: Option<String>,
}

/// Raw card data sent in a request (`CardRequestDataWithCsc`).
#[derive(Debug, Clone, Serialize)]
pub struct CardData {
    pub number: String,
    pub expiry_year: String,
    pub expiry_month: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardholder: Option<String>,
}

/// VAT data of a B2B Sberbank payment (`B2bSberbankVatData`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum B2bSberbankVatData {
    #[serde(rename = "calculated")]
    Calculated {
        /// `"5"`, `"7"`, `"10"`, `"20"` or `"22"`.
        rate: String,
        amount: MoneyAmount,
    },
    #[serde(rename = "untaxed")]
    Untaxed {},
    #[serde(rename = "mixed")]
    Mixed { amount: MoneyAmount },
}

/// Payer bank details of a B2B Sberbank payment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct B2bSberbankPayerBankDetails {
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub inn: Option<String>,
    #[serde(default)]
    pub kpp: Option<String>,
    #[serde(default)]
    pub bank_name: Option<String>,
    #[serde(default)]
    pub bank_bic: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
}

/// SBP payer bank details.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SbpPayerBankDetails {
    pub bank_id: String,
    pub bic: String,
}

/// Payment method of a payment (response, polymorphic by `type`).
///
/// Common fields live on [`PaymentMethod`]; this enum carries type-specific
/// data. Unknown types from the API fall back to `Unknown`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum PaymentMethodData {
    #[serde(rename = "bank_card")]
    BankCard {
        #[serde(default)]
        card: Option<BankCardData>,
    },
    #[serde(rename = "cash")]
    Cash {
        #[serde(default)]
        phone: Option<String>,
    },
    #[serde(rename = "qiwi")]
    Qiwi {},
    #[serde(rename = "alfabank")]
    Alfabank {
        #[serde(default)]
        login: Option<String>,
    },
    #[serde(rename = "webmoney")]
    Webmoney {},
    #[serde(rename = "apple_pay")]
    ApplePay {},
    #[serde(rename = "google_pay")]
    GooglePay {},
    #[serde(rename = "yoo_money")]
    YooMoney {
        #[serde(default)]
        account_number: Option<String>,
    },
    #[serde(rename = "sberbank")]
    Sberbank {
        #[serde(default)]
        phone: Option<String>,
        #[serde(default)]
        card: Option<InvoicingBankCardData>,
    },
    #[serde(rename = "mobile_balance")]
    MobileBalance {
        #[serde(default)]
        phone: Option<String>,
    },
    #[serde(rename = "installments")]
    Installments {},
    #[serde(rename = "b2b_sberbank")]
    B2bSberbank {
        #[serde(default)]
        payment_purpose: Option<String>,
        #[serde(default)]
        vat_data: Option<B2bSberbankVatData>,
        #[serde(default)]
        payer_bank_details: Option<B2bSberbankPayerBankDetails>,
    },
    #[serde(rename = "tinkoff_bank")]
    TinkoffBank {
        #[serde(default)]
        card: Option<InvoicingBankCardData>,
    },
    #[serde(rename = "wechat")]
    Wechat {},
    #[serde(rename = "sbp")]
    Sbp {
        #[serde(default)]
        sbp_operation_id: Option<String>,
        #[serde(default)]
        payer_bank_details: Option<SbpPayerBankDetails>,
    },
    #[serde(rename = "sber_loan")]
    SberLoan {
        #[serde(default)]
        loan_option: Option<String>,
        #[serde(default)]
        discount_amount: Option<MoneyAmount>,
        #[serde(default)]
        suspended_until: Option<String>,
    },
    #[serde(rename = "electronic_certificate")]
    ElectronicCertificate {
        #[serde(default)]
        card: Option<BankCardData>,
        #[serde(default)]
        electronic_certificate: Option<ElectronicCertificate>,
        #[serde(default)]
        articles: Option<Vec<ElectronicCertificateArticle>>,
    },
    #[serde(rename = "sber_bnpl")]
    SberBnpl {
        #[serde(default)]
        phone: Option<String>,
    },
    #[serde(rename = "alfa_pay")]
    AlfaPay {
        #[serde(default)]
        card: Option<InvoicingBankCardData>,
    },
    #[serde(other)]
    Unknown,
}

/// Electronic certificate data of a payment (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElectronicCertificate {
    #[serde(default)]
    pub amount: Option<MoneyAmount>,
    pub basket_id: String,
}

/// An approved article of an electronic certificate payment (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElectronicCertificateArticle {
    #[serde(default)]
    pub article_number: Option<u32>,
    #[serde(default)]
    pub tru_code: Option<String>,
    #[serde(default)]
    pub article_code: Option<String>,
    #[serde(default)]
    pub certificates: Option<Vec<ElectronicCertificate>>,
}

/// Payment method (response): common fields + type-specific [`PaymentMethodData`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentMethod {
    pub id: String,
    pub saved: bool,
    /// `active`, `inactive` or `pending`.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(flatten)]
    pub data: PaymentMethodData,
}

/// Payment method data (request body, polymorphic by `type`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum PaymentMethodDataRequest {
    #[serde(rename = "bank_card")]
    BankCard { card: CardData },
    #[serde(rename = "cash")]
    Cash { phone: String },
    #[serde(rename = "sberbank")]
    Sberbank { phone: String },
    #[serde(rename = "tinkoff_bank")]
    TinkoffBank {},
    #[serde(rename = "yoo_money")]
    YooMoney {},
    #[serde(rename = "mobile_balance")]
    MobileBalance { phone: String },
    #[serde(rename = "b2b_sberbank")]
    B2bSberbank {
        payment_purpose: String,
        vat_data: B2bSberbankVatData,
    },
    #[serde(rename = "sbp")]
    Sbp {},
    #[serde(rename = "sber_loan")]
    SberLoan {},
    #[serde(rename = "electronic_certificate")]
    ElectronicCertificate {
        #[serde(skip_serializing_if = "Option::is_none")]
        card: Option<CardData>,
        electronic_certificate: ElectronicCertificatePaymentData,
        #[serde(skip_serializing_if = "Option::is_none")]
        articles: Option<Vec<ElectronicCertificateArticleData>>,
    },
    #[serde(rename = "sber_bnpl")]
    SberBnpl { phone: String },
    #[serde(rename = "alfa_pay")]
    AlfaPay {},
}

/// Electronic certificate data of a payment (request).
#[derive(Debug, Clone, Serialize)]
pub struct ElectronicCertificatePaymentData {
    pub amount: MoneyAmount,
    pub basket_id: String,
}

/// Article data for an electronic certificate payment (request).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ElectronicCertificateArticleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tru_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<ElectronicCertificateData>>,
}

/// Electronic certificate data (request).
#[derive(Debug, Clone, Serialize)]
pub struct ElectronicCertificateData {
    pub certificate_id: String,
    pub tru_quantity: u32,
    pub available_compensation: MoneyAmount,
    pub applied_compensation: MoneyAmount,
}

/// A transfer within a split payment (response).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transfer {
    pub account_id: String,
    pub amount: MoneyAmount,
    /// `pending`, `waiting_for_capture`, `succeeded` or `canceled`.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_fee_amount: Option<MoneyAmount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// A transfer within a split payment (request).
#[derive(Debug, Clone, Serialize)]
pub struct TransferRequest {
    pub account_id: String,
    pub amount: MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee_amount: Option<MoneyAmount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Settlement of a deal within a payment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settlement {
    /// Always `payout`.
    #[serde(rename = "type", default = "default_settlement_type")]
    pub type_: String,
    pub amount: MoneyAmount,
}

fn default_settlement_type() -> String {
    "payout".to_string()
}

/// Deal info inside a payment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentDealInfo {
    pub id: String,
    #[serde(default)]
    pub settlements: Option<Vec<Settlement>>,
}

/// Invoice details inside a payment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvoiceDetails {
    #[serde(default)]
    pub id: Option<String>,
}

// ---------------------------------------------------------------------------
// 54-FZ receipt data (embedded in a payment request)
// ---------------------------------------------------------------------------

/// Receipt data for fiscalization (request).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReceiptData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<ReceiptCustomer>,
    pub items: Vec<ReceiptItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet: Option<bool>,
    /// 1вЂ“6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_system_code: Option<u8>,
    /// 1вЂ“11.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_industry_details: Option<Vec<IndustryDetails>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_operational_details: Option<OperationalDetails>,
}

/// `quantity` goes on the wire as a JSON number (unlike `MoneyAmount.value`,
/// which the API expects as a string).
pub(crate) mod quantity_serde {
    use rust_decimal::Decimal;
    use serde::{Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    pub fn serialize<S: Serializer>(v: &Decimal, s: S) -> Result<S::Ok, S::Error> {
        serde_json::Number::from_str(&v.normalize().to_string())
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Decimal, D::Error> {
        <Decimal as serde::Deserialize>::deserialize(d)
    }
}
/// Receipt customer (request).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReceiptCustomer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// A receipt line item (54-FZ).
///
/// Used both in requests (all `Option` extras skipped when `None`) and in
/// receipt responses (where `agent_type`, `supplier.inn` and
/// `additional_payment_subject_props` may be present).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub description: String,
    pub amount: MoneyAmount,
    /// JSON number on the wire (the API expects a number here, not a string).
    #[serde(with = "quantity_serde")]
    pub quantity: Decimal,
    /// 1вЂ“12.
    pub vat_code: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measure: Option<Measure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_quantity: Option<MarkQuantity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_of_origin_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customs_declaration_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excise: Option<String>,
    /// 54-FZ tag 1162, hex string (see [`crate::product_code`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_code: Option<String>,
    /// 1вЂ“6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_code_info: Option<MarkCodeInfo>,
    /// Must be `"0"` when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_subject_industry_details: Option<Vec<IndustryDetails>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplier: Option<Supplier>,
    /// Only in receipt responses (agent schemes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// Only in `POST /receipts` items (54-FZ tag 1260), в‰¤64 chars.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_payment_subject_props: Option<String>,
}

/// Unit of measurement of a receipt item (54-FZ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Measure {
    #[serde(rename = "piece")]
    Piece,
    #[serde(rename = "gram")]
    Gram,
    #[serde(rename = "kilogram")]
    Kilogram,
    #[serde(rename = "ton")]
    Ton,
    #[serde(rename = "centimeter")]
    Centimeter,
    #[serde(rename = "decimeter")]
    Decimeter,
    #[serde(rename = "meter")]
    Meter,
    #[serde(rename = "square_centimeter")]
    SquareCentimeter,
    #[serde(rename = "square_decimeter")]
    SquareDecimeter,
    #[serde(rename = "square_meter")]
    SquareMeter,
    #[serde(rename = "milliliter")]
    Milliliter,
    #[serde(rename = "liter")]
    Liter,
    #[serde(rename = "cubic_meter")]
    CubicMeter,
    #[serde(rename = "kilowatt_hour")]
    KilowattHour,
    #[serde(rename = "gigacalorie")]
    Gigacalorie,
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "hour")]
    Hour,
    #[serde(rename = "minute")]
    Minute,
    #[serde(rename = "second")]
    Second,
    #[serde(rename = "kilobyte")]
    Kilobyte,
    #[serde(rename = "megabyte")]
    Megabyte,
    #[serde(rename = "gigabyte")]
    Gigabyte,
    #[serde(rename = "terabyte")]
    Terabyte,
    #[serde(rename = "another")]
    Another,
}

/// Fractional quantity marking (54-FZ tag 1291).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkQuantity {
    pub numerator: u32,
    pub denominator: u32,
}

/// Mark code info (54-FZ tag 1163): exactly one field is set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkCodeInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_code_raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ean_8: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ean_13: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itf_14: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gs_10: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gs_1m: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fur: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egais_20: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egais_30: Option<String>,
}

/// Receipt supplier. `inn` is only present in receipt responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Supplier {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inn: Option<String>,
}

/// Receipt industry details (54-FZ tags 1262/1263).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryDetails {
    pub federal_id: String,
    pub document_date: String,
    pub document_number: String,
    pub value: String,
}

/// Receipt operational details (54-FZ tag 1271).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalDetails {
    pub operation_id: u8,
    pub value: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Optional payment parts
// ---------------------------------------------------------------------------

/// Utility payments data (`payment_order`, type `utilities`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename = "utilities")]
pub struct PaymentOrderUtilities {
    pub amount: MoneyAmount,
    pub payment_purpose: String,
    pub recipient: PaymentOrderRecipient,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oktmo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_period: Option<PaymentPeriod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_document_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unified_account_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

/// `payment_order` polymorphic wrapper (only `utilities` is defined by the API).
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum PaymentOrder {
    Utilities(PaymentOrderUtilities),
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentOrderRecipient {
    pub name: String,
    pub inn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kpp: Option<String>,
    pub bank: PaymentOrderBank,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentOrderBank {
    pub name: String,
    pub bic: String,
    pub account: String,
    pub correspondent_account: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentPeriod {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<String>,
    pub year: String,
}

/// Payment receiver for payouts to persons (`receiver`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Receiver {
    #[serde(rename = "mobile_balance")]
    MobileBalance { phone: String },
    #[serde(rename = "digital_wallet")]
    DigitalWallet { account_number: String },
    #[serde(rename = "bank_account")]
    BankAccount { account_number: String, bic: String },
}

/// Statement request for payment confirmations by email.
#[derive(Debug, Clone, Serialize)]
pub struct Statement {
    #[serde(rename = "type")]
    pub type_: String,
    pub delivery_method: StatementDeliveryMethod,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementDeliveryMethod {
    #[serde(rename = "type")]
    pub type_: String,
    pub email: String,
}

/// POS link reference inside a payment request.
#[derive(Debug, Clone, Serialize)]
pub struct PosLinkInfo {
    pub id: String,
    pub expires_at: DateTime<FixedOffset>,
}

/// Airline ticket data (request).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Airline {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub booking_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passengers: Option<Vec<AirlinePassenger>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<Vec<AirlineLeg>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AirlinePassenger {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AirlineLeg {
    pub departure_airport: String,
    pub destination_airport: String,
    pub departure_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_code: Option<String>,
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// Request for creating a payment (`POST /payments`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreatePaymentRequest {
    pub amount: MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ReceiptData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<RecipientData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_data: Option<PaymentMethodDataRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation: Option<ConfirmationData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_payment_method: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub airline: Option<Airline>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfers: Option<Vec<TransferRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal: Option<PaymentDealInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merchant_customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_order: Option<PaymentOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Receiver>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statements: Option<Vec<Statement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos_link: Option<PosLinkInfo>,
}

/// Request for capturing a payment (`POST /payments/{id}/capture`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CapturePaymentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<MoneyAmount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ReceiptData>,
}

/// Query for listing payments (`GET /payments`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct PaymentListQuery {
    pub created_at_gte: Option<DateTime<Utc>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lte: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub captured_at_gte: Option<DateTime<Utc>>,
    pub captured_at_gt: Option<DateTime<Utc>>,
    pub captured_at_lte: Option<DateTime<Utc>>,
    pub captured_at_lt: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub payment_method: Option<String>,
    pub merchant_customer_id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

/// Page of payments (`PaymentList`).
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentList {
    /// Always `list`.
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    pub items: Vec<Payment>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

// ---------------------------------------------------------------------------
// Client methods (Payment facade)
// ---------------------------------------------------------------------------

impl Client {
    /// Create a payment (`POST /payments`).
    ///
    /// When `idempotency_key` is `None` a fresh UUID is sent automatically.
    pub async fn create_payment(
        &self,
        request: &CreatePaymentRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Payment> {
        let value = self.post("payments", request, idempotency_key).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get a payment by id (`GET /payments/{payment_id}`).
    pub async fn get_payment(&self, payment_id: &str) -> Result<Payment> {
        let value = self.get(&format!("payments/{payment_id}"), &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Capture a payment (`POST /payments/{payment_id}/capture`).
    pub async fn capture_payment(
        &self,
        payment_id: &str,
        request: &CapturePaymentRequest,
        idempotency_key: Option<&str>,
    ) -> Result<Payment> {
        let value = self
            .post(
                &format!("payments/{payment_id}/capture"),
                request,
                idempotency_key,
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Cancel a payment (`POST /payments/{payment_id}/cancel`).
    pub async fn cancel_payment(
        &self,
        payment_id: &str,
        idempotency_key: Option<&str>,
    ) -> Result<Payment> {
        let value = self
            .post(
                &format!("payments/{payment_id}/cancel"),
                &serde_json::json!({}),
                idempotency_key,
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List payments (`GET /payments`).
    pub async fn list_payments(&self, query: &PaymentListQuery) -> Result<PaymentList> {
        let value = self
            .get("payments", &crate::client::to_query(query)?)
            .await?;
        Ok(serde_json::from_value(value)?)
    }
}
