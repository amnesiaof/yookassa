//! HTTP-level tests against a real (local) mock server: auth headers,
//! idempotence keys, 202 retries and the OAuth-only webhook endpoints.

use std::time::Duration;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use yookassa::{
    Auth, Client, CreatePaymentRequest, CreateWebhookRequest, MoneyAmount, WebhookEvent,
};

const PAYMENT_JSON: &str = r#"{
    "id": "22c5a8d9-000f-5000-9000-1a68514ee568",
    "status": "pending",
    "amount": {"value": "2.00", "currency": "RUB"},
    "created_at": "2019-08-28T09:57:35.341Z",
    "paid": false,
    "test": false,
    "refundable": true,
    "recipient": {"account_id": "111", "gateway_id": "222"}
}"#;

fn basic_client(base_url: &str) -> Client {
    Client::builder(Auth::Basic {
        shop_id: "123456".into(),
        secret_key: "test_secret".into(),
    })
    .base_url(base_url)
    .unwrap()
    .build()
    .unwrap()
}

fn oauth_client(base_url: &str) -> Client {
    Client::builder(Auth::OAuth("oauth-token".into()))
        .base_url(base_url)
        .unwrap()
        .build()
        .unwrap()
}

#[tokio::test]
async fn basic_auth_and_body_are_sent() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/payments"))
        .and(header("Authorization", "Basic MTIzNDU2OnRlc3Rfc2VjcmV0"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAYMENT_JSON))
        .expect(1)
        .mount(&server)
        .await;

    let client = basic_client(&server.uri());
    let payment = client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::new(2, yookassa::CurrencyCode::Rub),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(payment.id, "22c5a8d9-000f-5000-9000-1a68514ee568");
    assert!(payment.is_pending());
}

#[tokio::test]
async fn custom_idempotence_key_is_passed_through() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/payments"))
        .and(header("Idempotence-Key", "my-fixed-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAYMENT_JSON))
        .expect(1)
        .mount(&server)
        .await;

    let client = basic_client(&server.uri());
    client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::default(),
                ..Default::default()
            },
            Some("my-fixed-key"),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn auto_idempotence_key_is_a_uuid() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/payments"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAYMENT_JSON))
        .expect(1)
        .mount(&server)
        .await;

    let client = basic_client(&server.uri());
    client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::default(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    let request = &server.received_requests().await.unwrap()[0];
    let key = request
        .headers
        .get("Idempotence-Key")
        .expect("Idempotence-Key must be sent automatically")
        .to_str()
        .unwrap();
    assert!(
        uuid::Uuid::parse_str(key).is_ok(),
        "auto key must be a UUID, got {key}"
    );
}

#[tokio::test]
async fn http_202_is_retried_then_processing_error() {
    let server = MockServer::start().await;
    // Always answer 202: the client should retry 3 times total, then fail.
    Mock::given(method("POST"))
        .and(path("/payments"))
        .respond_with(ResponseTemplate::new(202).set_body_string("{}"))
        .expect(3)
        .mount(&server)
        .await;

    let client = oauth_client(&server.uri());
    let err = client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::default(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap_err();

    match err {
        yookassa::Error::Processing(attempts) => assert_eq!(attempts, 3),
        other => panic!("expected Processing, got {other:?}"),
    }
}

#[tokio::test]
async fn http_202_retries_then_succeeds() {
    let server = MockServer::start().await;
    // First attempt: 202, second: 200 with the payment body.
    Mock::given(method("POST"))
        .and(path("/payments"))
        .respond_with(ResponseTemplate::new(202).set_body_string("{}"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/payments"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAYMENT_JSON))
        .expect(1)
        .mount(&server)
        .await;

    let client = oauth_client(&server.uri());
    let payment = client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::default(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert!(payment.is_pending());
}

#[tokio::test]
async fn api_error_body_is_parsed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/payments"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "type": "error",
            "id": "err-1",
            "code": "invalid_request",
            "description": "Bad amount",
            "parameter": "amount"
        })))
        .mount(&server)
        .await;

    let client = basic_client(&server.uri());
    let err = client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::default(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap_err();

    match err {
        yookassa::Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.code, yookassa::ApiErrorCode::InvalidRequest);
            assert_eq!(api.description.as_deref(), Some("Bad amount"));
            assert_eq!(api.parameter.as_deref(), Some("amount"));
        }
        other => panic!("expected Api, got {other:?}"),
    }
}

#[tokio::test]
async fn list_payments_sends_query_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/payments"))
        .and(query_param("status", "succeeded"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "type": "list",
            "items": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = basic_client(&server.uri());
    let list = client
        .list_payments(&yookassa::PaymentListQuery {
            status: Some("succeeded".into()),
            limit: Some(5),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(list.items.is_empty());
}

#[tokio::test]
async fn webhooks_require_oauth_token() {
    let server = MockServer::start().await;
    let basic = basic_client(&server.uri());
    let err = basic
        .create_webhook(
            &CreateWebhookRequest::new(WebhookEvent::PaymentSucceeded, "https://example.com/hook"),
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, yookassa::Error::Configuration(_)));

    let err = basic.delete_webhook("wh-id", None).await.unwrap_err();
    assert!(matches!(err, yookassa::Error::Configuration(_)));
}

#[tokio::test]
async fn oauth_webhooks_send_bearer_and_delete_works() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhooks"))
        .and(header("Authorization", "Bearer oauth-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wh-1",
            "event": "payment.succeeded",
            "url": "https://example.com/hook"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/webhooks/wh-1"))
        .and(header("Authorization", "Bearer oauth-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .expect(1)
        .mount(&server)
        .await;

    let client = oauth_client(&server.uri());
    let webhook = client
        .create_webhook(
            &CreateWebhookRequest::new(WebhookEvent::PaymentSucceeded, "https://example.com/hook"),
            None,
        )
        .await
        .unwrap();
    assert_eq!(webhook.id, "wh-1");
    client.delete_webhook("wh-1", None).await.unwrap();
}

#[tokio::test]
async fn timeouts_and_slow_server_propagate_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/payments"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(PAYMENT_JSON)
                .set_delay(Duration::from_secs(2)),
        )
        .mount(&server)
        .await;

    // 100ms timeout — much shorter than the mock's 2s delay.
    let client = Client::builder(Auth::OAuth("t".into()))
        .base_url(server.uri())
        .unwrap()
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();
    let result = client
        .list_payments(&yookassa::PaymentListQuery::default())
        .await;
    assert!(result.is_err(), "request must fail on timeout");
}
