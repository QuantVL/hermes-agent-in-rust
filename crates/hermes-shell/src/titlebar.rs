//! Titlebar state — the window's displayed title as plain data.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Titlebar {
    pub title: String,
}

impl Titlebar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }
}

impl Default for Titlebar {
    fn default() -> Self {
        Self::new("Hermes")
    }
}
