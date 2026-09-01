//! Async client for the YooKassa API.
#![allow(clippy::result_large_err)] // reqwest::Error is large; boxing every result is noise
//!
//! Covers payments, refunds, receipts, deals, payouts, invoices, saved
//! payment methods, POS links, webhooks, account settings and incoming
//! HTTP notifications.
//!
//! ```
//! use yookassa::{Auth, Client, CreatePaymentRequest, MoneyAmount};
//!
//! # async fn example() -> yookassa::Result<()> {
//! let client = Client::new(Auth::Basic {
//!     shop_id: "123456".into(),
//!     secret_key: "test_...".into(),
//! })?;
//!
//! let payment = client
//!     .create_payment(
//!         &CreatePaymentRequest {
//!             amount: MoneyAmount::new(100, yookassa::CurrencyCode::Rub),
//!             description: Some("Order #1".into()),
//!             ..Default::default()
//!         },
//!         None, // a fresh Idempotence-Key is generated automatically
//!     )
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod amount;
mod client;
mod deal;
mod error;
mod invoice;
mod notification;
mod payment;
mod payment_method;
mod payout;
mod personal_data;
mod pos_link;
mod product_code;
mod receipt;
mod refund;
mod sbp_banks;
mod settings;
mod webhook;

pub use amount::{CurrencyCode, MoneyAmount};
pub use client::{Auth, Client, ClientBuilder};
pub use deal::{CreateDealRequest, Deal, DealList, DealListQuery};
pub use error::{ApiError, ApiErrorBody, ApiErrorCode, Error, Result};
pub use invoice::{
    CreateInvoiceRequest, Invoice, InvoiceCancellationDetails, InvoiceDeliveryMethod,
    InvoiceDeliveryMethodData, InvoiceLineItem, InvoiceLineItemResponse, InvoicePaymentData,
    InvoicePaymentDetails,
};
pub use notification::{WebhookNotification, is_ip_trusted};
pub use payment::{
    Airline, AirlineLeg, AirlinePassenger, AuthorizationDetails, B2bSberbankPayerBankDetails,
    B2bSberbankVatData, BankCardData, BankCardProduct, CancellationDetails, CapturePaymentRequest,
    CardData, Confirmation, ConfirmationData, CreatePaymentRequest, ElectronicCertificate,
    ElectronicCertificateArticle, ElectronicCertificateArticleData, ElectronicCertificateData,
    ElectronicCertificatePaymentData, IndustryDetails, InvoiceDetails, InvoicingBankCardData,
    MarkCodeInfo, MarkQuantity, Measure, Metadata, OperationalDetails, Payment, PaymentDealInfo,
    PaymentList, PaymentListQuery, PaymentMethod, PaymentMethodData, PaymentMethodDataRequest,
    PaymentOrder, PaymentOrderBank, PaymentOrderRecipient, PaymentOrderUtilities, PaymentPeriod,
    PosLinkInfo, ReceiptCustomer, ReceiptData, ReceiptItem, Recipient, RecipientData,
    SbpPayerBankDetails, Settlement, Statement, StatementDeliveryMethod, Supplier, ThreeDSecure,
    Transfer, TransferRequest,
};
pub use payment_method::{
    SavedMethodConfirmation, SavedMethodConfirmationData, SavedMethodData, SavedMethodDataRequest,
    SavedMethodHolder, SavedPaymentMethod, SbpPayerBank,
};
pub use payout::{
    CardDataForPayout, CreatePayoutRequest, IncomeReceipt, Payout, PayoutCancellationDetails,
    PayoutCardData, PayoutDealInfo, PayoutDestination, PayoutDestinationData, PayoutList,
    PayoutListQuery, PayoutSearchQuery, PayoutSelfEmployedInfo, PersonalDataRef,
};
pub use personal_data::{PersonalData, PersonalDataRequest};
pub use pos_link::{
    ChangePosLinkRecipientRequest, CreatePosLinkRequest, PosLink, PosLinkData, PosLinkLastPayment,
    PosLinkRecipient,
};
pub use product_code::ProductCode;
pub use receipt::{
    AdditionalUserProps, CreateReceiptRequest, Receipt, ReceiptList, ReceiptListQuery,
    ReceiptSettlement,
};
pub use refund::{
    CreateRefundRequest, ElectronicCertificateRefundArticle, ElectronicCertificateRefundData,
    Refund, RefundAuthorizationDetails, RefundCancellationDetails, RefundDealData, RefundDealInfo,
    RefundList, RefundListQuery, RefundMethod, RefundMethodData, RefundSource,
};
pub use rust_decimal::Decimal;
pub use sbp_banks::SbpParticipantBank;
pub use settings::{FiscalizationData, Me};
pub use webhook::{CreateWebhookRequest, Webhook, WebhookEvent, WebhookList};
