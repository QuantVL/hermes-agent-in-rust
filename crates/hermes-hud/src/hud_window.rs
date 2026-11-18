//! Floating HUD window state.
//!
//! [`HudWindow`] composes the compact composer, the session switcher and the
//! active-session marker. It is plain Rust state, so tests can build it with
//! no display, no GPUI event loop and no gateway. The [`gpui::Render`] impl is
//! the same minimal 0.2.2 text node used by the shell and is never invoked in
//! headless tests.

use hermes_chat::SessionInfo;
use hermes_core::Config;
use hermes_gateway::GatewayEvent;
use hermes_gpui::{GlobalState, Theme, WindowType};

use crate::{HudComposer, SessionSwitcher};

/// State of the floating HUD window.
#[derive(Debug, Clone)]
pub struct HudWindow {
    /// Compact prompt input.
    pub composer: HudComposer,
    /// Session picker.
    pub switcher: SessionSwitcher,
    /// The session currently shown by the HUD, if any.
    pub active_session: Option<SessionInfo>,
    /// Whether the gateway transport is currently connected.
    pub connected: bool,
    pub theme: Theme,
    pub window: WindowType,
    pub state: GlobalState,
}

impl HudWindow {
    /// Build a HUD from `config`, disconnected and with no active session.
    pub fn new(config: Config) -> Self {
        Self {
            composer: HudComposer::new(),
            switcher: SessionSwitcher::new(),
            active_session: None,
            connected: false,
            theme: Theme::default(),
            window: WindowType::Hud,
            state: GlobalState::new(config),
        }
    }

    /// PURE builder-style helper: return a HUD with `session` active.
    pub fn with_active_session(mut self, session: SessionInfo) -> Self {
        self.active_session = Some(session);
        self
    }

    /// Mutating counterpart to [`Self::with_active_session`].
    pub fn set_active_session(&mut self, session: SessionInfo) {
        self.active_session = Some(session);
    }

    pub fn has_active_session(&self) -> bool {
        self.active_session.is_some()
    }

    pub fn active_session_id(&self) -> Option<&str> {
        self.active_session
            .as_ref()
            .map(|session| session.id.as_str())
    }

    /// Fold a gateway transport event into the HUD's connection status.
    pub fn on_gateway_event(&mut self, event: GatewayEvent) {
        match event {
            GatewayEvent::Connected => self.connected = true,
            GatewayEvent::Disconnected(_) => self.connected = false,
            _ => {}
        }
    }
}

impl Default for HudWindow {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl gpui::Render for HudWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::ParentElement as _;
        let label: gpui::SharedString = format!(
            "hermes hud · {} · {}",
            self.window.as_str(),
            self.active_session_id().unwrap_or("no session")
        )
        .into();
        gpui::div().child(label)
    }
}
