//! Statusbar state — connection status and active profile as plain data.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statusbar {
    pub status: String,
    pub profile: String,
}

impl Statusbar {
    pub fn new(status: impl Into<String>, profile: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            profile: profile.into(),
        }
    }
}

impl Default for Statusbar {
    fn default() -> Self {
        Self::new("Ready", "default")
    }
}
