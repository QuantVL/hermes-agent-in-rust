//! Composer: user input to the D1 `prompt.submit` JSON-RPC request.
//!
//! Chat input is WebSocket-only (there is deliberately no `/api/chat` REST
//! endpoint). The method string here is binding for later UI tasks.

use hermes_core::{Request, Result};
use serde_json::json;

/// The one chat-submission method in the gateway's canonical protocol.
pub const PROMPT_SUBMIT_METHOD: &str = "prompt.submit";

/// Stateless composer that builds prompt requests.
#[derive(Debug, Clone, Copy, Default)]
pub struct Composer;

impl Composer {
    pub fn new() -> Self {
        Self
    }

    /// Build a `prompt.submit` request whose params carry `text` under the
    /// `text` key.
    pub fn submit(&self, text: &str) -> Result<Request> {
        Ok(Request::new(
            PROMPT_SUBMIT_METHOD,
            Some(json!({ "text": text })),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_round_trips_through_serde() {
        let req = Composer::new().submit("hello").unwrap();
        let wire = serde_json::to_string(&req).unwrap();
        assert!(wire.contains("prompt.submit"));
        assert!(wire.contains("hello"));
    }
}
