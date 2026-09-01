#![allow(clippy::result_large_err)]
//! Incoming webhook notifications: parse + IP allow-list check.
//!
//! No real HTTP server here — plug `handle()` into axum/actix/hyper/whatever.

use yookassa::{WebhookNotification, is_ip_trusted};

fn handle_notification(body: &str, remote_ip: &str) -> Result<(), yookassa::Error> {
    // 1. Only trust requests actually sent from YooKassa.
    if !is_ip_trusted(remote_ip) {
        return Ok(());
    }

    // 2. Parse the JSON body.
    let notification = WebhookNotification::parse(body)?;
    println!(
        "event {} for payment {}",
        notification.event, notification.object.id
    );

    // 3. React.
    if notification.object.is_succeeded() {
        println!(
            "payment {} succeeded — deliver the goods",
            notification.object.id
        );
    } else if notification.object.is_canceled() {
        println!("payment {} was canceled", notification.object.id);
    }
    Ok(())
}

fn main() {
    // Simulated POST body and sender IP:
    let body = r#"{
        "type": "notification",
        "event": "payment.succeeded",
        "object": {
            "id": "22e6b169-000f-5000-a00-e6b91c66fd8e",
            "status": "succeeded",
            "paid": true,
            "amount": {"value": "400.00", "currency": "RUB"},
            "created_at": "2022-07-20T09:39:38.388Z",
            "recipient": {"account_id": "222111", "gateway_id": "111222"},
            "refundable": true,
            "test": false
        }
    }"#;
    handle_notification(body, "77.75.153.1").unwrap(); // trusted
    assert!(!is_ip_trusted("203.0.113.1")); // attacker — skipped
    println!("done");
}
