//! Sidebar navigation state — plain data, no display required.

/// One sidebar navigation entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
}

/// Sidebar with a fixed set of default entries and a selected item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidebar {
    pub items: Vec<SidebarItem>,
    pub selected: Option<String>,
}

impl Sidebar {
    /// Build the default sidebar (chat, sessions, plugins, settings).
    pub fn new() -> Self {
        Self {
            items: vec![
                SidebarItem {
                    id: "chat".into(),
                    label: "Chat".into(),
                },
                SidebarItem {
                    id: "sessions".into(),
                    label: "Sessions".into(),
                },
                SidebarItem {
                    id: "plugins".into(),
                    label: "Plugins".into(),
                },
                SidebarItem {
                    id: "settings".into(),
                    label: "Settings".into(),
                },
            ],
            selected: Some("chat".into()),
        }
    }

    /// Select an item by id. Returns false when the id is not present.
    pub fn select(&mut self, id: &str) -> bool {
        if self.items.iter().any(|item| item.id == id) {
            self.selected = Some(id.to_string());
            true
        } else {
            false
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}
