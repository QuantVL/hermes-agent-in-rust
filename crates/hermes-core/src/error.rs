//! Error types — each source error type appears exactly once so `thiserror`
//! generates a single, non-conflicting `From<E>` impl per source.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HermesError {
    #[error("config error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("gateway error: {0}")]
    Gateway(String),

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("profile not found: {0}")]
    ProfileNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, HermesError>;
