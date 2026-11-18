//! Settings state with the shell's seven-tab structure.
//!
//! Tabs are plain enum values with stable wire strings, so the tab set can be
//! asserted without constructing a window.

/// The seven settings tabs, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Appearance,
    Models,
    Tools,
    Editor,
    Plugins,
    Gateway,
}

impl SettingsTab {
    /// All seven tabs in display order — the single source of truth for the
    /// tab set and its count.
    pub const ALL: [SettingsTab; 7] = [
        SettingsTab::General,
        SettingsTab::Appearance,
        SettingsTab::Models,
        SettingsTab::Tools,
        SettingsTab::Editor,
        SettingsTab::Plugins,
        SettingsTab::Gateway,
    ];

    /// Canonical spelling used in diagnostics and persistence.
    pub fn as_str(&self) -> &'static str {
        match self {
            SettingsTab::General => "general",
            SettingsTab::Appearance => "appearance",
            SettingsTab::Models => "models",
            SettingsTab::Tools => "tools",
            SettingsTab::Editor => "editor",
            SettingsTab::Plugins => "plugins",
            SettingsTab::Gateway => "gateway",
        }
    }

    /// Human-readable label shown in the tab bar.
    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::Models => "Models",
            SettingsTab::Tools => "Tools",
            SettingsTab::Editor => "Editor",
            SettingsTab::Plugins => "Plugins",
            SettingsTab::Gateway => "Gateway",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub active_tab: SettingsTab,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            active_tab: SettingsTab::General,
        }
    }

    pub fn select(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
