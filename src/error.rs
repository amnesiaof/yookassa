//! Error types.

use serde::Deserialize;
use std::fmt;

/// Typed YooKassa error code (the `code` field of an error body).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApiErrorCode {
    /// Invalid request parameters.
    InvalidRequest,
    /// Wrong shop credentials or OAuth token.
    InvalidCredentials,
    /// Not enough permissions (e.g. Basic auth on OAuth-only endpoints).
    Forbidden,
    /// Object not found.
    NotFound,
    /// Object expired.
    Gone,
    /// Rate limit exceeded.
    TooManyRequests,
    /// Payment refused by the issuer, `reason` is set on [`ApiError`].
    Refusal,
    /// YooKassa internal error.
    InternalServerError,
    /// Generic error without details.
    Common,
    /// Unknown code from the API.
    Unknown(String),
}

impl ApiErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidCredentials => "invalid_credentials",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Gone => "gone",
            Self::TooManyRequests => "too_many_requests",
            Self::Refusal => "refusal",
            Self::InternalServerError => "internal_server_error",
            Self::Common => "error",
            Self::Unknown(s) => s,
        }
    }
}

impl fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ApiErrorCode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "invalid_request" => Self::InvalidRequest,
            "invalid_credentials" => Self::InvalidCredentials,
            "forbidden" => Self::Forbidden,
            "not_found" => Self::NotFound,
            "gone" => Self::Gone,
            "too_many_requests" => Self::TooManyRequests,
            "refusal" => Self::Refusal,
            "internal_server_error" => Self::InternalServerError,
            "error" => Self::Common,
            other => Self::Unknown(other.to_string()),
        })
    }
}

/// Parsed error body of the API (the `Error` object from the docs).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApiErrorBody {
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parameter: Option<String>,
    #[serde(default)]
    pub retry_after: Option<u64>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// An API-level error: non-2xx status with a parsed error body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// HTTP status code.
    pub status: u16,
    /// Error code from the body.
    pub code: ApiErrorCode,
    /// Unique error id.
    pub id: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Offending parameter name, when the error is about one.
    pub parameter: Option<String>,
    /// Retry delay in milliseconds (present on `too_many_requests`).
    pub retry_after: Option<u64>,
    /// Refusal reason, present when `code` is `refusal`.
    pub reason: Option<String>,
}

impl ApiError {
    /// Build an [`ApiError`] from an HTTP status and a parsed error body.
    pub fn from_body(status: u16, body: ApiErrorBody) -> Self {
        let code = body
            .code
            .as_deref()
            .and_then(|c| c.parse().ok())
            .unwrap_or_else(|| match status {
                400 => ApiErrorCode::InvalidRequest,
                401 => ApiErrorCode::InvalidCredentials,
                403 => ApiErrorCode::Forbidden,
                404 => ApiErrorCode::NotFound,
                410 => ApiErrorCode::Gone,
                429 => ApiErrorCode::TooManyRequests,
                500 => ApiErrorCode::InternalServerError,
                _ => ApiErrorCode::Unknown(String::new()),
            });
        Self {
            status,
            code,
            id: body.id,
            description: body.description,
            parameter: body.parameter,
            retry_after: body.retry_after,
            reason: body.reason,
        }
    }
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.status, self.code)?;
        if let Some(d) = &self.description {
            write!(f, ": {d}")?;
        }
        if let Some(r) = &self.reason {
            write!(f, " (reason: {r})")?;
        }
        Ok(())
    }
}

/// Errors returned by the client.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// API responded with an error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// The payment is still processing (HTTP 202) after all retry attempts.
    #[error("payment is still processing (HTTP 202) after {0} attempts")]
    Processing(u32),
    /// Invalid client configuration.
    #[error("invalid configuration: {0}")]
    Configuration(String),
    /// Notification body is not a valid YooKassa notification.
    #[error("invalid notification: {0}")]
    InvalidNotification(String),
    /// Notification event is not supported by this version of the crate.
    #[error("unsupported notification event: {0}")]
    UnsupportedEvent(String),
    /// HTTP transport error.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// JSON serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// URL parse error.
    #[error(transparent)]
    Url(#[from] url::ParseError),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
