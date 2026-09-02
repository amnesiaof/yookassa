//! Deserialization and serialization tests on samples from the official
//! Python SDK's test suite and the OpenAPI specification.

use std::str::FromStr;
use yookassa::{
    ApiErrorCode, Auth, Client, Confirmation, CreatePaymentRequest, CurrencyCode, Decimal, Error,
    MoneyAmount, NotificationObject, PaymentListQuery, PaymentMethodData, ProductCode,
    WebhookNotification,
};

/// Payment sample from `test_payment_facade.py`.
const PAYMENT_SAMPLE: &str = r#"{
    "id": "21b23b5b-000f-5061-a000-0674e49a8c10",
    "status": "waiting_for_capture",
    "paid": false,
    "amount": {"value": "1.00", "currency": "RUB"},
    "created_at": "2017-11-30T15:45:31.130Z",
    "metadata": {"key": "data", "float_value": "123.32"},
    "payment_method": {"type": "bank_card", "id": "22b23b5b-000f-5061-a000-0674e49a8c99", "saved": false, "status": "inactive"},
    "recipient": {"account_id": "156833", "gateway_id": "468284"},
    "refundable": false,
    "test": false
}"#;

#[test]
fn payment_sample_parses() {
    let payment: yookassa::Payment = serde_json::from_str(PAYMENT_SAMPLE).unwrap();
    assert_eq!(payment.id, "21b23b5b-000f-5061-a000-0674e49a8c10");
    assert!(payment.is_waiting_for_capture());
    assert_eq!(payment.amount.value.to_string(), "1.00");
    assert_eq!(payment.amount.currency, CurrencyCode::Rub);
    assert_eq!(
        payment.created_at.to_rfc3339(),
        "2017-11-30T15:45:31.130+00:00"
    );
    assert_eq!(payment.metadata.unwrap()["key"], Some("data".into()));
    let recipient = payment.recipient.unwrap();
    assert_eq!(recipient.account_id, "156833");
    assert_eq!(recipient.gateway_id.unwrap(), "468284");
    match payment.payment_method.unwrap().data {
        PaymentMethodData::BankCard { .. } => {}
        other => panic!("expected bank_card, got {other:?}"),
    }
}

#[test]
fn payment_status_helpers() {
    let mut payment: yookassa::Payment = serde_json::from_str(PAYMENT_SAMPLE).unwrap();
    let checks = [
        (
            "pending",
            yookassa::Payment::is_pending as fn(&yookassa::Payment) -> bool,
        ),
        (
            "waiting_for_capture",
            yookassa::Payment::is_waiting_for_capture,
        ),
        ("succeeded", yookassa::Payment::is_succeeded),
        ("canceled", yookassa::Payment::is_canceled),
    ];
    for (status, check) in checks {
        payment.status = status.to_string();
        assert!(check(&payment), "status {status}");
    }
}

#[test]
fn payment_with_full_card_and_confirmation_parses() {
    // Sample from `test_webhook_notification.py`.
    let body = r#"{
        "id": "22d6d597-000f-5000-9000-145f6df21d6f",
        "status": "waiting_for_capture",
        "paid": true,
        "amount": {"value": "2.00", "currency": "RUB"},
        "authorization_details": {
            "rrn": "10000000000",
            "auth_code": "000000",
            "three_d_secure": {"applied": true}
        },
        "created_at": "2018-07-10T14:27:54.691Z",
        "expires_at": "2018-07-17T14:28:32.484Z",
        "metadata": {},
        "payment_method": {
            "type": "bank_card",
            "id": "22d6d597-000f-5000-9000-145f6df21d6f",
            "saved": false,
            "card": {
                "first6": "555555",
                "last4": "4444",
                "expiry_month": "07",
                "expiry_year": "2021",
                "card_type": "MasterCard",
                "issuer_country": "RU",
                "issuer_name": "Sberbank"
            },
            "title": "Bank card *4444"
        },
        "refundable": false,
        "test": false
    }"#;
    let payment: yookassa::Payment = serde_json::from_str(body).unwrap();
    let auth = payment.authorization_details.unwrap();
    assert_eq!(auth.auth_code.unwrap(), "000000");
    assert!(auth.three_d_secure.unwrap().applied);
    let method = payment.payment_method.unwrap();
    assert!(!method.saved);
    match method.data {
        PaymentMethodData::BankCard { card } => {
            assert_eq!(card.unwrap().card_type.unwrap(), "MasterCard");
        }
        other => panic!("expected bank_card, got {other:?}"),
    }
}

#[test]
fn unknown_payment_method_type_falls_back() {
    let method: yookassa::PaymentMethod = serde_json::from_str(
        r#"{"type": "some_future_type", "id": "x", "saved": false, "status": "inactive"}"#,
    )
    .unwrap();
    assert!(matches!(method.data, PaymentMethodData::Unknown));
}

#[test]
fn confirmation_variants_parse() {
    let redirect: Confirmation = serde_json::from_str(
        r#"{"type": "redirect", "confirmation_url": "https://yoomoney.ru/pay"}"#,
    )
    .unwrap();
    assert!(matches!(redirect, Confirmation::Redirect { .. }));
    let qr: Confirmation =
        serde_json::from_str(r#"{"type": "qr", "confirmation_data": "https://qr"}"#).unwrap();
    assert!(matches!(qr, Confirmation::Qr { .. }));
}

#[test]
fn api_error_code_mapping() {
    for (code, expected) in [
        ("invalid_request", ApiErrorCode::InvalidRequest),
        ("invalid_credentials", ApiErrorCode::InvalidCredentials),
        ("forbidden", ApiErrorCode::Forbidden),
        ("not_found", ApiErrorCode::NotFound),
        ("gone", ApiErrorCode::Gone),
        ("too_many_requests", ApiErrorCode::TooManyRequests),
        ("refusal", ApiErrorCode::Refusal),
        ("internal_server_error", ApiErrorCode::InternalServerError),
        ("error", ApiErrorCode::Common),
    ] {
        assert_eq!(ApiErrorCode::from_str(code).unwrap(), expected);
        assert_eq!(expected.as_str(), code);
    }
    assert_eq!(
        ApiErrorCode::from_str("whatever_new").unwrap(),
        ApiErrorCode::Unknown("whatever_new".into())
    );
}

#[test]
fn error_body_parses_into_api_error() {
    let body: yookassa::ApiErrorBody = serde_json::from_str(
        r#"{"type":"error","id":"47a1c9a4","code":"not_found","description":"Not found"}"#,
    )
    .unwrap();
    let error = yookassa::ApiError::from_body(404, body);
    assert_eq!(error.status, 404);
    assert_eq!(error.code, ApiErrorCode::NotFound);
    assert_eq!(error.description.as_deref(), Some("Not found"));

    let body: yookassa::ApiErrorBody =
        serde_json::from_str(r#"{"code":"refusal","reason":"fraud_suspected"}"#).unwrap();
    let error = yookassa::ApiError::from_body(400, body);
    assert_eq!(error.reason.as_deref(), Some("fraud_suspected"));
}

#[test]
fn create_payment_request_wire_format() {
    let request = CreatePaymentRequest {
        amount: MoneyAmount::new(Decimal::new(2000, 2), CurrencyCode::Rub),
        description: Some("Order #1".into()),
        payment_method_data: Some(yookassa::PaymentMethodDataRequest::BankCard {
            card: yookassa::CardData {
                number: "4111111111111111".into(),
                expiry_year: "2027".into(),
                expiry_month: "11".into(),
                csc: Some("123".into()),
                cardholder: None,
            },
        }),
        confirmation: Some(yookassa::ConfirmationData::Redirect {
            return_url: "https://example.com/return".into(),
            enforce: None,
        }),
        capture: Some(false),
        ..Default::default()
    };
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "amount": {"value": "20.00", "currency": "RUB"},
            "description": "Order #1",
            "payment_method_data": {"type": "bank_card", "card": {
                "number": "4111111111111111", "expiry_year": "2027", "expiry_month": "11", "csc": "123"
            }},
            "confirmation": {"type": "redirect", "return_url": "https://example.com/return"},
            "capture": false
        })
    );
}

#[test]
fn create_payment_request_minimal_wire_format() {
    let request = CreatePaymentRequest {
        amount: MoneyAmount::new(Decimal::new(2000, 2), CurrencyCode::Rub),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        serde_json::json!({"amount": {"value": "20.00", "currency": "RUB"}})
    );
}

#[test]
fn receipt_wire_format_quantity_is_number() {
    let receipt = yookassa::ReceiptData {
        customer: Some(yookassa::ReceiptCustomer {
            email: Some("test@test.com".into()),
            ..Default::default()
        }),
        items: vec![yookassa::ReceiptItem {
            description: "Item description".into(),
            amount: MoneyAmount::new(Decimal::new(100, 2), CurrencyCode::Rub),
            quantity: Decimal::new(1, 0),
            vat_code: 3,
            ..Default::default()
        }],
        tax_system_code: Some(2),
        ..Default::default()
    };
    let json = serde_json::to_value(&receipt).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "customer": {"email": "test@test.com"},
            "items": [{
                "description": "Item description",
                "amount": {"value": "1.00", "currency": "RUB"},
                "quantity": 1,
                "vat_code": 3
            }],
            "tax_system_code": 2
        })
    );
}

#[test]
fn b2b_sberbank_vat_wire_format() {
    let vat = yookassa::B2bSberbankVatData::Calculated {
        rate: "20".into(),
        amount: MoneyAmount::new(20, CurrencyCode::Rub),
    };
    assert_eq!(
        serde_json::to_value(&vat).unwrap(),
        serde_json::json!({"type": "calculated", "rate": "20", "amount": {"value": "20", "currency": "RUB"}})
    );
    let parsed: yookassa::B2bSberbankVatData = serde_json::from_value(
        serde_json::json!({"type": "mixed", "amount": {"value": "5.00", "currency": "RUB"}}),
    )
    .unwrap();
    assert!(matches!(parsed, yookassa::B2bSberbankVatData::Mixed { .. }));
}

#[test]
fn payment_list_query_wire_format() {
    let query = PaymentListQuery {
        status: Some("succeeded".into()),
        limit: Some(10),
        cursor: Some("abc".into()),
        ..Default::default()
    };
    let json = serde_json::to_value(&query).unwrap();
    assert_eq!(json["status"], "succeeded");
    assert_eq!(json["limit"], 10);
    assert_eq!(json["cursor"], "abc");
    // Unset fields are serialized as null; the client's `to_query` drops them.
    assert!(json["created_at_gte"].is_null());
}

#[test]
fn webhook_notification_parses() {
    // From `test_webhook_notification.py`.
    let body = r#"{
        "type": "notification",
        "event": "payment.waiting_for_capture",
        "object": {
            "id": "22d6d597-000f-5000-9000-145f6df21d6f",
            "status": "waiting_for_capture",
            "paid": true,
            "amount": {"value": "2.00", "currency": "RUB"},
            "created_at": "2018-07-10T14:27:54.691Z",
            "description": "Р—Р°РєР°Р· в„–72",
            "refundable": false,
            "test": false
        }
    }"#;
    let notification = WebhookNotification::parse(body).unwrap();
    assert_eq!(notification.notification_type, "notification");
    assert_eq!(notification.event, "payment.waiting_for_capture");
    assert!(
        matches!(&notification.object, NotificationObject::Payment(p) if p.is_waiting_for_capture())
    );
}

#[test]
fn webhook_notification_validates() {
    let err = WebhookNotification::parse(
        r#"{"type": "wrong", "event": "payment.succeeded", "object": {}}"#,
    );
    assert!(matches!(err, Err(Error::InvalidNotification(_))));
    let err = WebhookNotification::parse(r#"{"type": "notification", "object": {}}"#);
    assert!(matches!(err, Err(Error::InvalidNotification(_))));
    // Unknown event → UnsupportedEvent.
    let err = WebhookNotification::parse(
        r#"{"type": "notification", "event": "deal.closed", "object": {}}"#,
    );
    assert!(matches!(err, Err(Error::UnsupportedEvent(_))));
    // Known event with a malformed object → InvalidNotification.
    let err = WebhookNotification::parse(
        r#"{"type": "notification", "event": "refund.succeeded", "object": {}}"#,
    );
    assert!(matches!(err, Err(Error::InvalidNotification(_))));
}

#[test]
fn webhook_notification_refund_dispatch() {
    let body = r#"{
        "type": "notification",
        "event": "refund.succeeded",
        "object": {
            "id": "21c23aeb-000f-5000-a000-1cc60f4b35a7",
            "payment_id": "21c23aeb-000f-5000-9000-15c60f4b35a7",
            "status": "succeeded",
            "created_at": "2021-06-21T10:20:55.551Z",
            "amount": {"value": "42.00", "currency": "RUB"}
        }
    }"#;
    let notification = WebhookNotification::parse(body).unwrap();
    assert_eq!(notification.event, "refund.succeeded");
    match notification.object {
        NotificationObject::Refund(refund) => {
            assert!(refund.is_succeeded());
            assert_eq!(refund.payment_id, "21c23aeb-000f-5000-9000-15c60f4b35a7");
        }
        other => panic!("expected refund, got {other:?}"),
    }
}

#[test]
fn receipt_item_payment_subject_and_mode() {
    let item: yookassa::ReceiptItem = serde_json::from_value(serde_json::json!({
        "description": "Item",
        "amount": {"value": "1.00", "currency": "RUB"},
        "quantity": 1,
        "vat_code": 1,
        "payment_subject": "commodity",
        "payment_mode": "full_payment"
    }))
    .unwrap();
    assert_eq!(
        item.payment_subject,
        Some(yookassa::PaymentSubject::Commodity)
    );
    assert_eq!(item.payment_mode, Some(yookassa::PaymentMode::FullPayment));

    // Round-trip: unknown values fall back instead of failing.
    let unknown: yookassa::ReceiptItem = serde_json::from_value(serde_json::json!({
        "description": "Item",
        "amount": {"value": "1.00", "currency": "RUB"},
        "quantity": 1,
        "vat_code": 1,
        "payment_subject": "future_subject",
        "payment_mode": "future_mode"
    }))
    .unwrap();
    assert_eq!(
        unknown.payment_subject,
        Some(yookassa::PaymentSubject::Other)
    );
    assert_eq!(unknown.payment_mode, Some(yookassa::PaymentMode::Other));

    // #[serde(other)] variants do not round-trip: an unknown value read from
    // the API serializes back as "Other" and must never be sent upstream.
    assert_eq!(
        serde_json::to_value(&unknown.payment_subject).unwrap(),
        serde_json::json!("Other")
    );
}

#[test]
fn client_requires_credentials() {
    assert!(
        Client::new(Auth::Basic {
            shop_id: "123".into(),
            secret_key: "secret".into(),
        })
        .is_ok()
    );
    assert!(Client::new(Auth::OAuth("token".into())).is_ok());
    assert!(matches!(
        Client::new(Auth::Basic {
            shop_id: "123".into(),
            secret_key: String::new(),
        }),
        Err(Error::Configuration(_))
    ));
}

#[test]
fn product_code_gs10_result() {
    let code = ProductCode::new("010460406000590021N4N57RTCBUZTQ");
    assert_eq!(code.type_, "gs_10");
    assert_eq!(
        code.product_code,
        "44 4D 04 2F F7 5C 76 0C 4E 34 4E 35 37 52 54 43 42 55 5A 54 51"
    );
    assert_eq!(code.mark_code_value, "010460406000590021N4N57RTCBUZTQ");
}

#[test]
fn product_code_ean13_result() {
    let code = ProductCode::new("4006670128002");
    assert_eq!(code.type_, "ean_13");
    assert_eq!(code.product_code, "45 0D 03 A4 E0 26 53 82");
    assert_eq!(code.mark_code_value, "4006670128002");
}

#[test]
fn refund_sample_parses() {
    let body = r#"{
        "id": "21c23aeb-000f-5000-a000-1cc60f4b35a7",
        "payment_id": "21c23aeb-000f-5000-9000-15c60f4b35a7",
        "status": "pending",
        "created_at": "2021-06-21T10:20:55.551Z",
        "amount": {"value": "42.00", "currency": "RUB"},
        "sources": [{"account_id": "111", "amount": {"value": "42.00", "currency": "RUB"}}],
        "refund_method": {"type": "sbp", "sbp_operation_id": "14B0B33E95C14C4193D5AB27F1A3C8A6"}
    }"#;
    let refund: yookassa::Refund = serde_json::from_str(body).unwrap();
    assert!(refund.is_pending());
    assert_eq!(refund.amount.value.to_string(), "42.00");
    assert_eq!(refund.sources.unwrap().len(), 1);
    assert!(matches!(
        refund.refund_method,
        Some(yookassa::RefundMethod::Sbp { .. })
    ));
}

#[test]
fn receipt_sample_parses() {
    let body = r#"{
        "id": "rt-285e6f10-000f-5000-a000-19af3d5c5cf0",
        "type": "payment",
        "status": "pending",
        "payment_id": "285e6f10-000f-5000-a000-19af3d5c5cf0",
        "items": [{
            "description": "РўРµСЃС‚РѕРІС‹Р№ С‚РѕРІР°СЂ",
            "quantity": 2,
            "amount": {"value": "50.00", "currency": "RUB"},
            "vat_code": 2,
            "agent_type": "banking_payment_agent",
            "supplier": {"name": "РћРћРћ Р РѕРјР°С€РєР°", "phone": "79001234567", "inn": "7708662524"}
        }],
        "settlements": [{"type": "cashless", "amount": {"value": "100.00", "currency": "RUB"}}],
        "registered_at": "2021-06-21T10:22:10.000+00:00"
    }"#;
    let receipt: yookassa::Receipt = serde_json::from_str(body).unwrap();
    assert_eq!(receipt.type_, "payment");
    let item = &receipt.items[0];
    assert_eq!(item.quantity.to_string(), "2");
    assert_eq!(item.agent_type.as_deref(), Some("banking_payment_agent"));
    assert_eq!(
        item.supplier.as_ref().unwrap().inn.as_deref(),
        Some("7708662524")
    );
    assert_eq!(receipt.settlements.unwrap()[0].type_, "cashless");
}

#[test]
fn deal_sample_parses() {
    let body = r#"{
        "id": "dl-285e6f10-000f-5000-a000-19af3d5c5cf0",
        "type": "safe_deal",
        "fee_moment": "payment_succeeded",
        "status": "opened",
        "balance": {"value": "0.00", "currency": "RUB"},
        "payout_balance": {"value": "0.00", "currency": "RUB"},
        "created_at": "2024-01-01T10:00:00.000Z",
        "expires_at": "2024-07-01T10:00:00.000Z",
        "test": false
    }"#;
    let deal: yookassa::Deal = serde_json::from_str(body).unwrap();
    assert_eq!(deal.type_, "safe_deal");
    assert_eq!(deal.fee_moment.as_deref(), Some("payment_succeeded"));
    assert_eq!(deal.status.as_deref(), Some("opened"));
}

#[test]
fn payout_samples_parse() {
    let card = r#"{
        "id": "po-285e6f10-000f-5000-a000-19af3d5c5cf0",
        "amount": {"value": "100.00", "currency": "RUB"},
        "status": "succeeded",
        "payout_destination": {
            "type": "bank_card",
            "card": {"first6": "555555", "last4": "4444", "card_type": "MasterCard"}
        },
        "created_at": "2024-01-01T10:00:00.000Z",
        "test": false
    }"#;
    let payout: yookassa::Payout = serde_json::from_str(card).unwrap();
    assert!(payout.is_succeeded());
    assert!(matches!(
        payout.payout_destination,
        yookassa::PayoutDestination::BankCard { .. }
    ));

    let sbp = r#"{
        "id": "po-285e6f10-000f-5000-a000-19af3d5c5cf1",
        "amount": {"value": "50.00", "currency": "RUB"},
        "status": "canceled",
        "payout_destination": {
            "type": "sbp",
            "phone": "79001234567",
            "bank_id": "100000000111",
            "recipient_checked": false
        },
        "created_at": "2024-01-01T10:00:00.000Z",
        "cancellation_details": {"party": "yoo_money", "reason": "recipient_not_found"},
        "test": true
    }"#;
    let payout: yookassa::Payout = serde_json::from_str(sbp).unwrap();
    assert!(payout.is_canceled());
    assert_eq!(
        payout.cancellation_details.unwrap().reason.as_deref(),
        Some("recipient_not_found")
    );
}

#[test]
fn create_payout_request_wire_format() {
    let request = yookassa::CreatePayoutRequest {
        amount: MoneyAmount::new(100, CurrencyCode::Rub),
        payout_destination_data: Some(yookassa::PayoutDestinationData::BankCard {
            card: yookassa::CardDataForPayout {
                number: "5555555555554444".into(),
            },
        }),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        serde_json::json!({
            "amount": {"value": "100", "currency": "RUB"},
            "payout_destination_data": {"type": "bank_card", "card": {"number": "5555555555554444"}}
        })
    );
}

#[test]
fn create_receipt_request_wire_format() {
    let request = yookassa::CreateReceiptRequest {
        type_: "payment".into(),
        payment_id: Some("285e6f10-000f-5000-a000-19af3d5c5cf0".into()),
        customer: yookassa::ReceiptCustomer {
            email: Some("user@example.com".into()),
            ..Default::default()
        },
        items: vec![yookassa::ReceiptItem {
            description: "РўРµСЃС‚РѕРІС‹Р№ С‚РѕРІР°СЂ".into(),
            amount: MoneyAmount::new(50, CurrencyCode::Rub),
            quantity: Decimal::new(2, 0),
            vat_code: 2,
            ..Default::default()
        }],
        send: true,
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        serde_json::json!({
            "type": "payment",
            "payment_id": "285e6f10-000f-5000-a000-19af3d5c5cf0",
            "customer": {"email": "user@example.com"},
            "items": [{
                "description": "РўРµСЃС‚РѕРІС‹Р№ С‚РѕРІР°СЂ",
                "amount": {"value": "50", "currency": "RUB"},
                "quantity": 2,
                "vat_code": 2
            }],
            "send": true
        })
    );
}

#[test]
fn payout_list_query_renames_destination_type() {
    let query = yookassa::PayoutListQuery {
        payout_destination_type: Some("sbp".into()),
        limit: Some(5),
        ..Default::default()
    };
    let json = serde_json::to_value(&query).unwrap();
    assert_eq!(json["payout_destination.type"], "sbp");
    assert_eq!(json["limit"], 5);
    assert!(json.get("status").is_none());
}

#[test]
fn create_deal_request_wire_format() {
    let request = yookassa::CreateDealRequest::default();
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        serde_json::json!({"type": "safe_deal", "fee_moment": "payment_succeeded"})
    );
}

// ---------------------------------------------------------------------------
// Phase 3: invoices, saved payment methods, personal data, sbp banks,
// pos links, webhooks, settings
// ---------------------------------------------------------------------------

const INVOICE_SAMPLE: &str = r#"{
    "id": "in-e44e8088-bd73-43b1-959a-954f3a7d0c54",
    "status": "pending",
    "created_at": "2024-10-18T10:51:18.139Z",
    "expires_at": "2024-10-19T10:51:18.139Z",
    "description": "Order 72",
    "metadata": {"order": "72"},
    "cart": [
        {
            "description": "Account recharge",
            "price": {"value": "100.00", "currency": "RUB"},
            "quantity": 1.0
        }
    ],
    "delivery_method": {"type": "email"},
    "payment_details": {"id": "21b23b5b-000f-5061-a000-0674e49a8c10", "status": "succeeded"}
}"#;

#[test]
fn invoice_sample_parses() {
    let invoice: yookassa::Invoice = serde_json::from_str(INVOICE_SAMPLE).unwrap();
    assert!(invoice.is_pending());
    assert_eq!(invoice.id, "in-e44e8088-bd73-43b1-959a-954f3a7d0c54");
    assert_eq!(invoice.cart.len(), 1);
    assert_eq!(invoice.cart[0].quantity.to_string(), "1");
    assert!(matches!(
        invoice.delivery_method.unwrap(),
        yookassa::InvoiceDeliveryMethod::Email { .. }
    ));
    let details = invoice.payment_details.unwrap();
    assert_eq!(details.status.as_deref(), Some("succeeded"));
}

#[test]
fn create_invoice_request_wire_format() {
    let request = yookassa::CreateInvoiceRequest {
        payment_data: yookassa::InvoicePaymentData {
            amount: MoneyAmount::new(100, CurrencyCode::Rub),
            ..Default::default()
        },
        cart: vec![yookassa::InvoiceLineItem {
            description: "Account recharge".into(),
            price: MoneyAmount::new(100, CurrencyCode::Rub),
            discount_price: None,
            quantity: Decimal::new(1, 0),
        }],
        expires_at: "2024-10-19T10:51:18.139Z".parse().unwrap(),
        delivery_method_data: Some(yookassa::InvoiceDeliveryMethodData::Email {
            email: "user@example.com".into(),
        }),
        ..Default::default()
    };
    let json = serde_json::to_value(&request).unwrap();
    let cart = &json["cart"][0];
    assert_eq!(cart["quantity"], serde_json::json!(1));
    assert_eq!(
        json["delivery_method_data"],
        serde_json::json!({"type": "email", "email": "user@example.com"})
    );
    assert_eq!(json["payment_data"]["amount"]["value"], "100");
}

const SAVED_PM_SAMPLE: &str = r#"{
    "id": "1da5c87d-0984-50e8-a7f3-8de646dd9ec9",
    "type": "bank_card",
    "saved": true,
    "status": "active",
    "title": "Basic card",
    "holder": {"account_id": "123", "gateway_id": "456"},
    "card": {"first6": "555555", "last4": "4444", "expiry_year": "2033", "expiry_month": "11", "card_type": "MasterCard"},
    "metadata": {}
}"#;

#[test]
fn saved_payment_method_sample_parses() {
    let pm: yookassa::SavedPaymentMethod = serde_json::from_str(SAVED_PM_SAMPLE).unwrap();
    assert_eq!(pm.id, "1da5c87d-0984-50e8-a7f3-8de646dd9ec9");
    assert_eq!(pm.status, "active");
    assert_eq!(pm.holder.as_ref().unwrap().account_id, "123");
    match pm.data {
        yookassa::SavedMethodData::BankCard { card } => {
            assert_eq!(card.unwrap().last4.unwrap(), "4444");
        }
        other => panic!("expected bank_card, got {other:?}"),
    }
}

#[test]
fn saved_payment_method_sbp_and_unknown() {
    let sbp: yookassa::SavedPaymentMethod = serde_json::from_str(
        r#"{"id": "pm-1", "type": "sbp", "saved": true, "status": "active",
            "holder": {"account_id": "1"},
            "payer_bank_details": {"bank_id": "100000000111"}}"#,
    )
    .unwrap();
    match sbp.data {
        yookassa::SavedMethodData::Sbp { payer_bank_details } => {
            assert_eq!(payer_bank_details.unwrap().bank_id, "100000000111");
        }
        other => panic!("expected sbp, got {other:?}"),
    }

    let unknown: yookassa::SavedPaymentMethod = serde_json::from_str(
        r#"{"id": "pm-2", "type": "future_type", "saved": false, "status": "inactive", "holder": {"account_id": "1"}}"#,
    )
    .unwrap();
    assert!(matches!(
        unknown_type(unknown_pm(&unknown)),
        yookassa::SavedMethodData::Unknown
    ));
}

fn unknown_pm(pm: &yookassa::SavedPaymentMethod) -> &yookassa::SavedMethodData {
    &pm.data
}

fn unknown_type(data: &yookassa::SavedMethodData) -> &yookassa::SavedMethodData {
    data
}

#[test]
fn saved_method_request_wire_format() {
    let request = yookassa::SavedMethodDataRequest::BankCard {
        card: yookassa::CardData {
            number: "5555555555554477".into(),
            expiry_year: "2033".into(),
            expiry_month: "11".into(),
            csc: Some("123".into()),
            cardholder: None,
        },
        holder: None,
        client_ip: None,
        confirmation: None,
        metadata: None,
    };
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["type"], "bank_card");
    assert_eq!(json["card"]["number"], "5555555555554477");
    assert_eq!(json["card"]["csc"], "123");
}

#[test]
fn personal_data_sample_and_wire() {
    let data: yookassa::PersonalData = serde_json::from_str(
        r#"{
            "id": "pd-22e12f66-000f-5000-8000-18db351245c7",
            "type": "sbp_payout_recipient",
            "status": "active",
            "created_at": "2024-10-18T10:51:18.139Z",
            "metadata": {}
        }"#,
    )
    .unwrap();
    assert_eq!(data.id, "pd-22e12f66-000f-5000-8000-18db351245c7");
    assert_eq!(data.status, "active");

    let sbp_request = yookassa::PersonalDataRequest::SbpPayoutRecipient {
        last_name: "Ivanov".into(),
        first_name: "Ivan".into(),
        middle_name: Some("Ivanovich".into()),
        metadata: None,
    };
    let json = serde_json::to_value(&sbp_request).unwrap();
    assert_eq!(json["type"], "sbp_payout_recipient");
    assert_eq!(json["last_name"], "Ivanov");

    let statement_request = yookassa::PersonalDataRequest::PayoutStatementRecipient {
        last_name: "Ivanov".into(),
        first_name: "Ivan".into(),
        middle_name: None,
        birthdate: "2000-01-01".into(),
        metadata: None,
    };
    assert_eq!(
        serde_json::to_value(&statement_request).unwrap()["birthdate"],
        "2000-01-01"
    );
}

#[test]
fn sbp_banks_list_parses() {
    let list: serde_json::Value = serde_json::from_str(
        r#"{"type": "list", "items": [{"bank_id": "100000000111", "name": "Sberbank", "bic": "044525225"}]}"#,
    )
    .unwrap();
    let items = list["items"].clone();
    let banks: Vec<yookassa::SbpParticipantBank> = serde_json::from_value(items).unwrap();
    assert_eq!(banks[0].bank_id, "100000000111");
    assert_eq!(banks[0].bic, "044525225");
}

#[test]
fn pos_link_sample_and_wire() {
    let link: yookassa::PosLink = serde_json::from_str(
        r#"{
            "id": "pl-1da5c87d-0984-50e8-a7f3-8de646dd9ec9",
            "status": "active",
            "type": "nspk",
            "recipient": {"gateway_id": "468284"},
            "payment": {"id": "21b23b5b-000f-5061-a000-0674e49a8c10", "status": "succeeded"}
        }"#,
    )
    .unwrap();
    assert!(link.is_active());
    assert_eq!(link.type_.as_deref(), Some("nspk"));
    assert_eq!(link.payment.unwrap().status, "succeeded");

    let request = yookassa::CreatePosLinkRequest {
        recipient: yookassa::RecipientData {
            gateway_id: "468284".into(),
        },
        pos_link_data: yookassa::PosLinkData {
            link: "https://qr.nspk.ru/AS10001".into(),
        },
    };
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "recipient": {"gateway_id": "468284"},
            "pos_link_data": {"link": "https://qr.nspk.ru/AS10001"}
        })
    );
}

#[test]
fn webhook_request_wire_format() {
    let request = yookassa::CreateWebhookRequest::new(
        yookassa::WebhookEvent::PaymentSucceeded,
        "https://example.com/hook",
    );
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["event"], "payment.succeeded");
    assert_eq!(json["url"], "https://example.com/hook");

    let list: yookassa::WebhookList = serde_json::from_str(
        r#"{"type": "list", "items": [{"id": "wh-1", "event": "payment.canceled", "url": "https://e.com"}]}"#,
    )
    .unwrap();
    assert_eq!(list.items.len(), 1);
    assert_eq!(
        list.items[0].event().unwrap(),
        yookassa::WebhookEvent::PaymentCanceled
    );
}

#[tokio::test]
async fn webhooks_require_oauth() {
    let basic = Client::new(Auth::Basic {
        shop_id: "1".into(),
        secret_key: "s".into(),
    })
    .unwrap();
    let err = basic.list_webhooks().await.unwrap_err();
    assert!(matches!(err, Error::Configuration(_)));
}

#[test]
fn me_sample_parses() {
    let me: yookassa::Me = serde_json::from_str(
        r#"{
            "account_id": "123",
            "status": "enabled",
            "test": false,
            "fiscalization": {"provider": "atol", "enabled": true},
            "payment_methods": ["bank_card", "sbp"],
            "payout_methods": ["bank_card"],
            "payout_balance": {"value": "1000.00", "currency": "RUB"}
        }"#,
    )
    .unwrap();
    assert!(me.is_enabled());
    assert_eq!(me.fiscalization.unwrap().provider, "atol");
    assert_eq!(me.payout_balance.unwrap().value.to_string(), "1000.00");
    assert_eq!(me.payment_methods, vec!["bank_card", "sbp"]);
}
