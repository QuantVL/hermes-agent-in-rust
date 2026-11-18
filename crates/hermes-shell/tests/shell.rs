//! Task 5 shell-composition contract tests — written before the
//! implementation. Gateway-free and display-free: every region is plain Rust
//! state, so no window, socket or gateway is needed.

use hermes_core::Config;
use hermes_gpui::WindowType;
use hermes_shell::{AppShell, CommandPalette, Settings, SettingsTab, Sidebar};

#[test]
fn settings_has_exactly_seven_distinct_tabs() {
    assert_eq!(SettingsTab::ALL.len(), 7, "Settings must expose 7 tabs");

    let mut seen = std::collections::HashSet::new();
    for tab in SettingsTab::ALL {
        assert!(seen.insert(tab.as_str()), "duplicate tab {}", tab.as_str());
    }
}

#[test]
fn settings_defaults_to_general_and_can_select_a_tab() {
    let mut settings = Settings::default();
    assert_eq!(settings.active_tab, SettingsTab::General);

    settings.select(SettingsTab::Plugins);
    assert_eq!(settings.active_tab, SettingsTab::Plugins);
    assert_eq!(settings.active_tab.as_str(), "plugins");
    assert_eq!(settings.active_tab.label(), "Plugins");
}

#[test]
fn command_palette_filters_commands_by_query() {
    let mut palette = CommandPalette::new();
    assert!(!palette.open, "palette starts closed");

    palette.open();
    assert!(palette.open);

    palette.query = "set".into();
    let labels: Vec<&str> = palette
        .matching()
        .iter()
        .map(|command| command.label.as_str())
        .collect();
    assert!(
        labels.iter().any(|label| label.contains("Settings")),
        "query 'set' must match the Settings command"
    );

    palette.close();
    assert!(!palette.open);
    assert!(palette.query.is_empty());
}

#[test]
fn sidebar_tracks_selected_item() {
    let mut sidebar = Sidebar::new();
    assert!(!sidebar.items.is_empty(), "sidebar has default entries");

    assert!(sidebar.select("plugins"));
    assert_eq!(sidebar.selected.as_deref(), Some("plugins"));

    assert!(!sidebar.select("does-not-exist"));
    assert_eq!(sidebar.selected.as_deref(), Some("plugins"));
}

#[test]
fn app_shell_composes_all_shell_regions() {
    let shell = AppShell::new(WindowType::Main, Config::default());

    assert_eq!(shell.window, WindowType::Main);
    assert_eq!(shell.titlebar.title, "Hermes");
    assert!(!shell.sidebar.items.is_empty());
    assert!(!shell.command_palette.commands.is_empty());
    assert_eq!(shell.settings.active_tab, SettingsTab::General);
    assert_eq!(shell.state.config.profile, "default");
    assert!(shell.workspace.sessions.is_empty());
}
