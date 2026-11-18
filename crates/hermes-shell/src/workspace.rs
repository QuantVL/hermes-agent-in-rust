//! Workspace state — the active workspace and its session list.
//!
//! Construction and tests are gateway-free: an empty session list is built
//! from JSON, and the only gateway-touching method (`refresh_sessions`) takes
//! a client supplied by the caller and is never invoked in tests.

use hermes_chat::SessionList;
use hermes_gateway::GatewayClient;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub sessions: SessionList,
}

impl Workspace {
    /// Build a workspace with an empty session list.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            sessions: SessionList::from_value(serde_json::json!([])),
        }
    }

    /// Refresh sessions from a caller-supplied gateway client.
    /// This is the only method that dials; tests never call it.
    pub fn refresh_sessions(&mut self, client: &GatewayClient) -> hermes_gateway::Result<()> {
        self.sessions = SessionList::fetch(client)?;
        Ok(())
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new("main")
    }
}
