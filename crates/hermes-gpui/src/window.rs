//! The five shell window kinds and their stable wire/display names.

/// A kind of window the application can open.
///
/// `as_str()` is the single canonical spelling used in window titles,
/// diagnostics and later task IPC; do not spell these inline elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Main,
    Hud,
    PetOverlay,
    QuickEntry,
    WakeIndicator,
}

impl WindowType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowType::Main => "main",
            WindowType::Hud => "hud",
            WindowType::PetOverlay => "pet-overlay",
            WindowType::QuickEntry => "quick-entry",
            WindowType::WakeIndicator => "wake-indicator",
        }
    }
}
