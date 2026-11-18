//! Command palette state — commands, a query and open/closed state.
//!
//! Filtering is pure string matching, so the palette is fully testable
//! without a display.

/// One command offered by the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPalette {
    pub commands: Vec<Command>,
    pub query: String,
    pub open: bool,
}

impl CommandPalette {
    /// Build the default palette with the shell's common commands.
    pub fn new() -> Self {
        Self {
            commands: vec![
                Command {
                    id: "new-session".into(),
                    label: "New Session".into(),
                },
                Command {
                    id: "settings".into(),
                    label: "Open Settings".into(),
                },
                Command {
                    id: "plugins".into(),
                    label: "Manage Plugins".into(),
                },
                Command {
                    id: "toggle-sidebar".into(),
                    label: "Toggle Sidebar".into(),
                },
            ],
            query: String::new(),
            open: false,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }

    /// Commands whose id or label contains the query, case-insensitively.
    /// An empty query matches every command.
    pub fn matching(&self) -> Vec<&Command> {
        let needle = self.query.to_ascii_lowercase();
        self.commands
            .iter()
            .filter(|command| {
                command.id.contains(needle.as_str())
                    || command.label.to_ascii_lowercase().contains(needle.as_str())
            })
            .collect()
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}
