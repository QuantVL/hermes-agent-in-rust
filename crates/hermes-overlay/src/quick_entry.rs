//! Compact quick-entry input.
//!
//! All submission logic lives in [`hermes_chat::Composer`]; this type only
//! wraps it for a smaller input surface. [`QuickEntry::submit`] therefore
//! delegates straight to [`Composer::submit`] and can only produce the
//! canonical `prompt.submit` request — there is no second implementation of
//! the chat protocol here.

use hermes_chat::Composer;
use hermes_core::{Request, Result};
use hermes_gpui::WindowType;

/// The one chat-submission method this overlay can emit.
pub use hermes_chat::PROMPT_SUBMIT_METHOD;

/// State for the compact quick-entry box.
#[derive(Debug, Clone)]
pub struct QuickEntry {
    /// The wrapped chat composer — the single source of submission truth.
    pub composer: Composer,
    /// Current draft in the compact input box.
    pub draft: String,
    /// Canonical window kind for this overlay.
    pub window: WindowType,
}

impl QuickEntry {
    pub fn new() -> Self {
        Self {
            composer: Composer::new(),
            draft: String::new(),
            window: WindowType::QuickEntry,
        }
    }

    /// Submit `text` through [`Composer::submit`] as a `prompt.submit` request.
    pub fn submit(&self, text: &str) -> Result<Request> {
        self.composer.submit(text)
    }

    /// Submit the current draft, then clear it.
    pub fn submit_draft(&mut self) -> Result<Request> {
        let request = self.composer.submit(&self.draft)?;
        self.draft.clear();
        Ok(request)
    }

    pub fn clear(&mut self) {
        self.draft.clear();
    }
}

impl Default for QuickEntry {
    fn default() -> Self {
        Self::new()
    }
}
