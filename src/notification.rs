//! Incoming HTTP notifications about payment events (webhooks).
//!
//! YooKassa sends `POST` notifications with a JSON body to the URL you
//! configured in your merchant profile. There is no signature — authenticity
//! is checked by the sender IP (see [`is_ip_trusted`]).

use crate::error::Error;
use crate::payment::Payment;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// A notification about a payment event.
#[derive(Debug, Clone)]
pub struct WebhookNotification {
    /// Always `notification`.
    pub notification_type: String,
    /// `payment.waiting_for_capture`, `payment.succeeded`, `payment.canceled`.
    pub event: String,
    /// The updated payment object.
    pub object: Payment,
}

impl WebhookNotification {
    /// Parse a notification body (JSON).
    pub fn parse(body: &str) -> crate::error::Result<Self> {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "type", default)]
            type_: Option<String>,
            #[serde(default)]
            event: Option<String>,
            #[serde(default)]
            object: Option<serde_json::Value>,
        }
        let raw: Raw = serde_json::from_str(body)
            .map_err(|e| Error::InvalidNotification(format!("malformed body: {e}")))?;
        if raw.type_.as_deref() != Some("notification") {
            return Err(Error::InvalidNotification(
                "`type` must be \"notification\"".into(),
            ));
        }
        let event = raw
            .event
            .ok_or_else(|| Error::InvalidNotification("missing `event` field".into()))?;
        let object = raw
            .object
            .ok_or_else(|| Error::InvalidNotification("missing `object` field".into()))?;
        if !event.starts_with("payment.") {
            return Err(Error::UnsupportedEvent(event));
        }
        let object: Payment = serde_json::from_value(object)
            .map_err(|e| Error::InvalidNotification(format!("bad payment object: {e}")))?;
        Ok(Self {
            notification_type: "notification".to_string(),
            event,
            object,
        })
    }
}

/// IPv4/IPv6 networks YooKassa sends notifications from
/// (the same list as in the official SDKs).
const TRUSTED_NETWORKS: &[&str] = &[
    "77.75.153.0/25",
    "77.75.156.11",
    "77.75.156.35",
    "77.75.154.128/25",
    "185.71.76.0/27",
    "185.71.77.0/27",
    "2a02:5180:0:1509::/64",
    "2a02:5180:0:2655::/64",
    "2a02:5180:0:1533::/64",
    "2a02:5180:0:2669::/64",
];

/// Check that a notification was sent from a YooKassa IP address.
///
/// Unparseable addresses are reported as untrusted.
pub fn is_ip_trusted(ip: &str) -> bool {
    let Ok(addr) = ip.parse::<IpAddr>() else {
        return false;
    };
    TRUSTED_NETWORKS.iter().any(|net| match addr {
        IpAddr::V4(v4) => parse_v4(net).is_some_and(|(start, end)| v4 >= start && v4 <= end),
        IpAddr::V6(v6) => parse_v6(net).is_some_and(|(start, end)| v6 >= start && v6 <= end),
    })
}

fn parse_v4(s: &str) -> Option<(Ipv4Addr, Ipv4Addr)> {
    let (addr, prefix) = match s.split_once('/') {
        Some((a, p)) => (a, p.parse().unwrap_or(32)),
        None => (s, 32),
    };
    let base = addr.parse::<Ipv4Addr>().ok()?;
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let start = u32::from(base) & mask;
    Some((start.into(), (start | !mask).into()))
}

fn parse_v6(s: &str) -> Option<(Ipv6Addr, Ipv6Addr)> {
    let (addr, prefix) = match s.split_once('/') {
        Some((a, p)) => (a, p.parse().unwrap_or(128)),
        None => (s, 128),
    };
    let base = addr.parse::<Ipv6Addr>().ok()?;
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    let start = u128::from_be_bytes(base.octets()) & mask;
    Some((start.to_be_bytes().into(), (!start).to_be_bytes().into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_ips() {
        assert!(is_ip_trusted("77.75.153.1"));
        assert!(is_ip_trusted("77.75.153.127"));
        assert!(is_ip_trusted("77.75.156.11"));
        assert!(is_ip_trusted("77.75.156.35"));
        assert!(is_ip_trusted("77.75.154.200"));
        assert!(is_ip_trusted("185.71.76.5"));
        assert!(is_ip_trusted("2a02:5180:0:1509::1"));
        assert!(is_ip_trusted("2a02:5180:0:2669::abcd"));
    }

    #[test]
    fn untrusted_ips() {
        assert!(!is_ip_trusted("77.75.153.128"));
        assert!(!is_ip_trusted("77.75.156.12"));
        assert!(!is_ip_trusted("1.2.3.4"));
        assert!(!is_ip_trusted("::1"));
        assert!(!is_ip_trusted("not an ip"));
        assert!(!is_ip_trusted(""));
    }
}
