//! The application shell composing every UI region.
//!
//! [`AppShell`] is plain Rust state so it can be assembled and asserted
//! without a display or a gateway. The [`gpui::Render`] implementation is a
//! minimal 0.2.2 text node mirroring the proven spike; it is never invoked in
//! tests, which keeps them display-free.

use hermes_chat::Composer;
use hermes_core::Config;
use hermes_gpui::{GlobalState, Theme, WindowType};

use crate::{
    command_palette::CommandPalette, settings::Settings, sidebar::Sidebar, statusbar::Statusbar,
    titlebar::Titlebar, workspace::Workspace,
};

#[derive(Debug, Clone)]
pub struct AppShell {
    pub sidebar: Sidebar,
    pub titlebar: Titlebar,
    pub statusbar: Statusbar,
    pub workspace: Workspace,
    pub command_palette: CommandPalette,
    pub settings: Settings,
    pub theme: Theme,
    pub window: WindowType,
    pub state: GlobalState,
    pub composer: Composer,
}

impl AppShell {
    /// Compose a shell for `window` using `config`. No gateway is dialed and
    /// no display is opened here.
    pub fn new(window: WindowType, config: Config) -> Self {
        Self {
            sidebar: Sidebar::new(),
            titlebar: Titlebar::new("Hermes"),
            statusbar: Statusbar::new("Ready", config.profile.as_str()),
            workspace: Workspace::new(window.as_str()),
            command_palette: CommandPalette::new(),
            settings: Settings::new(),
            theme: Theme::default(),
            window,
            state: GlobalState::new(config),
            composer: Composer::new(),
        }
    }

    /// Toggle the command palette open/closed.
    pub fn toggle_palette(&mut self) {
        if self.command_palette.open {
            self.command_palette.close();
        } else {
            self.command_palette.open();
        }
    }
}

impl Default for AppShell {
    fn default() -> Self {
        Self::new(WindowType::Main, Config::default())
    }
}

impl gpui::Render for AppShell {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::ParentElement as _;
        let label: gpui::SharedString = format!(
            "hermes · {} · {}",
            self.window.as_str(),
            self.state.config.profile
        )
        .into();
        gpui::div().child(label)
    }
}
