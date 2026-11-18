//! Chat message model.
//!
//! Plain Rust values (no GPUI types) so the transcript can be built and
//! asserted without a display. Later UI tasks map these onto GPUI widgets.

use std::time::{SystemTime, UNIX_EPOCH};

/// Who authored a chat message. `Assistant` is the default because that is
/// the role of the buffer a streamed reply is folded into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Role {
    User,
    #[default]
    Assistant,
    System,
}

/// One message in the chat transcript.
///
/// `ts` is a unix epoch timestamp in milliseconds. It is `0` for
/// [`Message::default`], which keeps pure tests deterministic; the named
/// constructors stamp the current time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub ts: u64,
}

impl Message {
    /// Create a message with the current timestamp.
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            ts: now_millis(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Append one streamed text fragment to this message's content.
    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_constructors_set_role_and_timestamp() {
        let user = Message::user("hi");
        assert_eq!(user.role, Role::User);
        assert!(user.ts > 0);
        assert_eq!(Message::system("cfg").role, Role::System);
    }
}
