//! `hermes-hud` — floating HUD, compact composer and session switcher.
//!
//! The HUD is plain Rust state: a [`HudWindow`] holds a [`HudComposer`] for
//! input, a [`SessionSwitcher`] for session selection and the active session.
//! Session activation maps to the gateway's canonical `session.activate`
//! RPC via a pure helper; chat input delegates to
//! [`hermes_chat::Composer`] and produces `prompt.submit`.
//!
//! Every type here is constructible without a gateway connection or a
//! display, so the contract tests in `tests/hud.rs` are fully headless.

pub mod hud_composer;
pub mod hud_session_switcher;
pub mod hud_window;

pub use hud_composer::HudComposer;
pub use hud_session_switcher::{SessionSwitcher, SESSION_ACTIVATE_METHOD};
pub use hud_window::HudWindow;
