//! Error types for the gateway client.
//!
//! Each external source error type appears in exactly one `#[from]` variant so
//! `thiserror` never generates conflicting `From<E>` impls (E0119).
//!
//! `ErrorCode` is the semantic mapping of `gateway ensure` exit codes 2..7.
//! Callers must branch on it, not on raw integers.

use thiserror::Error;

/// Semantic meaning of `gateway ensure` exit codes 2..7.
///
/// The client's behaviour for each code is documented on the variant; these
/// rules are binding for every consumer of `discover`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorCode {
    /// Exit 2 — the request/config was invalid. Do **not** retry; surface the
    /// reason to the user and fix the input.
    #[default]
    Invalid = 2,

    /// Exit 3 — `runtime_protocol` mismatch. Refuse the connection and tell
    /// the user to upgrade the client; retrying cannot help.
    Incompatible = 3,

    /// Exit 4 — authorization failed. Re-authenticate, then retry once.
    Authorization = 4,

    /// Exit 5 — gateway is starting or a deadline was exceeded. Retry with
    /// bounded exponential backoff.
    Starting = 5,

    /// Exit 6 — gateway is draining. Stop starting new work and fail over to
    /// another instance if one is available.
    Draining = 6,

    /// Exit 7 — gateway is inaccessible or an instance conflict occurred.
    /// Surface to the user; do not silently retry in a tight loop.
    Inaccessible = 7,
}

impl ErrorCode {
    /// Map a raw `gateway ensure` exit code to its semantic variant.
    pub fn from_exit_code(code: i32) -> Option<Self> {
        match code {
            2 => Some(Self::Invalid),
            3 => Some(Self::Incompatible),
            4 => Some(Self::Authorization),
            5 => Some(Self::Starting),
            6 => Some(Self::Draining),
            7 => Some(Self::Inaccessible),
            _ => None,
        }
    }

    /// Map the `state` string from the `ensure --json` envelope.
    pub fn from_state(state: &str) -> Option<Self> {
        match state {
            "invalid" => Some(Self::Invalid),
            "incompatible" => Some(Self::Incompatible),
            "authorization" | "unauthorized" => Some(Self::Authorization),
            "starting" | "deadline" => Some(Self::Starting),
            "draining" => Some(Self::Draining),
            "inaccessible" | "conflict" => Some(Self::Inaccessible),
            _ => None,
        }
    }

    /// The raw exit code for this variant.
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// All failures produced by this crate.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("websocket error: {0}")]
    Ws(String),

    #[error(transparent)]
    Core(#[from] hermes_core::HermesError),

    #[error("discovery failed: {0}")]
    Discovery(String),

    #[error("incompatible gateway: {0}")]
    Incompatible(String),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("invalid origin: {0}")]
    InvalidOrigin(String),

    #[error("invalid ticket: {0}")]
    InvalidTicket(String),

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("gateway ensure exited with code {code}: {message}")]
    ExitCode {
        code: i32,
        error_code: Option<ErrorCode>,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, GatewayError>;
