//! Session list from `GET /api/sessions`.
//!
//! [`parse_sessions`] is pure so tests need no gateway; [`SessionList::fetch`]
//! is the only part that dials, and it uses the already-landed
//! [`GatewayClient::get`].

use hermes_gateway::GatewayClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// REST path probing the gateway's session authority.
pub const SESSIONS_PATH: &str = "/api/sessions";

/// One entry in the session list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// The parsed session list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionList {
    pub sessions: Vec<SessionInfo>,
}

impl SessionList {
    /// Fetch and parse the gateway's session list.
    pub fn fetch(client: &GatewayClient) -> hermes_gateway::Result<Self> {
        Ok(Self::from_value(client.get(SESSIONS_PATH)?))
    }

    /// Parse a gateway JSON response into a session list.
    pub fn from_value(value: Value) -> Self {
        Self {
            sessions: parse_sessions(value),
        }
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

/// Parse a `GET /api/sessions` response into session entries.
///
/// Accepts a bare JSON array or an envelope with a `sessions`, `data` or
/// `items` array. Entries without a string/number `id` are skipped.
pub fn parse_sessions(value: Value) -> Vec<SessionInfo> {
    let entries: Vec<Value> = match value {
        Value::Array(items) => items,
        Value::Object(map) => map
            .get("sessions")
            .or_else(|| map.get("data"))
            .or_else(|| map.get("items"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    entries.iter().filter_map(parse_session).collect()
}

fn parse_session(value: &Value) -> Option<SessionInfo> {
    let id = match value.get("id")? {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        _ => return None,
    };
    let title = value
        .get("title")
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let updated_at = value
        .get("updated_at")
        .or_else(|| value.get("updatedAt"))
        .and_then(Value::as_str)
        .map(str::to_string);

    Some(SessionInfo {
        id,
        title,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_and_malformed_responses_yield_no_sessions() {
        assert!(parse_sessions(Value::Null).is_empty());
        assert!(parse_sessions(json!({ "sessions": "not-array" })).is_empty());
        assert!(SessionList::from_value(json!([])).is_empty());
    }
}
