//! Hermes gateway client: recon-verified discovery, native control-socket
//! session tickets, REST and WebSocket transports, and origin-lock path
//! validation.
//!
//! All wire formats in this crate are binding and were verified against the
//! Python source; do not invent protocol fields here.

pub mod control;
pub mod discovery;
pub mod error;
pub mod origin;
pub mod rest;
pub mod stream;
pub mod ticket;

pub use control::{control_socket_path, CONTROL_SOCKET_NAME, CONTROL_TIMEOUT};
pub use discovery::{
    discover, parse_ensure_response, verdict_from_exit_and_line, GatewayEndpoint,
    REQUIRED_CAPABILITY, SUPPORTED_RUNTIME_PROTOCOL,
};
pub use error::{ErrorCode, GatewayError, Result};
pub use origin::{join_api_origin, validate_bare_origin};
pub use rest::GatewayClient;
pub use stream::{GatewayEvent, WsStream};
pub use ticket::{validate_ticket_token, Ticket, TicketBroker, TicketPurpose};
