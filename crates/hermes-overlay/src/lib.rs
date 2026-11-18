//! `hermes-overlay` — pet overlay, quick entry and wake indicator.
//!
//! Every type here is plain Rust state: the pet overlay is pure frame/step
//! arithmetic, quick entry delegates submission to [`hermes_chat::Composer`]
//! so it can only ever emit the canonical `prompt.submit` request, and the
//! wake indicator flips state through a pure toggle helper. No gateway client,
//! GPUI window or animation timer is ever constructed, which keeps the
//! contract tests in `tests/overlay.rs` fully headless.

pub mod pet_overlay;
pub mod quick_entry;
pub mod wake_indicator;

pub use pet_overlay::{PetOverlay, PET_FRAME_COUNT};
pub use quick_entry::{QuickEntry, PROMPT_SUBMIT_METHOD};
pub use wake_indicator::{toggle_state, WakeIndicator, WakeState};
