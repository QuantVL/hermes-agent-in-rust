//! Native session tickets, minted over the private control socket.
//!
//! There is deliberately no `gateway attach --json` call here: that verb only
//! mints `purpose=interactive`. All four native purposes go through the
//! `session-ticket` control-socket verb. Tickets are single-use (~30s), so the
//! broker never caches — callers mint one per request.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::{control, GatewayError, Result};

/// The four purposes accepted by the `session-ticket` control verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TicketPurpose {
    /// WebSocket connection (the only purpose `gateway attach` can mint).
    #[default]
    Interactive,
    /// One REST request.
    NativeHttp,
    /// Creating a profile through the native API.
    NativeProfileCreate,
    /// Reading/mutating the native profile registry.
    NativeProfileRegistry,
}

impl TicketPurpose {
    /// Exact wire string for this purpose.
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::NativeHttp => "native-http",
            Self::NativeProfileCreate => "native-profile-create",
            Self::NativeProfileRegistry => "native-profile-registry",
        }
    }
}

/// A single-use, short-lived gateway ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket(pub String);

impl Ticket {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Mints fresh tickets against one profile's control socket.
#[derive(Debug, Clone)]
pub struct TicketBroker {
    control_home: PathBuf,
    profile_id: String,
    instance_id: String,
}

impl TicketBroker {
    pub fn new(
        control_home: impl AsRef<Path>,
        profile_id: impl Into<String>,
        instance_id: impl Into<String>,
    ) -> Self {
        Self {
            control_home: control_home.as_ref().to_path_buf(),
            profile_id: profile_id.into(),
            instance_id: instance_id.into(),
        }
    }

    /// Mint one fresh ticket for `purpose`. No caching, no reuse.
    pub fn mint(&self, purpose: TicketPurpose) -> Result<Ticket> {
        let request = json!({
            "protocol": 1,
            "id": 1,
            "verb": "session-ticket",
            "params": {
                "profile_id": self.profile_id.clone(),
                "instance_id": self.instance_id.clone(),
                "purpose": purpose.as_wire(),
            }
        });
        let mut frame = serde_json::to_string(&request)?;
        frame.push('\n');

        let response_line = control::query(&self.control_home, &frame)?;
        let response: Value = serde_json::from_str(&response_line)?;

        let ticket = response
            .get("ticket")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                GatewayError::InvalidState("session-ticket response has no `ticket` field".into())
            })?;

        let profile_id = response.get("profile_id").and_then(Value::as_str);
        if profile_id != Some(self.profile_id.as_str()) {
            return Err(GatewayError::InvalidState(format!(
                "ticket profile_id mismatch: {profile_id:?}"
            )));
        }
        let instance_id = response.get("instance_id").and_then(Value::as_str);
        if instance_id != Some(self.instance_id.as_str()) {
            return Err(GatewayError::InvalidState(format!(
                "ticket instance_id mismatch: {instance_id:?}"
            )));
        }
        if response.get("runtime_protocol").and_then(Value::as_u64) != Some(1) {
            return Err(GatewayError::Incompatible(
                "session-ticket response has runtime_protocol != 1".into(),
            ));
        }

        Ok(Ticket(ticket.to_string()))
    }
}

/// Validate a ticket as an RFC 6455 subprotocol token before dialing.
///
/// RFC 6455 tokens use the HTTP `token` grammar: ASCII alphanumerics plus
/// `! # $ % & ' * + - . ^ _ ` | ~`. In particular `=`, `/` and whitespace are
/// rejected so a ticket can never inject extra subprotocols or a header.
pub fn validate_ticket_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(GatewayError::InvalidTicket("ticket is empty".into()));
    }
    if !token.bytes().all(is_token_byte) {
        return Err(GatewayError::InvalidTicket(format!(
            "ticket is not a valid RFC 6455 token: {token:?}"
        )));
    }
    Ok(())
}

fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}
