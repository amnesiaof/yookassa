#![allow(clippy::result_large_err)]
//! Create a payment and capture it.
//!
//! `$env:SHOP_ID = "..."; $env:SECRET_KEY = "test_..."` then
//! `cargo run --example payment -- <payment_id_to_capture>`

use yookassa::{Auth, Client, CreatePaymentRequest, CurrencyCode, MoneyAmount};

#[tokio::main]
async fn main() -> Result<(), yookassa::Error> {
    let shop_id = std::env::var("SHOP_ID").expect("SHOP_ID env var");
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
    let client = Client::new(Auth::Basic {
        shop_id,
        secret_key,
    })?;

    // 1. Create a two-stage payment.
    let payment = client
        .create_payment(
            &CreatePaymentRequest {
                amount: MoneyAmount::new(100, CurrencyCode::Rub),
                description: Some("Order #42".into()),
                confirmation: Some(yookassa::ConfirmationData::Redirect {
                    return_url: "https://example.com/return".into(),
                    enforce: None,
                }),
                ..Default::default()
            },
            None,
        )
        .await?;
    println!("created {} status {}", payment.id, payment.status);
    println!("redirect the user to: {:?}", payment.confirmation);

    // 2. If a payment id was passed, capture it (after the user has paid).
    if let Some(id) = std::env::args().nth(1) {
        let captured = client
            .capture_payment(&id, &Default::default(), None)
            .await?;
        println!("captured {}: {}", captured.id, captured.status);
    }
    Ok(())
}
