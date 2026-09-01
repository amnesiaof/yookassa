#![allow(clippy::result_large_err)]
//! List payments, refunds, deals and payouts (cursor pagination).
//!
//! `$env:SHOP_ID = "..."; $env:SECRET_KEY = "test_..."` then
//! `cargo run --example lists`

use yookassa::{Auth, Client, PaymentListQuery};

#[tokio::main]
async fn main() -> Result<(), yookassa::Error> {
    let shop_id = std::env::var("SHOP_ID").expect("SHOP_ID env var");
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
    let client = Client::new(Auth::Basic {
        shop_id,
        secret_key,
    })?;

    // Payments: succeeded, page by page.
    let mut query = PaymentListQuery {
        limit: Some(20),
        status: Some("succeeded".into()),
        ..Default::default()
    };
    loop {
        let page = client.list_payments(&query).await?;
        for p in &page.items {
            println!("payment {} {} {}", p.id, p.status, p.amount);
        }
        match page.next_cursor {
            Some(cursor) => query.cursor = Some(cursor),
            None => break,
        }
    }

    // Refunds / deals / payouts use the same shape:
    let refunds = client.list_refunds(&Default::default()).await?;
    println!("{} refunds", refunds.items.len());
    let deals = client.list_deals(&Default::default()).await?;
    println!("{} deals", deals.items.len());
    Ok(())
}
