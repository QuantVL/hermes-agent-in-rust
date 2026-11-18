//! `hermes-shell` — main application shell UI and plugin host API.
//!
//! The shell composes the sidebar, titlebar, statusbar, workspace, command
//! palette and settings, and defines the [`PluginHost`] trait that
//! `hermes-plugins` implements. Every type here is plain Rust state: tests
//! build the shell without a display and mock the host without a gateway.

pub mod app_shell;
pub mod command_palette;
pub mod plugin_host;
pub mod settings;
pub mod sidebar;
pub mod statusbar;
pub mod titlebar;
pub mod workspace;

pub use app_shell::AppShell;
pub use command_palette::{Command, CommandPalette};
pub use plugin_host::PluginHost;
pub use settings::{Settings, SettingsTab};
pub use sidebar::{Sidebar, SidebarItem};
pub use statusbar::Statusbar;
pub use titlebar::Titlebar;
pub use workspace::Workspace;
