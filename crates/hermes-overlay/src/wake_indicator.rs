//! Wake/sleep indicator with a pure toggle.
//!
//! [`toggle_state`] is a pure mapping from one [`WakeState`] to the other, so
//! flipping the indicator never needs a timer, gateway event or display.

use hermes_gpui::WindowType;

/// Whether the assistant is awake (listening) or asleep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WakeState {
    /// Not listening — the default resting state.
    #[default]
    Asleep,
    /// Actively listening.
    Awake,
}

/// PURE helper: return the opposite wake state.
pub fn toggle_state(state: WakeState) -> WakeState {
    match state {
        WakeState::Asleep => WakeState::Awake,
        WakeState::Awake => WakeState::Asleep,
    }
}

/// State of the wake indicator overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeIndicator {
    /// Current wake/sleep state.
    pub state: WakeState,
    /// Canonical window kind for this overlay.
    pub window: WindowType,
}

impl WakeIndicator {
    /// Build an indicator in the default (asleep) state.
    pub fn new() -> Self {
        Self {
            state: WakeState::default(),
            window: WindowType::WakeIndicator,
        }
    }

    pub fn is_awake(&self) -> bool {
        self.state == WakeState::Awake
    }

    pub fn is_asleep(&self) -> bool {
        self.state == WakeState::Asleep
    }

    /// PURE: a new indicator holding the opposite state.
    pub fn toggled(&self) -> Self {
        Self {
            state: toggle_state(self.state),
            window: self.window,
        }
    }

    /// Flip this indicator's state via the pure helper.
    pub fn toggle(&mut self) {
        self.state = toggle_state(self.state);
    }

    pub fn set_awake(&mut self) {
        self.state = WakeState::Awake;
    }

    pub fn set_asleep(&mut self) {
        self.state = WakeState::Asleep;
    }
}

impl Default for WakeIndicator {
    fn default() -> Self {
        Self::new()
    }
}
