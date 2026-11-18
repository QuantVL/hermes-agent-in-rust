//! Compact composer used by the floating HUD.
//!
//! All submission logic lives in [`hermes_chat::Composer`]; this type only
//! wraps it for the HUD's smaller input surface. [`HudComposer::submit`]
//! therefore delegates straight to [`Composer::submit`] and produces the
//! canonical `prompt.submit` request — no second implementation of the
//! protocol is allowed.

use hermes_chat::Composer;
use hermes_core::{Request, Result};

/// Composer state for the compact HUD (plus the in-progress draft text).
#[derive(Debug, Clone, Default)]
pub struct HudComposer {
    /// The wrapped chat composer — the single source of submission truth.
    pub composer: Composer,
    /// Current draft in the compact input box.
    pub draft: String,
}

impl HudComposer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit `text` through [`Composer::submit`] as a `prompt.submit` request.
    pub fn submit(&self, text: &str) -> Result<Request> {
        self.composer.submit(text)
    }

    /// Submit and then clear the HUD draft.
    pub fn submit_draft(&mut self) -> Result<Request> {
        let request = self.composer.submit(&self.draft)?;
        self.draft.clear();
        Ok(request)
    }

    pub fn clear(&mut self) {
        self.draft.clear();
    }
}
