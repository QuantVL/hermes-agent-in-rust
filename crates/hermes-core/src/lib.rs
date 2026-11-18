//! Hermes Agent (Rust) — shared types, configuration, protocol, and errors.

pub mod config;
pub mod error;
pub mod protocol;

pub use config::{
    Config, ConfigError, EditorConfig, GatewayConfig, ModelConfig, ModelProvider, ProfileConfig,
    ThemeMode, ToolsConfig,
};
pub use error::{HermesError, Result};
pub use protocol::{JsonRpcError, MessageId, Notification, Request, Response};

/// JSON-RPC protocol version implemented by this crate.
pub const PROTOCOL_VERSION: &str = "2.0";
