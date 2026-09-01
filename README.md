# yookassa

Async Rust client for the [YooKassa API](https://yookassa.ru/developers/api).

- Basic (shop id + secret key) and OAuth authentication
- Automatic `Idempotence-Key` for POST/DELETE (custom key supported)
- HTTP 202 retries with backoff (same schedule as the official SDK)
- `YM-User-Agent` header, real request timeouts
- Money as `rust_decimal` — amounts are exact decimal strings, never floats
- Typed error codes (`invalid_request`, `refusal` + `reason`, ...)
- Incoming webhook notifications + YooKassa IP allow-list check
- 54-FZ helpers: receipt items, mark codes, tag-1162 `ProductCode`

Covers: payments, refunds, receipts, deals, payouts (+ `/payouts/search`),
invoices, saved payment methods, POS links, webhooks, `/me` settings,
SBP banks list. (No `/self_employed` endpoints — the current API spec does
not define them.)

## Installation

```toml
[dependencies]
yookassa = "0.1"
```

## Usage

### Create a payment

```rust,no_run
use yookassa::{Auth, Client, CreatePaymentRequest, CurrencyCode, MoneyAmount};

#[tokio::main]
async fn main() -> yookassa::Result<()> {
    let client = Client::new(Auth::Basic {
        shop_id: "123456".into(),
        secret_key: "test_...".into(),
    })?;

    let payment = client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::new(100, CurrencyCode::Rub),
                description: Some("Order #1".into()),
                confirmation: Some(yookassa::ConfirmationData::Redirect {
                    return_url: "https://example.com/return".into(),
                    enforce: None,
                }),
                ..Default::default()
            },
            None, // a fresh Idempotence-Key is generated automatically
        )
        .await?;

    // Send the user to payment.confirmation's URL (redirect flow)
    Ok(())
}
```

### Capture / cancel / refund

```rust,no_run
# async fn example(client: &yookassa::Client, payment_id: &str) -> yookassa::Result<()> {
// two-stage payments: confirm the hold
let captured = client.capture_payment(payment_id, &Default::default(), None).await?;

// or cancel it
let canceled = client.cancel_payment(payment_id, None).await?;

// refund a succeeded payment
let refund = client
    .create_refund(
        &yookassa::CreateRefundRequest {
            payment_id: payment_id.into(),
            amount: yookassa::MoneyAmount::new(10, yookassa::CurrencyCode::Rub),
            ..Default::default()
        },
        None,
    )
    .await?;
# Ok(())
# }
```

### Listing with cursor pagination

```rust,no_run
# async fn example(client: &yookassa::Client) -> yookassa::Result<()> {
let mut query = yookassa::PaymentListQuery {
    limit: Some(50),
    status: Some("succeeded".into()),
    ..Default::default()
};
let page = client.list_payments(&query).await?;
if let Some(cursor) = page.next_cursor {
    query.cursor = Some(cursor); // next page
}
# Ok(())
# }
```

### Incoming notifications (webhooks)

YooKassa POSTs a JSON notification when a payment changes status.
Verify the sender IP before trusting the body:

```rust
use yookassa::{is_ip_trusted, WebhookNotification};

fn handle(body: &str, remote_ip: &str) -> yookassa::Result<()> {
    if !is_ip_trusted(remote_ip) {
        return Ok(()); // not from YooKassa — ignore
    }
    let notification = WebhookNotification::parse(body)?;
    match notification.event.as_str() {
        "payment.succeeded" => { /* deliver goods */ }
        "payment.canceled" => { /* release reserved stock */ }
        _ => {}
    }
    Ok(())
}
```

### Webhook management (OAuth only)

```rust,no_run
# async fn example(oauth_token: &str) -> yookassa::Result<()> {
let client = yookassa::Client::new(yookassa::Auth::OAuth(oauth_token.into()))?;
let webhook = client
    .create_webhook(
        &yookassa::CreateWebhookRequest::new(
            yookassa::WebhookEvent::PaymentSucceeded.as_str(),
            "https://example.com/yookassa-hook",
        ),
        None,
    )
    .await?;
# Ok(())
# }
```

### 54-FZ receipts

Receipt items embed into payments, refunds and standalone receipts.
`ProductCode` builds the tag-1162 hex string from a scanned DataMatrix code:

```rust
use yookassa::{MarkCodeInfo, ProductCode};

let scanned = "010460406000590021N4N57RTCBUZTQ";
let code = ProductCode::new(scanned);
assert_eq!(code.type_, "gs_10");
assert_eq!(
    code.product_code,
    "44 4D 04 2F F7 5C 76 0C 4E 34 4E 35 37 52 54 43 42 55 5A 54 51"
);
let _mark_info = MarkCodeInfo { ean_13: Some("4006670128002".into()), ..Default::default() };
```

## Error handling

```rust,no_run
# async fn example(client: &yookassa::Client) {
match client.get_payment("does-not-exist").await {
    Ok(_) => {}
    Err(yookassa::Error::Api(e)) => {
        eprintln!("{} {}: {:?}", e.status, e.code, e.description);
        if e.code == yookassa::ApiErrorCode::Refusal {
            eprintln!("refusal reason: {:?}", e.reason);
        }
    }
    Err(e) => eprintln!("transport: {e}"),
}
# }
```

## License

MIT
