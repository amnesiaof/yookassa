//! HTTP client: auth, idempotence keys, 202 retries, YM-User-Agent.

use crate::error::{ApiError, ApiErrorBody, Error, Result};
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

/// Authentication for the API: shop credentials (Basic) or an OAuth token (Bearer).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Auth {
    /// Shop id + secret key from the merchant profile.
    Basic { shop_id: String, secret_key: String },
    /// OAuth token (required for webhooks and partner endpoints).
    OAuth(String),
}

/// Base backoff between 202 retries: 1.8 s, doubled on every attempt
/// (the same schedule as the Python SDK).
const BACKOFF_BASE: Duration = Duration::from_millis(1800);

/// YooKassa API client.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    auth: Auth,
    base_url: Url,
    max_attempts: u32,
}

/// Builder for a [`Client`] with non-default settings.
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    auth: Auth,
    base_url: Url,
    timeout: Duration,
    max_attempts: u32,
}

impl Client {
    /// Create a client with default settings:
    /// `https://api.yookassa.ru/v3`, 30 s timeout, 3 attempts on HTTP 202.
    pub fn new(auth: Auth) -> Result<Self> {
        Self::builder(auth).build()
    }

    pub fn builder(auth: Auth) -> ClientBuilder {
        ClientBuilder {
            auth,
            base_url: Url::parse("https://api.yookassa.ru/v3").unwrap(),
            timeout: Duration::from_secs(30),
            max_attempts: 3,
        }
    }
}

impl ClientBuilder {
    /// Override the API base URL (default `https://api.yookassa.ru/v3`).
    pub fn base_url(mut self, url: impl Into<String>) -> Result<Self> {
        self.base_url = Url::parse(&url.into())?;
        Ok(self)
    }

    /// Request timeout (default 30 s; the Python SDK sets none).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Number of attempts for POST requests answered with HTTP 202
    /// (default 3, same as the Python SDK `max_attempts`).
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn build(self) -> Result<Client> {
        if matches!(&self.auth, Auth::Basic { shop_id, secret_key }
            if shop_id.is_empty() || secret_key.is_empty())
        {
            return Err(Error::Configuration(
                "shop_id and secret_key must not be empty".into(),
            ));
        }
        let user_agent = format!("YooKassa.Rust/{}", env!("CARGO_PKG_VERSION"));
        let http = reqwest::Client::builder()
            .timeout(self.timeout)
            .user_agent(&user_agent)
            .build()?;
        Ok(Client {
            http,
            auth: self.auth,
            base_url: self.base_url,
            max_attempts: self.max_attempts,
        })
    }
}

impl Client {
    fn auth_header(&self) -> String {
        match &self.auth {
            Auth::Basic {
                shop_id,
                secret_key,
            } => {
                let raw = format!("{shop_id}:{secret_key}");
                let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
                format!("Basic {encoded}")
            }
            Auth::OAuth(token) => format!("Bearer {token}"),
        }
    }

    /// Verify the client uses an OAuth token (webhooks are OAuth-only).
    pub(crate) fn require_oauth(&self) -> Result<()> {
        match &self.auth {
            Auth::OAuth(_) => Ok(()),
            Auth::Basic { .. } => Err(Error::Configuration(
                "webhooks require an OAuth token, not Basic auth".into(),
            )),
        }
    }

    fn url_for(&self, path: &str) -> Url {
        self.base_url.join(path).expect("valid relative path")
    }

    pub(crate) async fn post(
        &self,
        path: &str,
        body: &impl Serialize,
        idempotency_key: Option<&str>,
    ) -> Result<Value> {
        let idempotency_key = idempotency_key
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        for attempt in 1..=self.max_attempts.max(1) {
            let resp = self
                .http
                .post(self.url_for(path))
                .header("YM-User-Agent", self.user_agent())
                .header("Authorization", self.auth_header())
                .header("Idempotence-Key", &idempotency_key)
                .json(body)
                .send()
                .await?;
            if resp.status() == 202 {
                if attempt < self.max_attempts.max(1) {
                    tokio::time::sleep(BACKOFF_BASE * 2u32.pow(attempt - 1)).await;
                    continue;
                }
                return Err(Error::Processing(self.max_attempts));
            }
            return self.decode(resp).await;
        }
        unreachable!()
    }

    pub(crate) async fn get(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let resp = self
            .http
            .get(self.url_for(path))
            .header("YM-User-Agent", self.user_agent())
            .header("Authorization", self.auth_header())
            .query(query)
            .send()
            .await?;
        self.decode(resp).await
    }

    #[allow(dead_code)] // used by the webhooks facade
    pub(crate) async fn delete(&self, path: &str, idempotency_key: Option<&str>) -> Result<Value> {
        let idempotency_key = idempotency_key
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let resp = self
            .http
            .delete(self.url_for(path))
            .header("YM-User-Agent", self.user_agent())
            .header("Authorization", self.auth_header())
            .header("Idempotence-Key", &idempotency_key)
            .send()
            .await?;
        self.decode(resp).await
    }

    async fn decode(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        if status.is_success() {
            let value = resp.json::<Value>().await?;
            return Ok(value);
        }
        let body: Result<ApiErrorBody> = resp.json().await.map_err(Error::from);
        match body {
            Ok(body) => Err(ApiError::from_body(status.as_u16(), body).into()),
            Err(_) => Err(ApiError {
                status: status.as_u16(),
                code: crate::error::ApiErrorCode::Unknown(String::new()),
                id: None,
                description: None,
                parameter: None,
                retry_after: None,
                reason: None,
            }
            .into()),
        }
    }

    fn user_agent(&self) -> String {
        format!(
            "YooKassa.Rust/{} ({})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS
        )
    }
}

/// Serialize a list query into URL query pairs (RFC 3339 datetimes, `Z` suffix).
pub(crate) fn to_query<T: Serialize>(query: &T) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(query)?;
    let obj = match value {
        Value::Null => return Ok(Vec::new()),
        Value::Object(map) => map,
        _ => return Ok(Vec::new()),
    };
    let mut pairs = Vec::new();
    for (k, v) in obj {
        if v.is_null() {
            continue;
        }
        let text = match v {
            Value::String(s) => s,
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            _ => continue,
        };
        pairs.push((k, text));
    }
    Ok(pairs)
}
