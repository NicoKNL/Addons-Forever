//! Crate-wide error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("network: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("provider {provider}: {message}")]
    Provider { provider: String, message: String },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("archive: {0}")]
    Zip(String),
    #[error("config: {0}")]
    Config(String),
    #[error("blocked by privacy allowlist: URL host `{host}` is not an addon provider")]
    Blocked { host: String },
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Provider { provider: provider.into(), message: message.into() }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
