//! Session switcher for the compact HUD.
//!
//! [`SessionSwitcher`] keeps the gateway's session list plus the current
//! selection. Building the activation request is a *pure* mapping
//! ([`SessionSwitcher::activation_request`]): it takes a [`SessionInfo`] and
//! returns a JSON-RPC [`Request`] without dialing a gateway or touching a
//! display, so it can be asserted in headless tests.
//!
//! `session.activate` is part of the gateway's canonical RPC vocabulary — do
//! not substitute another method name.

use hermes_chat::SessionInfo;
use hermes_core::Request;
use serde_json::json;

/// The single gateway RPC method that activates a session.
pub const SESSION_ACTIVATE_METHOD: &str = "session.activate";

/// State for picking the session the HUD should display.
#[derive(Debug, Clone, Default)]
pub struct SessionSwitcher {
    /// Sessions last reported by the gateway.
    pub sessions: Vec<SessionInfo>,
    /// The currently selected session, if any.
    pub selected: Option<SessionInfo>,
}

impl SessionSwitcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a switcher over a known session list with no selection yet.
    pub fn from_sessions(sessions: Vec<SessionInfo>) -> Self {
        Self {
            sessions,
            selected: None,
        }
    }

    /// PURE helper: map a selected session to its `session.activate` request.
    ///
    /// The request carries the session id under `session_id`. No I/O, no
    /// gateway client and no window is involved.
    pub fn activation_request(session: &SessionInfo) -> Request {
        Request::new(
            SESSION_ACTIVATE_METHOD,
            Some(json!({ "session_id": session.id })),
        )
    }

    /// Record `session` as selected and return the activation request for it.
    pub fn select(&mut self, session: SessionInfo) -> Request {
        let request = Self::activation_request(&session);
        self.selected = Some(session);
        request
    }

    /// Find a known session by id, select it and return its activation request.
    pub fn select_by_id(&mut self, id: &str) -> Option<Request> {
        let session = self.sessions.iter().find(|entry| entry.id == id)?.clone();
        Some(self.select(session))
    }

    /// Id of the selected session, if one has been chosen.
    pub fn selected_id(&self) -> Option<&str> {
        self.selected.as_ref().map(|session| session.id.as_str())
    }
}
